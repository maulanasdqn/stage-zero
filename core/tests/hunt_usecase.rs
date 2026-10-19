use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use stagezero_core::DomainError;
use stagezero_core::entities::{
    Applicant, Application, ApplicationStatus, Attachment, Contact, ContactOrigin, EmailTemplate,
    Founder, HuntCriteria, HuntEvent, HuntOutcome, NewApplication, OutgoingEmail, Startup,
    TeamSizeRange,
};
use stagezero_core::traits::{
    ApplicationRepository, ContactFinder, Delay, FounderSource, HuntProgress, Mailer,
    StartupDirectory,
};
use stagezero_core::usecases::{HuntUseCase, OutreachProfile, ResetUseCase, StatusUseCase};

struct FixedDirectory(Vec<Startup>);

impl StartupDirectory for FixedDirectory {
    async fn fetch(&self, _criteria: &HuntCriteria) -> Result<Vec<Startup>, DomainError> {
        Ok(self.0.clone())
    }
}

struct FixedFounders(HashMap<String, Vec<Founder>>);

impl FounderSource for FixedFounders {
    async fn founders(&self, startup: &Startup) -> Result<Vec<Founder>, DomainError> {
        Ok(self.0.get(&startup.slug).cloned().unwrap_or_default())
    }
}

struct FixedContacts(HashMap<String, String>);

impl ContactFinder for FixedContacts {
    async fn find(
        &self,
        startup: &Startup,
        founders: &[Founder],
    ) -> Result<Option<Contact>, DomainError> {
        Ok(self.0.get(&startup.slug).map(|email| Contact {
            email: email.clone(),
            founder: founders.first().cloned(),
            origin: ContactOrigin::Scraped,
        }))
    }
}

#[derive(Clone, Default)]
struct MemoryRepository {
    rows: Arc<Mutex<Vec<Application>>>,
}

impl MemoryRepository {
    fn seeded(slug: &str) -> Self {
        let repository = Self::default();
        repository.rows.lock().unwrap().push(Application {
            id: 1,
            yc_slug: slug.into(),
            company_name: slug.into(),
            website: None,
            contact_email: None,
            status: ApplicationStatus::Applied,
            applied_at: "2026-01-01 00:00:00".into(),
        });
        repository
    }

    fn statuses(&self) -> Vec<(String, ApplicationStatus)> {
        self.rows
            .lock()
            .unwrap()
            .iter()
            .map(|row| (row.yc_slug.clone(), row.status))
            .collect()
    }
}

impl ApplicationRepository for MemoryRepository {
    async fn exists(&self, yc_slug: &str) -> Result<bool, DomainError> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .iter()
            .any(|row| row.yc_slug == yc_slug))
    }

    async fn record(&self, application: NewApplication) -> Result<(), DomainError> {
        let mut rows = self.rows.lock().unwrap();
        let id = rows.len() as i64 + 1;
        rows.push(Application {
            id,
            yc_slug: application.yc_slug,
            company_name: application.company_name,
            website: application.website,
            contact_email: application.contact_email,
            status: application.status,
            applied_at: "2026-01-01 00:00:00".into(),
        });
        Ok(())
    }

    async fn list(
        &self,
        status: Option<ApplicationStatus>,
    ) -> Result<Vec<Application>, DomainError> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .iter()
            .filter(|row| status.is_none_or(|status| row.status == status))
            .cloned()
            .collect())
    }

    async fn count_applied_today(&self) -> Result<usize, DomainError> {
        Ok(0)
    }

    async fn reset(&self, yc_slug: Option<String>) -> Result<usize, DomainError> {
        let mut rows = self.rows.lock().unwrap();
        let before = rows.len();
        rows.retain(|row| yc_slug.as_ref().is_some_and(|slug| &row.yc_slug != slug));
        Ok(before - rows.len())
    }
}

#[derive(Clone, Default)]
struct RecordingMailer {
    sent: Arc<Mutex<Vec<OutgoingEmail>>>,
    failing_recipient: Option<String>,
}

impl Mailer for RecordingMailer {
    async fn send(&self, email: &OutgoingEmail) -> Result<String, DomainError> {
        if self.failing_recipient.as_deref() == Some(email.to.as_str()) {
            return Err(DomainError::Mailer("rejected".into()));
        }
        self.sent.lock().unwrap().push(email.clone());
        Ok("message-id".into())
    }
}

