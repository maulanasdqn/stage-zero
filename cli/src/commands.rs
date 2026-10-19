use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use clap::{Args, Parser, Subcommand};
use stagezero_core::entities::{
    Application, ApplicationStatus, ContactOrigin, HuntCriteria, HuntEvent, HuntOutcome,
    HuntReport, TeamSizeRange,
};
use stagezero_core::traits::HuntProgress;
use stagezero_infra::config::{
    DEFAULT_CONFIG_PATH, EXAMPLE_CONFIG, StageZeroConfig, load_storage_config,
};
use stagezero_infra::container::{build_live_services, build_repository_services};

#[derive(Debug, Parser)]
#[command(
    name = "stagezero",
    version,
    about = "Discover early-stage YC startups and send founders a personalized application"
)]
pub struct Cli {
    #[arg(long, short, global = true, default_value = DEFAULT_CONFIG_PATH, help = "Path to stagezero.toml")]
    pub config: PathBuf,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "Create a starter stagezero.toml")]
    Init(InitArgs),
    #[command(about = "Find startups, extract founder contacts and send applications")]
    Hunt(HuntArgs),
    #[command(about = "Show tracked applications")]
    Status(StatusArgs),
    #[command(about = "Delete tracked applications so startups can be contacted again")]
    Reset(ResetArgs),
}

#[derive(Debug, Args)]
pub struct InitArgs {
    #[arg(long, help = "Overwrite an existing config file")]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct HuntArgs {
    #[arg(
        long,
        short,
        help = "YC batch such as W24, S24, F24, X25 or winter-2024"
    )]
    pub batch: String,
    #[arg(
        long,
        short,
        default_value = "1-10",
        help = "Team size range, e.g. 1-10"
    )]
    pub size: TeamSizeRange,
    #[arg(long, short, help = "Industry keywords, e.g. \"B2B SaaS\"")]
    pub industry: Option<String>,
    #[arg(long, short, help = "Maximum number of new startups to process")]
    pub limit: Option<usize>,
    #[arg(long, help = "Preview emails without sending or recording anything")]
    pub dry_run: bool,
    #[arg(long, short, help = "Skip the confirmation prompt before sending")]
    pub yes: bool,
}

#[derive(Debug, Args)]
pub struct StatusArgs {
    #[arg(
        long,
        short,
        help = "Filter by status: APPLIED, SKIPPED or FAILED_NO_CONTACT"
    )]
    pub status: Option<ApplicationStatus>,
}

#[derive(Debug, Args)]
pub struct ResetArgs {
    #[arg(long, help = "Only reset a single YC slug")]
    pub slug: Option<String>,
    #[arg(long, short, help = "Skip the confirmation prompt")]
    pub yes: bool,
}

pub async fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::Init(args) => init(&cli.config, args),
        Command::Hunt(args) => hunt(&cli.config, args).await,
        Command::Status(args) => status(&cli.config, args).await,
        Command::Reset(args) => reset(&cli.config, args).await,
    }
}

fn init(config_path: &Path, args: InitArgs) -> anyhow::Result<()> {
    if config_path.exists() && !args.force {
        bail!(
            "{} already exists, pass --force to overwrite it",
            config_path.display()
        );
    }
    std::fs::write(config_path, EXAMPLE_CONFIG)
        .with_context(|| format!("cannot write {}", config_path.display()))?;
    println!(
        "Created {}. Fill in your details, resume path and Resend sender.",
        config_path.display()
    );
    println!("Set RESEND_API_KEY in your environment or in the [resend] section.");
    Ok(())
}

async fn hunt(config_path: &Path, args: HuntArgs) -> anyhow::Result<()> {
    let config = StageZeroConfig::load(config_path)?;
    let services = build_live_services(&config)?;

    if !args.dry_run {
        if !services.mailer_configured {
            bail!(
                "Resend API key missing: set RESEND_API_KEY or [resend].api_key, or use --dry-run"
            );
        }
        if !services.resume_loaded {
            bail!(
                "resume not found at {}, fix [applicant].resume_path or use --dry-run",
                config.applicant.resume_path.display()
            );
        }
        if !args.yes
            && !confirm(&format!(
                "Send real emails to founders of {} startups (team size {}){}?",
                args.batch,
                args.size,
                args.industry
                    .as_deref()
                    .map(|industry| format!(", industry \"{industry}\""))
                    .unwrap_or_default()
            ))?
        {
            println!("Aborted.");
            return Ok(());
        }
    }

    let criteria = HuntCriteria {
        batch: args.batch,
        team_size: args.size,
        industry: args.industry,
        limit: args.limit,
        dry_run: args.dry_run,
    };
    let report = services.hunt.execute(&criteria, &TerminalProgress).await?;
    print_report(&report, criteria.dry_run);
    Ok(())
}

