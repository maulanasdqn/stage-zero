use std::time::Duration;

use stagezero_core::usecases::{HuntUseCase, OutreachProfile, ResetUseCase, StatusUseCase};

use crate::config::StageZeroConfig;
use crate::delay::RandomDelay;
use crate::errors::InfraError;
use crate::http::build_http_client;
use crate::resend_mailer::ResendMailer;
use crate::sqlite_repo::SqliteApplicationRepository;
use crate::website_contacts::WebsiteContactFinder;
use crate::yc_scraper::{YcFounderSource, YcOssDirectory};

pub type LiveHuntUseCase = HuntUseCase<
    YcOssDirectory,
    YcFounderSource,
    WebsiteContactFinder,
    SqliteApplicationRepository,
    ResendMailer,
    RandomDelay,
>;

pub type LiveStatusUseCase = StatusUseCase<SqliteApplicationRepository>;

pub type LiveResetUseCase = ResetUseCase<SqliteApplicationRepository>;

pub struct LiveServices {
    pub hunt: LiveHuntUseCase,
    pub status: LiveStatusUseCase,
    pub reset: LiveResetUseCase,
    pub mailer_configured: bool,
    pub resume_loaded: bool,
}

pub fn build_live_services(config: &StageZeroConfig) -> Result<LiveServices, InfraError> {
    let client = build_http_client()?;
    let repository = SqliteApplicationRepository::open(&config.storage.database_path)?;
    let mailer = ResendMailer::new(
        client.clone(),
        config.resend_api_key(),
        config.resend.from.clone(),
    );
    let mailer_configured = mailer.is_configured();
    let resume = config.load_resume()?;
    let resume_loaded = resume.is_some();
    let delay = RandomDelay::new(
        Duration::from_secs(config.outreach.min_delay_secs),
        Duration::from_secs(config.outreach.max_delay_secs),
    );
    let profile = OutreachProfile {
        applicant: config.applicant(),
        template: config.template.clone(),
        resume,
        daily_limit: config.outreach.daily_limit,
    };

    Ok(LiveServices {
        hunt: HuntUseCase::new(
            YcOssDirectory::new(client.clone()),
            YcFounderSource::new(client.clone()),
            WebsiteContactFinder::new(client),
            repository.clone(),
            mailer,
            delay,
            profile,
        ),
        status: StatusUseCase::new(repository.clone()),
        reset: ResetUseCase::new(repository),
        mailer_configured,
        resume_loaded,
    })
}

pub fn build_repository_services(
    database_path: &std::path::Path,
) -> Result<(LiveStatusUseCase, LiveResetUseCase), InfraError> {
    let repository = SqliteApplicationRepository::open(database_path)?;
    Ok((
        StatusUseCase::new(repository.clone()),
        ResetUseCase::new(repository),
    ))
}
