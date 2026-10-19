use std::collections::BTreeSet;
use std::sync::LazyLock;

use futures::future::join_all;
use regex::Regex;
use reqwest::header::{ACCEPT, USER_AGENT};
use reqwest::{Client, Url};
use scraper::{Html, Selector};
use stagezero_core::DomainError;
use stagezero_core::entities::{Contact, ContactOrigin, Founder, Startup};
use stagezero_core::traits::ContactFinder;

use crate::http::random_user_agent;

const CANDIDATE_PATHS: [&str; 6] = ["", "/about", "/team", "/contact", "/about-us", "/company"];

const IGNORED_SUFFIXES: [&str; 9] = [
    ".png", ".jpg", ".jpeg", ".gif", ".svg", ".webp", ".avif", ".css", ".js",
];

const IGNORED_FRAGMENTS: [&str; 10] = [
    "noreply",
    "no-reply",
    "donotreply",
    "example.com",
    "sentry",
    "wixpress",
    "@2x",
    "domain.com",
    "email.com",
    "yourname",
];

static EMAIL_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[A-Za-z0-9][A-Za-z0-9._%+-]*@[A-Za-z0-9-]+(?:\.[A-Za-z0-9-]+)*\.[A-Za-z]{2,24}")
        .expect("email pattern is valid")
});

static MAILTO_SELECTOR: LazyLock<Selector> = LazyLock::new(|| {
    Selector::parse(r#"a[href^="mailto:"], a[href^="MAILTO:"]"#).expect("mailto selector is valid")
});

static CLOUDFLARE_EMAIL_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("[data-cfemail]").expect("cloudflare selector is valid"));

#[derive(Clone)]
pub struct WebsiteContactFinder {
    client: Client,
}

struct FetchedPage {
    final_host: Option<String>,
    html: String,
}

impl WebsiteContactFinder {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    async fn fetch_page(&self, url: Url) -> Option<FetchedPage> {
        let response = self
            .client
            .get(url)
            .header(USER_AGENT, random_user_agent())
            .header(ACCEPT, "text/html,application/xhtml+xml")
            .send()
            .await
            .ok()?
            .error_for_status()
            .ok()?;
        let final_host = response.url().host_str().map(bare_host);
        let html = response.text().await.ok()?;
        Some(FetchedPage { final_host, html })
    }
}

impl ContactFinder for WebsiteContactFinder {
    async fn find(
        &self,
        startup: &Startup,
        founders: &[Founder],
    ) -> Result<Option<Contact>, DomainError> {
        let Some(base_url) = startup.website.as_deref().and_then(parse_website) else {
            return Ok(None);
        };
        let Some(domain) = base_url.host_str().map(bare_host) else {
            return Ok(None);
        };

        let urls = CANDIDATE_PATHS
            .iter()
            .filter_map(|path| match *path {
                "" => Some(base_url.clone()),
                path => base_url.join(path).ok(),
            })
            .collect::<Vec<_>>();
        let pages = join_all(urls.into_iter().map(|url| self.fetch_page(url)))
            .await
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        if pages.is_empty() {
            return Err(DomainError::Scraping(format!(
                "{} is unreachable",
                base_url
            )));
        }

        let mut allowed_domains = BTreeSet::from([domain.clone()]);
        allowed_domains.extend(pages.iter().filter_map(|page| page.final_host.clone()));

        let emails = pages
            .iter()
            .flat_map(|page| extract_emails(&page.html))
            .collect::<BTreeSet<_>>();

        let scraped = choose_contact(&emails, &allowed_domains, founders);
        Ok(scraped.or_else(|| guess_contact(&domain, founders)))
    }
}

fn parse_website(website: &str) -> Option<Url> {
    let trimmed = website.trim();
    let with_scheme = match trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        true => trimmed.to_string(),
        false => format!("https://{trimmed}"),
    };
    Url::parse(&with_scheme).ok()
}

fn bare_host(host: &str) -> String {
    host.trim_start_matches("www.").to_lowercase()
}