#[derive(Clone, Default)]
struct CountingDelay(Arc<AtomicUsize>);

impl Delay for CountingDelay {
    async fn wait(&self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[derive(Default)]
struct CollectingProgress(Mutex<Vec<HuntEvent>>);

impl HuntProgress for CollectingProgress {
    fn on_discovered(&self, _count: usize) {}

    fn on_event(&self, event: &HuntEvent) {
        self.0.lock().unwrap().push(event.clone());
    }
}

fn startup(slug: &str, website: Option<&str>) -> Startup {
    Startup {
        slug: slug.into(),
        name: format!("{slug} Inc"),
        website: website.map(Into::into),
        batch: "Winter 2024".into(),
        team_size: Some(3),
        industries: vec!["Fintech".into()],
        one_liner: "Payments for robots".into(),
    }
}

fn profile(with_resume: bool, daily_limit: usize) -> OutreachProfile {
    OutreachProfile {
        applicant: Applicant {
            name: "Ada".into(),
            email: "ada@example.dev".into(),
            portfolio: Some("https://ada.dev".into()),
        },
        template: EmailTemplate {
            subject: "{{company_name}} x {{applicant_name}}".into(),
            body: "Hi {{founder_first_name}}, loved {{company_name}} in {{ industry }}.".into(),
        },
        resume: with_resume.then(|| Attachment {
            filename: "resume.pdf".into(),
            content: Arc::from(b"%PDF".as_slice()),
        }),
        daily_limit,
    }
}

fn criteria(dry_run: bool, limit: Option<usize>) -> HuntCriteria {
    HuntCriteria {
        batch: "W24".into(),
        team_size: TeamSizeRange { min: 1, max: 10 },
        industry: None,
        limit,
        dry_run,
    }
}

fn scenario() -> (FixedDirectory, FixedFounders, FixedContacts) {
    let directory = FixedDirectory(vec![
        startup("alpha", Some("https://alpha.com")),
        startup("beta", Some("https://beta.com")),
        startup("gamma", None),
        startup("delta", Some("https://delta.com")),
    ]);
    let founders = FixedFounders(HashMap::from([(
        "alpha".to_string(),
        vec![Founder {
            full_name: "Grace Hopper".into(),
            title: Some("CEO".into()),
        }],
    )]));
    let contacts = FixedContacts(HashMap::from([
        ("alpha".to_string(), "grace@alpha.com".to_string()),
        ("delta".to_string(), "hello@delta.com".to_string()),
    ]));
    (directory, founders, contacts)
}

#[tokio::test]
async fn hunt_sends_records_and_skips_duplicates() {
    let (directory, founders, contacts) = scenario();
    let repository = MemoryRepository::seeded("beta");
    let mailer = RecordingMailer::default();
    let delay = CountingDelay::default();
    let use_case = HuntUseCase::new(
        directory,
        founders,
        contacts,
        repository.clone(),
        mailer.clone(),
        delay.clone(),
        profile(true, 10),
    );
    let progress = CollectingProgress::default();

    let report = use_case
        .execute(&criteria(false, None), &progress)
        .await
        .unwrap();

    assert_eq!(report.discovered, 4);
    assert_eq!(report.already_processed, 1);
    assert_eq!(report.applied, 2);
    assert_eq!(report.skipped_no_website, 1);
    assert_eq!(delay.0.load(Ordering::SeqCst), 2);

    let sent = mailer.sent.lock().unwrap();
    assert_eq!(sent.len(), 2);
    assert_eq!(sent[0].to, "grace@alpha.com");
    assert_eq!(sent[0].subject, "alpha Inc x Ada");
    assert_eq!(sent[0].body, "Hi Grace, loved alpha Inc in Fintech.");
    assert_eq!(sent[0].attachments.len(), 1);
    assert_eq!(sent[1].body, "Hi there, loved delta Inc in Fintech.");

    assert_eq!(
        repository.statuses(),
        vec![
            ("beta".to_string(), ApplicationStatus::Applied),
            ("alpha".to_string(), ApplicationStatus::Applied),
            ("gamma".to_string(), ApplicationStatus::Skipped),
            ("delta".to_string(), ApplicationStatus::Applied),
        ]
    );
}

#[tokio::test]
async fn dry_run_previews_without_sending_or_recording() {
    let (directory, founders, contacts) = scenario();
    let repository = MemoryRepository::default();
    let mailer = RecordingMailer::default();
    let use_case = HuntUseCase::new(
        directory,
        founders,
        contacts,
        repository.clone(),
        mailer.clone(),
        CountingDelay::default(),
        profile(false, 10),
    );
    let progress = CollectingProgress::default();

    let report = use_case
        .execute(&criteria(true, Some(2)), &progress)
        .await
        .unwrap();

    assert_eq!(report.previewed, 1);
    assert_eq!(report.no_contact, 1);
    assert!(mailer.sent.lock().unwrap().is_empty());
    assert!(repository.statuses().is_empty());
    let events = progress.0.lock().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].outcome, HuntOutcome::Previewed);
    assert_eq!(events[1].outcome, HuntOutcome::NoContact);
    assert_eq!(
        events[0].email.as_ref().map(|email| email.to.as_str()),
        Some("grace@alpha.com")
    );
}