async fn status(config_path: &Path, args: StatusArgs) -> anyhow::Result<()> {
    let storage = load_storage_config(config_path)?;
    let (status, _) = build_repository_services(&storage.database_path)?;
    let applications = status.list(args.status).await?;
    let summary = status.summary().await?;

    if applications.is_empty() {
        println!("No applications tracked yet.");
    } else {
        print_applications(&applications);
    }
    println!();
    println!(
        "Total {} | APPLIED {} | SKIPPED {} | FAILED_NO_CONTACT {}",
        summary.total, summary.applied, summary.skipped, summary.failed_no_contact
    );
    Ok(())
}

async fn reset(config_path: &Path, args: ResetArgs) -> anyhow::Result<()> {
    let storage = load_storage_config(config_path)?;
    let (_, reset) = build_repository_services(&storage.database_path)?;
    let scope = args
        .slug
        .as_deref()
        .map(|slug| format!("the record for `{slug}`"))
        .unwrap_or_else(|| "ALL tracked applications".to_string());
    if !args.yes && !confirm(&format!("Delete {scope}?"))? {
        println!("Aborted.");
        return Ok(());
    }
    let removed = reset.execute(args.slug).await?;
    println!("Removed {removed} record(s).");
    Ok(())
}

fn confirm(question: &str) -> anyhow::Result<bool> {
    print!("{question} [y/N] ");
    std::io::stdout().flush()?;
    let mut answer = String::new();
    std::io::stdin().lock().read_line(&mut answer)?;
    Ok(matches!(answer.trim().to_lowercase().as_str(), "y" | "yes"))
}

struct TerminalProgress;

impl HuntProgress for TerminalProgress {
    fn on_discovered(&self, count: usize) {
        println!("Found {count} matching startups.");
    }

    fn on_event(&self, event: &HuntEvent) {
        let contact = event
            .contact
            .as_ref()
            .map(|contact| {
                let origin = match contact.origin {
                    ContactOrigin::Scraped => "scraped",
                    ContactOrigin::Guessed => "guessed",
                };
                format!(" -> {} ({origin})", contact.email)
            })
            .unwrap_or_default();
        let label = match &event.outcome {
            HuntOutcome::AlreadyProcessed => "SKIP   already processed".to_string(),
            HuntOutcome::SkippedNoWebsite => "SKIP   no website".to_string(),
            HuntOutcome::NoContact => "MISS   no contact found".to_string(),
            HuntOutcome::Previewed => "DRY    preview".to_string(),
            HuntOutcome::Applied => "SENT   applied".to_string(),
            HuntOutcome::SendFailed(reason) => format!("FAIL   {reason}"),
        };
        println!("[{label}] {} ({}){contact}", event.company_name, event.slug);

        if event.outcome == HuntOutcome::Previewed
            && let Some(email) = &event.email
        {
            println!("    To: {}", email.to);
            println!("    Subject: {}", email.subject);
            for line in email.body.lines() {
                println!("    | {line}");
            }
            println!();
        }
    }
}

fn print_report(report: &HuntReport, dry_run: bool) {
    println!();
    println!("Hunt finished{}:", if dry_run { " (dry run)" } else { "" });
    println!("  discovered          {}", report.discovered);
    println!("  already processed   {}", report.already_processed);
    println!("  applied             {}", report.applied);
    println!("  previewed           {}", report.previewed);
    println!("  no contact          {}", report.no_contact);
    println!("  no website          {}", report.skipped_no_website);
    println!("  send failures       {}", report.send_failed);
    if report.daily_limit_reached {
        println!("  daily send limit reached, run again tomorrow");
    }
}

fn print_applications(applications: &[Application]) {
    let rows = applications
        .iter()
        .map(|application| {
            [
                application.applied_at.clone(),
                application.status.to_string(),
                application.yc_slug.clone(),
                application.company_name.clone(),
                application
                    .contact_email
                    .clone()
                    .unwrap_or_else(|| "-".into()),
            ]
        })
        .collect::<Vec<_>>();
    let headers = ["APPLIED AT", "STATUS", "SLUG", "COMPANY", "CONTACT"];
    let widths = (0..headers.len())
        .map(|column| {
            rows.iter()
                .map(|row| row[column].chars().count())
                .chain(std::iter::once(headers[column].len()))
                .max()
                .unwrap_or_default()
        })
        .collect::<Vec<_>>();
    let format_row = |cells: Vec<&str>| {
        cells
            .iter()
            .zip(&widths)
            .map(|(cell, width)| format!("{cell:<width$}"))
            .collect::<Vec<_>>()
            .join("  ")
    };
    println!("{}", format_row(headers.to_vec()));
    for row in &rows {
        println!("{}", format_row(row.iter().map(String::as_str).collect()));
    }
}
