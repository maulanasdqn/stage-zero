use std::collections::HashMap;

use crate::entities::{
    Applicant, Application, ApplicationStatus, Attachment, BatchInfo, Contact, DirectoryEntry,
    EmailPreview, EmailTemplate, Founder, HuntCriteria, HuntEvent, HuntOutcome, HuntReport,
    NewApplication, OutgoingEmail, Startup, StartupInsight, StatusSummary, TemplateContext,
};
use crate::errors::DomainError;
use crate::pagination::{
    ApplicationQuery, ExploreQuery, ExploreResult, Paginated, compare_entries, matches_search,
};
use crate::traits::{
    ApplicationRepository, ContactFinder, Delay, FounderSource, HuntProgress, Mailer,
    StartupDirectory,
};

#[derive(Debug, Clone)]
pub struct OutreachProfile {
    pub applicant: Applicant,
    pub template: EmailTemplate,
    pub resume: Option<Attachment>,
    pub daily_limit: usize,
}

pub struct HuntUseCase<D, F, C, R, M, W> {
    directory: D,
    founder_source: F,
    contact_finder: C,
    repository: R,
    mailer: M,
    delay: W,
    profile: OutreachProfile,
}

struct ProcessedStartup {
    outcome: HuntOutcome,
    contact: Option<Contact>,
    email: Option<EmailPreview>,
}

