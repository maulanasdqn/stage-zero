# StageZero

StageZero finds early-stage Y Combinator startups, works out who the founders are and how to reach them, and sends each one a personalized application with your resume attached. You can run it from the terminal or from a local web UI where you review every email before it goes out.

It is written in Rust as a Cargo workspace that follows Clean Architecture.

## Features

- **YC directory:** lists every YC batch and filters startups by batch, team size and industry keywords. Data comes from the [yc-oss](https://github.com/yc-oss/api) API.
- **Founders and contacts:** reads founder names from each company's YC page, then looks for an email on the startup's website (`mailto:` links, plain text, Cloudflare-obfuscated addresses). Addresses that match a founder's name rank highest, generic ones like `support@` are ignored, and when nothing is found it falls back to a guess like `firstname@domain`. Guesses are labelled so you can review them.
- **Personalized emails:** a template with variables, plus a library of your highlights. Each startup gets the highlights that best match its industries, tags, one-liner and description.
- **Review before sending:** in the web UI you open a startup, check the founders and contact, edit the recipient, subject and body, then send or skip.
- **Auto hunt:** processes a whole batch in one go, with a dry-run mode that only previews emails.
- **Sending:** Cloudflare Email Sending (default) or Resend, with your resume attached.
- **Safety rails:** a SQLite tracker so no startup is contacted twice, a daily send limit, and a random 2–5 second delay between startups.

## Architecture

```text
stage-zero/
├── stagezero-core/    entities, trait ports, use cases, personalization (no I/O)
├── stagezero-infra/   SQLite, yc-oss directory, YC founder scraper, website contact
│                      finder, Cloudflare and Resend mailers, config, composition root
├── stagezero-api/     Axum server: JSON API under /api, serves the built web UI
├── stagezero-web/     React + TanStack Router/Query/Form + shadcn/ui frontend
└── stagezero-cli/     `stagezero` binary (clap)
```

The core defines traits (`StartupDirectory`, `FounderSource`, `ContactFinder`, `ApplicationRepository`, `Mailer`, `Delay`). Infra implements them, and the CLI and API both wire them together through `stagezero_infra::container`.

## Quick start

Requires Rust 1.88 or newer, plus Node.js and pnpm for the web UI.

```bash
cargo run -p stagezero-cli -- init        # writes stagezero.toml from stagezero.example.toml
```

Edit `stagezero.toml` (it is git-ignored), put your resume where `[applicant].resume_path` points, and set your email credentials:

```bash
export CLOUDFLARE_API_TOKEN=...           # token with Email Sending permission
```

Then preview without sending anything:

```bash
cargo run -p stagezero-cli -- hunt --batch W24 --size 1-10 --industry "B2B SaaS" --limit 5 --dry-run
```

Or build the web UI and start the server at http://127.0.0.1:3000:

```bash
pnpm --dir stagezero-web install
pnpm --dir stagezero-web build
cargo run -p stagezero-api
```

## Configuration

`stagezero.toml` sections:

| Section | Keys |
| --- | --- |
| `[email]` | `provider` (`cloudflare` or `resend`), `from`, `from_name`, `cloudflare_account_id`, optional `api_token` |
| `[applicant]` | `name`, `email` (used as reply-to), `resume_path`, `portfolio`, `max_highlights` |
| `[[applicant.highlights]]` | `text`, `keywords`, optional `pinned` |
| `[template]` | `subject`, `body` |
| `[storage]` | `database_path` (default `stagezero.db`) |
| `[outreach]` | `min_delay_secs`, `max_delay_secs`, `daily_limit` |

Environment variables override the file:

| Variable | Purpose |
| --- | --- |
| `CLOUDFLARE_API_TOKEN` | Cloudflare API token when `provider = "cloudflare"` |
| `CLOUDFLARE_ACCOUNT_ID` | Cloudflare account ID |
| `RESEND_API_KEY` | Resend API key when `provider = "resend"` |
| `STAGEZERO_CONFIG` | Config path for the API server (default `stagezero.toml`) |
| `STAGEZERO_ADDR` | Listen address for the API server (default `127.0.0.1:3000`) |
| `STAGEZERO_WEB_DIR` | Built web UI to serve (default `stagezero-web/dist`) |
| `RUST_LOG` | Log filter, e.g. `info` or `stagezero_infra=debug` |

For Cloudflare, the sending domain must be onboarded first (`wrangler email sending enable yourdomain.com`). If the `from` address should also receive replies, enable Email Routing for the domain.

### Template variables

Write variables as `{{name}}` or `{{ name }}`.

| Variable | Value |
| --- | --- |
| `founder_first_name` | Matched or sole founder's first name, otherwise `there` |
| `founder_name` | Founder's full name, otherwise `<Company> team` |
| `company_name`, `batch`, `industry`, `one_liner` | From the YC directory |
| `one_liner_clause` | ` ("<one-liner>")`, or empty when the startup has none |
| `highlights` | Bullet list of your highlights that best match the startup |
| `applicant_name`, `applicant_first_name`, `applicant_email`, `portfolio` | From `[applicant]` |

### Highlights

Each highlight has keywords. A keyword that appears in the startup's industries or tags scores 3, in the one-liner 2, and in the long description 1. Matching works on whole words and accepts plurals, so `agent` matches "agents" but `ai` does not match "email". The top `max_highlights` are used. Pinned highlights always come first, and if fewer highlights match than `max_highlights`, the rest of the slots are filled in config order. The review drawer in the UI shows which keywords matched.

## CLI

```text
stagezero init [--force]
stagezero hunt --batch <W24|S24|F24|X25|winter-2024> [--size 1-10] [--industry "<keywords>"] [--limit N] [--dry-run] [--yes]
stagezero status [--status APPLIED|SKIPPED|FAILED_NO_CONTACT]
stagezero reset [--slug <yc-slug>] [--yes]
```

All commands accept `--config <path>`.

## Web UI

`stagezero-web` is a Vite app built with React 19, TanStack Router (file-based routes), TanStack Query, TanStack Table v9, TanStack Form with Zod, Tailwind CSS v4 and shadcn/ui. The UI is composed from shadcn components (Sidebar, Card, Item, Field, InputGroup, Empty, ButtonGroup, Sheet, AlertDialog and more) rather than hand-styled HTML. It has three pages:

- **Explore:** pick a batch and filters, then search, sort and page through the results (server-side, with every option kept in the URL), and open a startup in a side sheet to see its founders, contact, picked highlights and an editable draft. Send (with confirmation) or skip from there.
- **Auto hunt:** start a dry run or a live hunt and follow its progress, report and per-startup activity.
- **Applications:** summary cards and a paginated, sortable tracker table with search, a status filter, per-row actions and reset.

For development, run the API and the Vite dev server side by side. Vite proxies `/api` to `127.0.0.1:3000`:

```bash
cargo run -p stagezero-api
pnpm --dir stagezero-web dev     # http://localhost:5173
```

Frontend checks: `pnpm --dir stagezero-web lint` and `pnpm --dir stagezero-web build` (which also type-checks).

## HTTP API

Every endpoint lives under `/api`. Any other path serves the web UI.

List endpoints return `{ "data": [...], "pagination": { "page", "per_page", "total", "total_pages" } }`. `page` starts at 1 and `per_page` defaults to 20 (maximum 100). `/api/startups` also returns `batch_total` and `tracked`. YC directory responses are cached in memory for 10 minutes.

| Method | Path | Description |
| --- | --- | --- |
| GET | `/api/health` | Health check |
| GET | `/api/settings` | Mailer, sender and resume status |
| GET | `/api/batches` | YC batches, newest first |
| GET | `/api/startups?batch=&size=&industry=&q=&hide_contacted=&sort=&order=&page=&per_page=` | Paginated startups in a batch with their tracker status. `sort` is `name` or `team_size` |
| POST | `/api/startups/inspect` | Founders, contact, highlights and drafted email for `{ "startup": ... }` |
| POST | `/api/startups/send` | Send a reviewed `{ "startup": ..., "email": { "to", "subject", "body" } }` |
| POST | `/api/startups/skip` | Mark `{ "startup": ... }` as skipped |
| GET | `/api/applications?status=&q=&sort=&order=&page=&per_page=` | Paginated tracker. `sort` is `applied_at` (default, newest first), `company_name` or `status` |
| GET | `/api/applications/summary` | Counts per status |
| DELETE | `/api/applications?slug=` | Remove one record, or all records when `slug` is omitted |
| POST | `/api/hunt` | Start a background hunt with `{ "batch", "size", "industry", "limit", "dry_run" }` |
| GET | `/api/hunt` | Progress, events and report of the current or last hunt |

## Development

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Codebase rules:

- **No comments** in `.rs` files. Names should explain intent.
- **Every dependency goes through a trait** defined in `stagezero-core`. Use cases depend on those traits, never on concrete infra types.

The tests use in-memory fakes for the core use cases, and HTML/JSON fixtures in `stagezero-infra/tests/fixtures` for the scrapers, contact finder, mailers and SQLite repository.

## Responsible use

Send to a small number of startups at a time, keep the daily limit low, and read every email before it goes out. Respect the websites you visit and the people you write to. Use a domain with SPF, DKIM and DMARC set up, and take bounces seriously: guessed addresses can hurt your sender reputation.
