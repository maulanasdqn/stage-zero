use stagezero_core::DomainError;
use stagezero_core::entities::OutgoingEmail;
use stagezero_core::traits::Mailer;

use crate::cloudflare_mailer::CloudflareMailer;
use crate::resend_mailer::ResendMailer;

#[derive(Clone)]
pub enum ConfiguredMailer {
    Cloudflare(CloudflareMailer),
    Resend(ResendMailer),
}

impl ConfiguredMailer {
    pub fn provider_name(&self) -> &'static str {
        match self {
            ConfiguredMailer::Cloudflare(_) => "Cloudflare Email",
            ConfiguredMailer::Resend(_) => "Resend",
        }
    }

    pub fn is_configured(&self) -> bool {
        match self {
            ConfiguredMailer::Cloudflare(mailer) => mailer.is_configured(),
            ConfiguredMailer::Resend(mailer) => mailer.is_configured(),
        }
    }
}

impl Mailer for ConfiguredMailer {
    async fn send(&self, email: &OutgoingEmail) -> Result<String, DomainError> {
        match self {
            ConfiguredMailer::Cloudflare(mailer) => mailer.send(email).await,
            ConfiguredMailer::Resend(mailer) => mailer.send(email).await,
        }
    }
}