impl<D, F, C, R, M, W> HuntUseCase<D, F, C, R, M, W>
where
    D: StartupDirectory,
    F: FounderSource,
    C: ContactFinder,
    R: ApplicationRepository,
    M: Mailer,
    W: Delay,
{
    pub fn new(
        directory: D,
        founder_source: F,
        contact_finder: C,
        repository: R,
        mailer: M,
        delay: W,
        profile: OutreachProfile,
    ) -> Self {
        Self {
            directory,
            founder_source,
            contact_finder,
            repository,
            mailer,
            delay,
            profile,
        }
    }

    pub async fn execute<P: HuntProgress>(
        &self,
        criteria: &HuntCriteria,
        progress: &P,
    ) -> Result<HuntReport, DomainError> {
        self.validate(criteria)?;

        let startups = self.directory.fetch(criteria).await?;
        progress.on_discovered(startups.len());

        let mut report = HuntReport {
            discovered: startups.len(),
            ..HuntReport::default()
        };
        let mut remaining_sends = match criteria.dry_run {
            true => None,
            false => {
                let sent_today = self.repository.count_applied_today().await?;
                Some(self.profile.daily_limit.saturating_sub(sent_today))
            }
        };
        let mut processed = 0usize;
        let mut pending_delay = false;

        for startup in &startups {
            if criteria.limit.is_some_and(|limit| processed >= limit) {
                break;
            }

            if self.repository.exists(&startup.slug).await? {
                let event = HuntEvent {
                    slug: startup.slug.clone(),
                    company_name: startup.name.clone(),
                    outcome: HuntOutcome::AlreadyProcessed,
                    contact: None,
                    email: None,
                };
                report.register(&event.outcome);
                progress.on_event(&event);
                continue;
            }

            if remaining_sends == Some(0) {
                report.daily_limit_reached = true;
                break;
            }

            if pending_delay {
                self.delay.wait().await;
            }

            let processed_startup = self.process(startup, criteria.dry_run).await?;
            if processed_startup.outcome == HuntOutcome::Applied {
                remaining_sends = remaining_sends.map(|remaining| remaining.saturating_sub(1));
            }

            let event = HuntEvent {
                slug: startup.slug.clone(),
                company_name: startup.name.clone(),
                outcome: processed_startup.outcome,
                contact: processed_startup.contact,
                email: processed_startup.email,
            };
            report.register(&event.outcome);
            progress.on_event(&event);

            processed += 1;
            pending_delay = true;
        }

        Ok(report)
    }

    fn validate(&self, criteria: &HuntCriteria) -> Result<(), DomainError> {
        if criteria.batch.trim().is_empty() {
            return Err(DomainError::InvalidInput("batch must not be empty".into()));
        }
        if !criteria.dry_run && self.profile.resume.is_none() {
            return Err(DomainError::InvalidInput(
                "a resume attachment is required unless running with dry run".into(),
            ));
        }
        if !criteria.dry_run && self.profile.daily_limit == 0 {
            return Err(DomainError::InvalidInput(
                "daily limit is zero, nothing can be sent".into(),
            ));
        }
        Ok(())
    }

    async fn process(
        &self,
        startup: &Startup,
        dry_run: bool,
    ) -> Result<ProcessedStartup, DomainError> {
        if startup
            .website
            .as_deref()
            .is_none_or(|website| website.trim().is_empty())
        {
            self.record_unless_dry_run(dry_run, startup, None, ApplicationStatus::Skipped)
                .await?;
            return Ok(ProcessedStartup {
                outcome: HuntOutcome::SkippedNoWebsite,
                contact: None,
                email: None,
            });
        }

        let founders = self
            .founder_source
            .founders(startup)
            .await
            .unwrap_or_default();
        let contact = self
            .contact_finder
            .find(startup, &founders)
            .await
            .ok()
            .flatten();

        let Some(contact) = contact else {
            self.record_unless_dry_run(dry_run, startup, None, ApplicationStatus::FailedNoContact)
                .await?;
            return Ok(ProcessedStartup {
                outcome: HuntOutcome::NoContact,
                contact: None,
                email: None,
            });
        };

        let email =
            self.compose_email(startup, &founders, &contact.email, contact.founder.as_ref());
        let preview = EmailPreview::from(&email);

        if dry_run {
            return Ok(ProcessedStartup {
                outcome: HuntOutcome::Previewed,
                contact: Some(contact),
                email: Some(preview),
            });
        }

        let outcome = match self.mailer.send(&email).await {
            Ok(_) => {
                self.repository
                    .record(NewApplication::for_startup(
                        startup,
                        Some(contact.email.clone()),
                        ApplicationStatus::Applied,
                    ))
                    .await?;
                HuntOutcome::Applied
            }
            Err(error) => HuntOutcome::SendFailed(error.to_string()),
        };

        Ok(ProcessedStartup {
            outcome,
            contact: Some(contact),
            email: Some(preview),
        })
    }

    async fn record_unless_dry_run(
        &self,
        dry_run: bool,
        startup: &Startup,
        contact_email: Option<String>,
        status: ApplicationStatus,
    ) -> Result<(), DomainError> {
        if dry_run {
            return Ok(());
        }
        self.repository
            .record(NewApplication::for_startup(startup, contact_email, status))
            .await
    }

    fn compose_email(
        &self,
        startup: &Startup,
        founders: &[Founder],
        recipient: &str,
        contact_founder: Option<&Founder>,
    ) -> OutgoingEmail {
        let sole_founder = match founders {
            [only_founder] => Some(only_founder),
            _ => None,
        };
        let greeted_founder = contact_founder.or(sole_founder);
        let context = TemplateContext {
            startup,
            founder: greeted_founder,
            applicant: &self.profile.applicant,
        };
        let (subject, body) = self.profile.template.render(&context);
        self.outgoing(EmailPreview {
            to: recipient.to_string(),
            subject,
            body,
        })
    }

    fn outgoing(&self, preview: EmailPreview) -> OutgoingEmail {
        OutgoingEmail {
            to: preview.to,
            reply_to: Some(self.profile.applicant.email.clone()),
            subject: preview.subject,
            body: preview.body,
            attachments: self.profile.resume.iter().cloned().collect(),
        }
    }

    pub async fn batches(&self) -> Result<Vec<BatchInfo>, DomainError> {
        self.directory.batches().await
    }

    pub async fn explore(
        &self,
        criteria: &HuntCriteria,
        query: &ExploreQuery,
    ) -> Result<ExploreResult, DomainError> {
        if criteria.batch.trim().is_empty() {
            return Err(DomainError::InvalidInput("batch must not be empty".into()));
        }
        let startups = self.directory.fetch(criteria).await?;
        let statuses = self
            .repository
            .list(None)
            .await?
            .into_iter()
            .map(|application| (application.yc_slug, application.status))
            .collect::<HashMap<_, _>>();
        let entries = startups
            .into_iter()
            .map(|startup| DirectoryEntry {
                status: statuses.get(&startup.slug).copied(),
                startup,
            })
            .collect::<Vec<_>>();
        let batch_total = entries.len();
        let tracked = entries
            .iter()
            .filter(|entry| entry.status.is_some())
            .count();

        let needle = query
            .search
            .as_deref()
            .map(str::trim)
            .filter(|needle| !needle.is_empty())
            .map(str::to_lowercase);
        let mut visible = entries
            .into_iter()
            .filter(|entry| !query.hide_contacted || entry.status.is_none())
            .filter(|entry| {
                needle
                    .as_deref()
                    .is_none_or(|needle| matches_search(entry, needle))
            })
            .collect::<Vec<_>>();
        if let Some(sort) = query.sort {
            visible
                .sort_by(|left, right| query.direction.apply(compare_entries(left, right, sort)));
        }

        let Paginated { data, pagination } = query.page.slice(visible);
        Ok(ExploreResult {
            data,
            pagination,
            batch_total,
            tracked,
        })
    }

    pub async fn inspect(&self, startup: &Startup) -> Result<StartupInsight, DomainError> {
        let founders = self
            .founder_source
            .founders(startup)
            .await
            .unwrap_or_default();
        let has_website = startup
            .website
            .as_deref()
            .is_some_and(|website| !website.trim().is_empty());
        let contact = match has_website {
            true => self
                .contact_finder
                .find(startup, &founders)
                .await
                .ok()
                .flatten(),
            false => None,
        };
        let recipient = contact
            .as_ref()
            .map(|contact| contact.email.as_str())
            .unwrap_or_default();
        let contact_founder = contact
            .as_ref()
            .and_then(|contact| contact.founder.as_ref());
        let email =
            EmailPreview::from(&self.compose_email(startup, &founders, recipient, contact_founder));
        let status = self
            .repository
            .find(&startup.slug)
            .await?
            .map(|application| application.status);
        Ok(StartupInsight {
            highlights: self.profile.applicant.highlights_for(startup),
            startup: startup.clone(),
            founders,
            contact,
            email,
            status,
        })
    }

    pub async fn send_reviewed(
        &self,
        startup: &Startup,
        email: EmailPreview,
    ) -> Result<HuntOutcome, DomainError> {
        let recipient = email.to.trim().to_string();
        if !is_email_address(&recipient) {
            return Err(DomainError::InvalidInput(format!(
                "`{recipient}` is not a valid recipient address"
            )));
        }
        if email.subject.trim().is_empty() || email.body.trim().is_empty() {
            return Err(DomainError::InvalidInput(
                "subject and body must not be empty".into(),
            ));
        }
        if self.profile.resume.is_none() {
            return Err(DomainError::InvalidInput(
                "a resume attachment is required to send".into(),
            ));
        }
        let already_applied = self
            .repository
            .find(&startup.slug)
            .await?
            .is_some_and(|application| application.status == ApplicationStatus::Applied);
        if already_applied {
            return Err(DomainError::InvalidInput(format!(
                "{} was already applied to",
                startup.name
            )));
        }
        if self.repository.count_applied_today().await? >= self.profile.daily_limit {
            return Err(DomainError::InvalidInput(
                "daily send limit reached, try again tomorrow".into(),
            ));
        }

        let outgoing = self.outgoing(EmailPreview {
            to: recipient.clone(),
            ..email
        });
        match self.mailer.send(&outgoing).await {
            Ok(_) => {
                self.repository
                    .record(NewApplication::for_startup(
                        startup,
                        Some(recipient),
                        ApplicationStatus::Applied,
                    ))
                    .await?;
                Ok(HuntOutcome::Applied)
            }
            Err(error) => Ok(HuntOutcome::SendFailed(error.to_string())),
        }
    }

    pub async fn skip(&self, startup: &Startup) -> Result<(), DomainError> {
        self.repository
            .record(NewApplication::for_startup(
                startup,
                None,
                ApplicationStatus::Skipped,
            ))
            .await
    }
}

