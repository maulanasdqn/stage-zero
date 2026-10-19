use std::collections::BTreeSet;

use stagezero_core::entities::{
    ApplicationStatus, Attachment, ContactOrigin, Founder, HuntCriteria, NewApplication,
    OutgoingEmail, Startup, TeamSizeRange,
};
use stagezero_core::traits::ApplicationRepository;
use stagezero_infra::cloudflare_mailer::{CloudflareEmailRequest, interpret_response};
use stagezero_infra::config::EmailProvider;
use stagezero_infra::config::{EXAMPLE_CONFIG, StageZeroConfig};
use stagezero_infra::sqlite_repo::SqliteApplicationRepository;
use stagezero_infra::website_contacts::{
    choose_contact, decode_cloudflare_email, extract_emails, guess_contact,
};
use stagezero_infra::yc_scraper::{batch_slug, extract_founders, filter_companies, parse_batches};

const COMPANIES_JSON: &str = include_str!("fixtures/batch.json");
const COMPANY_PAGE_HTML: &str = include_str!("fixtures/company_page.html");
const WEBSITE_HTML: &str = include_str!("fixtures/website.html");

fn criteria(industry: Option<&str>) -> HuntCriteria {
    HuntCriteria {
        batch: "W24".into(),
        team_size: TeamSizeRange { min: 1, max: 10 },
        industry: industry.map(Into::into),
        limit: None,
        dry_run: true,
    }
}

fn founders() -> Vec<Founder> {
    vec![
        Founder {
            full_name: "Grace Hopper".into(),
            title: Some("CEO".into()),
        },
        Founder {
            full_name: "Alan Turing".into(),
            title: Some("CTO".into()),
        },
    ]
}

#[test]
fn batch_codes_map_to_yc_oss_slugs() {
    assert_eq!(batch_slug("W24").unwrap(), "winter-2024");
    assert_eq!(batch_slug("s24").unwrap(), "summer-2024");
    assert_eq!(batch_slug("F24").unwrap(), "fall-2024");
    assert_eq!(batch_slug("X25").unwrap(), "spring-2025");
    assert_eq!(batch_slug("SP25").unwrap(), "spring-2025");
    assert_eq!(batch_slug("Winter 2026").unwrap(), "winter-2026");
    assert_eq!(batch_slug("summer-2023").unwrap(), "summer-2023");
    assert!(batch_slug("Q24").is_err());
    assert!(batch_slug("W").is_err());
}

#[test]
fn companies_are_filtered_by_size_status_and_industry() {
    let all = filter_companies(COMPANIES_JSON, &criteria(None)).unwrap();
    let slugs = all
        .iter()
        .map(|startup| startup.slug.as_str())
        .collect::<Vec<_>>();
    assert_eq!(slugs, vec!["ledgerly", "shipfast"]);

    let saas = filter_companies(COMPANIES_JSON, &criteria(Some("B2B SaaS"))).unwrap();
    assert_eq!(saas.len(), 1);
    assert_eq!(saas[0].slug, "shipfast");
    assert_eq!(saas[0].industries, vec!["B2B".to_string()]);

    let fintech = filter_companies(COMPANIES_JSON, &criteria(Some("fintech"))).unwrap();
    assert_eq!(fintech[0].slug, "ledgerly");
}

#[test]
fn founders_are_parsed_from_yc_company_page() {
    let parsed = extract_founders(COMPANY_PAGE_HTML).unwrap();
    assert_eq!(parsed, founders());
}

#[test]
fn emails_are_extracted_from_mailto_text_and_cloudflare() {
    let emails = extract_emails(WEBSITE_HTML);
    assert!(emails.contains("grace@ledgerly.ai"));
    assert!(emails.contains("support@ledgerly.ai"));
    assert!(emails.contains("hello@ledgerly.ai"));
    assert!(emails.contains("investor@othercorp.com"));
    assert!(!emails.iter().any(|email| email.ends_with(".png")));
    assert!(!emails.iter().any(|email| email.contains("noreply")));
}

