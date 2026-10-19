use crate::entities::{
    Applicant, Application, ApplicationStatus, Attachment, Contact, EmailPreview, EmailTemplate,
    Founder, HuntCriteria, HuntEvent, HuntOutcome, HuntReport, NewApplication, OutgoingEmail,
    Startup, StatusSummary, TemplateContext,
};
use crate::errors::DomainError;
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

        let email = self.compose_email(startup, &founders, &contact);
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
        contact: &Contact,
    ) -> OutgoingEmail {
        let sole_founder = match founders {
            [only_founder] => Some(only_founder),
            _ => None,
        };
        let greeted_founder = contact.founder.as_ref().or(sole_founder);
        let context = TemplateContext {
            startup,
            founder: greeted_founder,
            applicant: &self.profile.applicant,
        };
        let (subject, body) = self.profile.template.render(&context);
        OutgoingEmail {
            to: contact.email.clone(),
            reply_to: Some(self.profile.applicant.email.clone()),
            subject,
            body,
            attachments: self.profile.resume.iter().cloned().collect(),
        }
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