#[tokio::test]
async fn missing_contact_is_recorded_and_failed_send_is_not() {
    let (directory, founders, contacts) = scenario();
    let repository = MemoryRepository::default();
    let mailer = RecordingMailer {
        failing_recipient: Some("hello@delta.com".into()),
        ..RecordingMailer::default()
    };
    let use_case = HuntUseCase::new(
        directory,
        founders,
        contacts,
        repository.clone(),
        mailer,
        CountingDelay::default(),
        profile(true, 10),
    );

    let report = use_case
        .execute(&criteria(false, None), &CollectingProgress::default())
        .await
        .unwrap();

    assert_eq!(report.no_contact, 1);
    assert_eq!(report.send_failed, 1);
    let statuses = repository.statuses();
    assert!(statuses.contains(&("beta".to_string(), ApplicationStatus::FailedNoContact)));
    assert!(!statuses.iter().any(|(slug, _)| slug == "delta"));
}

#[tokio::test]
async fn daily_limit_stops_sending() {
    let (directory, founders, contacts) = scenario();
    let mailer = RecordingMailer::default();
    let use_case = HuntUseCase::new(
        directory,
        founders,
        contacts,
        MemoryRepository::default(),
        mailer.clone(),
        CountingDelay::default(),
        profile(true, 1),
    );

    let report = use_case
        .execute(&criteria(false, None), &CollectingProgress::default())
        .await
        .unwrap();

    assert_eq!(report.applied, 1);
    assert!(report.daily_limit_reached);
    assert_eq!(mailer.sent.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn real_run_requires_resume() {
    let (directory, founders, contacts) = scenario();
    let use_case = HuntUseCase::new(
        directory,
        founders,
        contacts,
        MemoryRepository::default(),
        RecordingMailer::default(),
        CountingDelay::default(),
        profile(false, 10),
    );

    let result = use_case
        .execute(&criteria(false, None), &CollectingProgress::default())
        .await;

    assert!(matches!(result, Err(DomainError::InvalidInput(_))));
}

#[tokio::test]
async fn status_summary_and_reset() {
    let repository = MemoryRepository::seeded("alpha");
    repository
        .record(NewApplication::for_startup(
            &startup("beta", None),
            None,
            ApplicationStatus::Skipped,
        ))
        .await
        .unwrap();

    let summary = StatusUseCase::new(repository.clone())
        .summary()
        .await
        .unwrap();
    assert_eq!(summary.total, 2);
    assert_eq!(summary.applied, 1);
    assert_eq!(summary.skipped, 1);

    let reset = ResetUseCase::new(repository.clone());
    assert_eq!(reset.execute(Some("beta".into())).await.unwrap(), 1);
    assert_eq!(reset.execute(None).await.unwrap(), 1);
    assert!(repository.statuses().is_empty());
}

#[test]
fn team_size_and_status_parsing() {
    assert_eq!(
        "1-10".parse::<TeamSizeRange>().unwrap(),
        TeamSizeRange { min: 1, max: 10 }
    );
    assert_eq!(
        "5".parse::<TeamSizeRange>().unwrap(),
        TeamSizeRange { min: 5, max: 5 }
    );
    assert!("10-1".parse::<TeamSizeRange>().is_err());
    assert_eq!(
        "failed_no_contact".parse::<ApplicationStatus>().unwrap(),
        ApplicationStatus::FailedNoContact
    );
}