#[test]
fn cloudflare_obfuscation_is_decoded() {
    assert_eq!(
        decode_cloudflare_email("2a424f4646456a464f4e4d4f584653044b43").as_deref(),
        Some("hello@ledgerly.ai")
    );
    assert!(decode_cloudflare_email("zz").is_none());
}

#[test]
fn founder_email_beats_role_and_foreign_domains() {
    let emails = extract_emails(WEBSITE_HTML);
    let allowed = BTreeSet::from(["ledgerly.ai".to_string()]);
    let contact = choose_contact(&emails, &allowed, &founders()).unwrap();
    assert_eq!(contact.email, "grace@ledgerly.ai");
    assert_eq!(contact.founder.unwrap().full_name, "Grace Hopper");
    assert_eq!(contact.origin, ContactOrigin::Scraped);

    let generic = BTreeSet::from([
        "support@ledgerly.ai".to_string(),
        "hello@ledgerly.ai".to_string(),
    ]);
    assert_eq!(
        choose_contact(&generic, &allowed, &[]).unwrap().email,
        "hello@ledgerly.ai"
    );

    let unsuitable = BTreeSet::from([
        "support@ledgerly.ai".to_string(),
        "privacy@ledgerly.ai".to_string(),
    ]);
    assert!(choose_contact(&unsuitable, &allowed, &founders()).is_none());

    let foreign = BTreeSet::from(["investor@othercorp.com".to_string()]);
    assert!(choose_contact(&foreign, &allowed, &founders()).is_none());
}

#[test]
fn guessed_contact_uses_first_founder() {
    let contact = guess_contact("ledgerly.ai", &founders()).unwrap();
    assert_eq!(contact.email, "grace@ledgerly.ai");
    assert_eq!(contact.origin, ContactOrigin::Guessed);
    assert!(guess_contact("ledgerly.ai", &[]).is_none());
}

#[test]
fn example_config_parses() {
    let config = StageZeroConfig::parse(EXAMPLE_CONFIG).unwrap();
    assert_eq!(config.outreach.daily_limit, 20);
    assert!(config.template.body.contains("{{founder_first_name}}"));
}

#[tokio::test]
async fn sqlite_repository_round_trip() {
    let repository = SqliteApplicationRepository::in_memory().unwrap();
    let startup = Startup {
        slug: "ledgerly".into(),
        name: "Ledgerly".into(),
        website: Some("https://ledgerly.ai".into()),
        batch: "Winter 2024".into(),
        team_size: Some(3),
        industries: vec!["Fintech".into()],
        one_liner: String::new(),
        tags: Vec::new(),
        description: String::new(),
    };

    assert!(!repository.exists("ledgerly").await.unwrap());
    repository
        .record(NewApplication::for_startup(
            &startup,
            None,
            ApplicationStatus::FailedNoContact,
        ))
        .await
        .unwrap();
    repository
        .record(NewApplication::for_startup(
            &startup,
            Some("grace@ledgerly.ai".into()),
            ApplicationStatus::Applied,
        ))
        .await
        .unwrap();

    assert!(repository.exists("ledgerly").await.unwrap());
    let rows = repository.list(None).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].status, ApplicationStatus::Applied);
    assert_eq!(rows[0].contact_email.as_deref(), Some("grace@ledgerly.ai"));
    assert_eq!(repository.count_applied_today().await.unwrap(), 1);
    assert!(
        repository
            .list(Some(ApplicationStatus::Skipped))
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(repository.reset(None).await.unwrap(), 1);
    assert!(!repository.exists("ledgerly").await.unwrap());
}

#[test]
fn example_config_defaults_to_cloudflare() {
    let config = StageZeroConfig::parse(EXAMPLE_CONFIG).unwrap();
    assert_eq!(config.email.provider, EmailProvider::Cloudflare);
    assert_eq!(
        config.email.formatted_sender(),
        "Your Name <hello@yourdomain.com>"
    );
}

