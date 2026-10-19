use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use stagezero_core::DomainError;
use stagezero_core::entities::{Attachment, OutgoingEmail};
use stagezero_core::traits::Mailer;

pub const CLOUDFLARE_API_BASE_URL: &str = "https://api.cloudflare.com/client/v4";

#[derive(Debug, Clone)]
pub struct CloudflareCredentials {
    pub account_id: String,
    pub api_token: String,
}

#[derive(Clone)]
pub struct CloudflareMailer {
    client: Client,
    credentials: Option<CloudflareCredentials>,
    from_address: String,
    from_name: Option<String>,
    base_url: String,
}

#[derive(Debug, Serialize)]
pub struct CloudflareSender {
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CloudflareAttachment {
    pub content: String,
    pub filename: String,
    #[serde(rename = "type")]
    pub content_type: String,
    pub disposition: String,
}

#[derive(Debug, Serialize)]
pub struct CloudflareEmailRequest {
    pub to: String,
    pub from: CloudflareSender,
    pub subject: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<CloudflareAttachment>,
}

#[derive(Debug, Deserialize)]
struct CloudflareApiError {
    code: i64,
    message: String,
}

#[derive(Debug, Default, Deserialize)]
struct CloudflareDeliveryResult {
    #[serde(default)]
    delivered: Vec<String>,
    #[serde(default)]
    permanent_bounces: Vec<String>,
    #[serde(default)]
    queued: Vec<String>,
    #[serde(default)]
    suppressed_recipients: Vec<String>,
    #[serde(default)]
    message_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CloudflareEnvelope {
    success: bool,
    #[serde(default)]
    errors: Vec<CloudflareApiError>,
    #[serde(default)]
    result: Option<CloudflareDeliveryResult>,
}

fn content_type_for(attachment: &Attachment) -> &'static str {
    let lowercase = attachment.filename.to_lowercase();
    match lowercase.rsplit_once('.').map(|(_, extension)| extension) {
        Some("pdf") => "application/pdf",
        Some("doc") => "application/msword",
        Some("docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        Some("txt") => "text/plain",
        _ => "application/octet-stream",
    }
}

impl CloudflareEmailRequest {
    pub fn from_email(from_address: &str, from_name: Option<&str>, email: &OutgoingEmail) -> Self {
        Self {
            to: email.to.clone(),
            from: CloudflareSender {
                address: from_address.to_string(),
                name: from_name.map(str::to_string),
            },
            subject: email.subject.clone(),
            text: email.body.clone(),
            reply_to: email.reply_to.clone(),
            attachments: email
                .attachments
                .iter()
                .map(|attachment| CloudflareAttachment {
                    content: STANDARD.encode(&attachment.content),
                    filename: attachment.filename.clone(),
                    content_type: content_type_for(attachment).to_string(),
                    disposition: "attachment".to_string(),
                })
                .collect(),
        }
    }
}

pub fn interpret_response(recipient: &str, raw: &str) -> Result<String, DomainError> {
    let envelope: CloudflareEnvelope = serde_json::from_str(raw).map_err(|error| {
        DomainError::Mailer(format!("unexpected Cloudflare response ({error}): {raw}"))
    })?;
    if !envelope.success {
        let reasons = envelope
            .errors
            .iter()
            .map(|error| format!("{} {}", error.code, error.message))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(DomainError::Mailer(format!(
            "Cloudflare rejected the email: {reasons}"
        )));
    }
    let result = envelope.result.unwrap_or_default();
    let matches = |addresses: &[String]| {
        addresses
            .iter()
            .any(|address| address.eq_ignore_ascii_case(recipient))
    };
    if matches(&result.permanent_bounces) {
        return Err(DomainError::Mailer(format!(
            "{recipient} permanently bounced"
        )));
    }
    if matches(&result.suppressed_recipients) {
        return Err(DomainError::Mailer(format!(
            "{recipient} is on the Cloudflare suppression list"
        )));
    }
    let state = match (matches(&result.delivered), matches(&result.queued)) {
        (true, _) => "delivered",
        (_, true) => "queued",
        _ => "accepted",
    };
    Ok(match result.message_id {
        Some(message_id) => format!("{state} {message_id}"),
        None => state.to_string(),
    })
}

impl CloudflareMailer {
    pub fn new(
        client: Client,
        credentials: Option<CloudflareCredentials>,
        from_address: impl Into<String>,
        from_name: Option<String>,
    ) -> Self {
        Self {
            client,
            credentials,
            from_address: from_address.into(),
            from_name,
            base_url: CLOUDFLARE_API_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into().trim_end_matches('/').to_string();
        self
    }

    pub fn is_configured(&self) -> bool {
        self.credentials.is_some()
    }
}

impl Mailer for CloudflareMailer {
    async fn send(&self, email: &OutgoingEmail) -> Result<String, DomainError> {
        let credentials = self.credentials.as_ref().ok_or_else(|| {
            DomainError::Mailer("Cloudflare account id or API token is not configured".into())
        })?;
        let url = format!(
            "{}/accounts/{}/email/sending/send",
            self.base_url, credentials.account_id
        );
        let request = CloudflareEmailRequest::from_email(
            &self.from_address,
            self.from_name.as_deref(),
            email,
        );
        let response = self
            .client
            .post(&url)
            .bearer_auth(&credentials.api_token)
            .json(&request)
            .send()
            .await
            .map_err(|error| DomainError::Mailer(error.to_string()))?;
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| DomainError::Mailer(error.to_string()))?;
        interpret_response(&email.to, &body).map_err(|error| match status.is_success() {
            true => error,
            false => DomainError::Mailer(format!("HTTP {status}: {error}")),
        })
    }
}
