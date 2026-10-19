use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use stagezero_core::DomainError;
use stagezero_core::entities::OutgoingEmail;
use stagezero_core::traits::Mailer;

pub const RESEND_EMAILS_ENDPOINT: &str = "https://api.resend.com/emails";

#[derive(Clone)]
pub struct ResendMailer {
    client: Client,
    api_key: Option<String>,
    from: String,
    endpoint: String,
}

#[derive(Debug, Serialize)]
pub struct ResendAttachment {
    pub filename: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ResendEmailRequest {
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<ResendAttachment>,
}

#[derive(Debug, Deserialize)]
struct ResendEmailResponse {
    id: String,
}

impl ResendEmailRequest {
    pub fn from_email(from: &str, email: &OutgoingEmail) -> Self {
        Self {
            from: from.to_string(),
            to: vec![email.to.clone()],
            subject: email.subject.clone(),
            text: email.body.clone(),
            reply_to: email.reply_to.clone(),
            attachments: email
                .attachments
                .iter()
                .map(|attachment| ResendAttachment {
                    filename: attachment.filename.clone(),
                    content: STANDARD.encode(&attachment.content),
                })
                .collect(),
        }
    }
}

impl ResendMailer {
    pub fn new(client: Client, api_key: Option<String>, from: impl Into<String>) -> Self {
        Self {
            client,
            api_key,
            from: from.into(),
            endpoint: RESEND_EMAILS_ENDPOINT.to_string(),
        }
    }

    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }

    pub fn is_configured(&self) -> bool {
        self.api_key.is_some()
    }
}

impl Mailer for ResendMailer {
    async fn send(&self, email: &OutgoingEmail) -> Result<String, DomainError> {
        let api_key = self
            .api_key
            .as_deref()
            .ok_or_else(|| DomainError::Mailer("Resend API key is not configured".into()))?;
        let response = self
            .client
            .post(&self.endpoint)
            .bearer_auth(api_key)
            .json(&ResendEmailRequest::from_email(&self.from, email))
            .send()
            .await
            .map_err(|error| DomainError::Mailer(error.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(DomainError::Mailer(format!(
                "Resend responded {status}: {body}"
            )));
        }

        response
            .json::<ResendEmailResponse>()
            .await
            .map(|parsed| parsed.id)
            .map_err(|error| DomainError::Mailer(format!("unexpected Resend response: {error}")))
    }
}