#[test]
fn cloudflare_request_matches_rest_schema() {
    let email = OutgoingEmail {
        to: "grace@ledgerly.ai".into(),
        reply_to: Some("me@gmail.com".into()),
        subject: "Hello".into(),
        body: "Body".into(),
        attachments: vec![Attachment {
            filename: "Resume.PDF".into(),
            content: std::sync::Arc::from(b"%PDF".as_slice()),
        }],
    };
    let request = CloudflareEmailRequest::from_email("hello@msdqn.dev", Some("Maulana"), &email);
    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["to"], "grace@ledgerly.ai");
    assert_eq!(json["from"]["address"], "hello@msdqn.dev");
    assert_eq!(json["from"]["name"], "Maulana");
    assert_eq!(json["reply_to"], "me@gmail.com");
    assert_eq!(json["text"], "Body");
    assert_eq!(json["attachments"][0]["content"], "JVBERg==");
    assert_eq!(json["attachments"][0]["type"], "application/pdf");
    assert_eq!(json["attachments"][0]["disposition"], "attachment");
}

#[test]
fn cloudflare_responses_are_interpreted() {
    let delivered = r#"{"success":true,"errors":[],"messages":[],"result":{"delivered":["grace@ledgerly.ai"],"permanent_bounces":[],"queued":[]}}"#;
    assert_eq!(
        interpret_response("grace@ledgerly.ai", delivered).unwrap(),
        "delivered"
    );

    let queued = r#"{"success":true,"errors":[],"messages":[],"result":{"delivered":[],"permanent_bounces":[],"queued":["Grace@Ledgerly.ai"]}}"#;
    assert_eq!(
        interpret_response("grace@ledgerly.ai", queued).unwrap(),
        "queued"
    );

    let live = r#"{"success":true,"errors":[],"messages":[],"result":{"message_id":"<abc@msdqn.dev>","delivered":[],"queued":["maulanasdqn@gmail.com"],"permanent_bounces":[],"suppressed_recipients":[]}}"#;
    assert_eq!(
        interpret_response("maulanasdqn@gmail.com", live).unwrap(),
        "queued <abc@msdqn.dev>"
    );

    let suppressed = r#"{"success":true,"errors":[],"messages":[],"result":{"delivered":[],"queued":[],"permanent_bounces":[],"suppressed_recipients":["grace@ledgerly.ai"]}}"#;
    assert!(interpret_response("grace@ledgerly.ai", suppressed).is_err());

    let bounced = r#"{"success":true,"errors":[],"messages":[],"result":{"delivered":[],"permanent_bounces":["grace@ledgerly.ai"],"queued":[]}}"#;
    assert!(interpret_response("grace@ledgerly.ai", bounced).is_err());

    let rejected = r#"{"success":false,"errors":[{"code":10203,"message":"email.sending.error.email.sending_disabled"}],"messages":[],"result":null}"#;
    let error = interpret_response("grace@ledgerly.ai", rejected).unwrap_err();
    assert!(error.to_string().contains("sending_disabled"));
}

#[test]
fn batches_are_sorted_newest_first() {
    let raw = r#"{"batches":{
        "winter-2024":{"name":"Winter 2024","count":248},
        "fall-2026":{"name":"Fall 2026","count":147},
        "summer-2024":{"name":"Summer 2024","count":248},
        "spring-2026":{"name":"Spring 2026","count":192},
        "unspecified":{"name":"Unspecified","count":1}
    }}"#;
    let slugs = parse_batches(raw)
        .unwrap()
        .into_iter()
        .map(|batch| batch.slug)
        .collect::<Vec<_>>();
    assert_eq!(
        slugs,
        vec!["fall-2026", "spring-2026", "summer-2024", "winter-2024"]
    );
}