fn is_email_address(value: &str) -> bool {
    match value.split_once('@') {
        Some((local, domain)) => {
            !local.is_empty()
                && domain.contains('.')
                && !domain.starts_with('.')
                && !domain.ends_with('.')
                && !value.chars().any(char::is_whitespace)
        }
        None => false,
    }
}

pub struct StatusUseCase<R> {
    repository: R,
}

impl<R: ApplicationRepository> StatusUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn list(
        &self,
        status: Option<ApplicationStatus>,
    ) -> Result<Vec<Application>, DomainError> {
        self.repository.list(status).await
    }

    pub async fn page(
        &self,
        query: &ApplicationQuery,
    ) -> Result<Paginated<Application>, DomainError> {
        self.repository.list_page(query).await
    }

    pub async fn summary(&self) -> Result<StatusSummary, DomainError> {
        let applications = self.repository.list(None).await?;
        let count = |status: ApplicationStatus| {
            applications
                .iter()
                .filter(|application| application.status == status)
                .count()
        };
        Ok(StatusSummary {
            total: applications.len(),
            applied: count(ApplicationStatus::Applied),
            skipped: count(ApplicationStatus::Skipped),
            failed_no_contact: count(ApplicationStatus::FailedNoContact),
        })
    }
}

pub struct ResetUseCase<R> {
    repository: R,
}

impl<R: ApplicationRepository> ResetUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, yc_slug: Option<String>) -> Result<usize, DomainError> {
        self.repository.reset(yc_slug).await
    }
}
