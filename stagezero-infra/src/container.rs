use std::time::Duration;

use stagezero_core::usecases::{HuntUseCase, OutreachProfile, ResetUseCase, StatusUseCase};

use crate::cloudflare_mailer::{CloudflareCredentials, CloudflareMailer};
use crate::config::{EmailProvider, StageZeroConfig};
use crate::delay::RandomDelay;
use crate::errors::InfraError;
use crate::http::build_http_client;
use crate::mailer::ConfiguredMailer;
use crate::resend_mailer::ResendMailer;
use crate::sqlite_repo::SqliteApplicationRepository;
use crate::website_contacts::WebsiteContactFinder;
use crate::yc_scraper::{YcFounderSource, YcOssDirectory};

pub type LiveHuntUseCase = HuntUseCase<
    YcOssDirectory,
    YcFounderSource,
    WebsiteContactFinder,
    SqliteApplicationRepository,
    ConfiguredMailer,
    RandomDelay,
>;

pub type LiveStatusUseCase = StatusUseCase<SqliteApplicationRepository>;

pub type LiveResetUseCase = ResetUseCase<SqliteApplicationRepository>;

pub struct LiveServices {
    pub hunt: LiveHuntUseCase,
    pub status: LiveStatusUseCase,
    pub reset: LiveResetUseCase,
    pub mailer_configured: bool,
    pub mailer_provider: &'static str,
    pub sender: String,
    pub resume_loaded: bool,
}

pub fn build_live_services(config: &StageZeroConfig) -> Result<LiveServices, InfraError> {
    let client = build_http_client()?;
    let repository = SqliteApplicationRepository::open(&config.storage.database_path)?;
    let mailer = build_mailer(&client, config);
    let mailer_configured = mailer.is_configured();
    let mailer_provider_name = mailer.provider_name();
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
        mailer_provider: mailer_provider_name,
        sender: config.email.formatted_sender(),
        resume_loaded,
    })
}

pub fn build_mailer(client: &reqwest::Client, config: &StageZeroConfig) -> ConfiguredMailer {
    let email = &config.email;
    match email.provider {
        EmailProvider::Cloudflare => {
            let credentials = email.cloudflare_account_id().zip(email.api_token()).map(
                |(account_id, api_token)| CloudflareCredentials {
                    account_id,
                    api_token,
                },
            );
            ConfiguredMailer::Cloudflare(CloudflareMailer::new(
                client.clone(),
                credentials,
                email.from.clone(),
                email.from_name.clone(),
            ))
        }
        EmailProvider::Resend => ConfiguredMailer::Resend(ResendMailer::new(
            client.clone(),
            email.api_token(),
            email.formatted_sender(),
        )),
    }
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