pub fn extract_emails(html: &str) -> BTreeSet<String> {
    let document = Html::parse_document(html);

    let from_mailto = document
        .select(&MAILTO_SELECTOR)
        .filter_map(|anchor| anchor.value().attr("href"))
        .map(|href| {
            href[7..]
                .split('?')
                .next()
                .unwrap_or_default()
                .replace("%40", "@")
                .to_string()
        });

    let from_cloudflare = document
        .select(&CLOUDFLARE_EMAIL_SELECTOR)
        .filter_map(|element| element.value().attr("data-cfemail"))
        .filter_map(decode_cloudflare_email);

    let from_text = EMAIL_PATTERN
        .find_iter(html)
        .map(|found| found.as_str().to_string());

    from_mailto
        .chain(from_cloudflare)
        .chain(from_text)
        .flat_map(|candidate| {
            EMAIL_PATTERN
                .find_iter(&candidate)
                .map(|found| found.as_str().to_lowercase())
                .collect::<Vec<_>>()
        })
        .map(|email| email.trim_end_matches('.').to_string())
        .filter(|email| is_plausible_email(email))
        .collect()
}

pub fn decode_cloudflare_email(encoded: &str) -> Option<String> {
    let bytes = (0..encoded.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(encoded.get(index..index + 2)?, 16).ok())
        .collect::<Option<Vec<u8>>>()?;
    let (key, payload) = bytes.split_first()?;
    String::from_utf8(payload.iter().map(|byte| byte ^ key).collect()).ok()
}

fn is_plausible_email(email: &str) -> bool {
    !IGNORED_SUFFIXES
        .iter()
        .any(|suffix| email.ends_with(suffix))
        && !IGNORED_FRAGMENTS
            .iter()
            .any(|fragment| email.contains(fragment))
        && email.len() <= 254
}

fn email_domain_allowed(email: &str, allowed_domains: &BTreeSet<String>) -> bool {
    let Some((_, email_domain)) = email.rsplit_once('@') else {
        return false;
    };
    allowed_domains
        .iter()
        .any(|domain| email_domain == domain || email_domain.ends_with(&format!(".{domain}")))
}

fn normalized_name(name: &str) -> String {
    name.chars()
        .filter(char::is_ascii_alphanumeric)
        .collect::<String>()
        .to_lowercase()
}

fn matching_founder<'a>(local_part: &str, founders: &'a [Founder]) -> Option<&'a Founder> {
    let compact_local = normalized_name(local_part);
    founders.iter().find(|founder| {
        let first = normalized_name(founder.first_name());
        let last = founder.last_name().map(normalized_name).unwrap_or_default();
        let first_initial = first.chars().next().map(String::from).unwrap_or_default();
        let last_initial = last.chars().next().map(String::from).unwrap_or_default();
        let candidates = [
            first.clone(),
            last.clone(),
            format!("{first}{last}"),
            format!("{first_initial}{last}"),
            format!("{first}{last_initial}"),
        ];
        candidates
            .iter()
            .any(|candidate| candidate.len() >= 2 && *candidate == compact_local)
    })
}

const UNSUITABLE_ROLE_SCORE: u32 = 5;

fn role_score(local_part: &str) -> u32 {
    match local_part {
        "founder" | "founders" | "ceo" | "cto" | "cofounder" | "co-founder" => 80,
        "hello" | "hi" | "hey" | "team" => 60,
        "jobs" | "careers" | "hiring" | "work" => 55,
        "contact" => 50,
        "info" | "sales" => 30,
        "support" | "help" | "privacy" | "legal" | "security" | "press" | "abuse" | "billing"
        | "dpo" | "compliance" | "media" | "marketing" => UNSUITABLE_ROLE_SCORE,
        _ => 40,
    }
}

pub fn choose_contact(
    emails: &BTreeSet<String>,
    allowed_domains: &BTreeSet<String>,
    founders: &[Founder],
) -> Option<Contact> {
    emails
        .iter()
        .filter(|email| email_domain_allowed(email, allowed_domains))
        .filter_map(|email| {
            let (local_part, _) = email.split_once('@')?;
            let founder = matching_founder(local_part, founders);
            let score = match founder {
                Some(_) => 100,
                None => role_score(local_part),
            };
            Some((score, email, founder))
        })
        .filter(|(score, _, _)| *score > UNSUITABLE_ROLE_SCORE)
        .max_by(|left, right| left.0.cmp(&right.0).then_with(|| right.1.cmp(left.1)))
        .map(|(_, email, founder)| Contact {
            email: email.clone(),
            founder: founder.cloned(),
            origin: ContactOrigin::Scraped,
        })
}

pub fn guess_contact(domain: &str, founders: &[Founder]) -> Option<Contact> {
    founders.iter().find_map(|founder| {
        let first_name = normalized_name(founder.first_name());
        (first_name.len() >= 2).then(|| Contact {
            email: format!("{first_name}@{domain}"),
            founder: Some(founder.clone()),
            origin: ContactOrigin::Guessed,
        })
    })
}
