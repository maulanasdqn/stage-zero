use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::{Connection, OptionalExtension, Row, params};
use stagezero_core::DomainError;
use stagezero_core::entities::{Application, ApplicationStatus, NewApplication};
use stagezero_core::traits::ApplicationRepository;

use crate::errors::InfraError;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS applications (
    id INTEGER PRIMARY KEY,
    yc_slug TEXT UNIQUE NOT NULL,
    company_name TEXT NOT NULL,
    website TEXT,
    contact_email TEXT,
    status TEXT NOT NULL CHECK (status IN ('APPLIED', 'SKIPPED', 'FAILED_NO_CONTACT')),
    applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_applications_status ON applications (status);
";

#[derive(Clone)]
pub struct SqliteApplicationRepository {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteApplicationRepository {
    pub fn open(path: &Path) -> Result<Self, InfraError> {
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent)?;
        }
        Self::initialize(Connection::open(path)?)
    }

    pub fn in_memory() -> Result<Self, InfraError> {
        Self::initialize(Connection::open_in_memory()?)
    }

    fn initialize(connection: Connection) -> Result<Self, InfraError> {
        connection.execute_batch(SCHEMA)?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    async fn run<T, Operation>(&self, operation: Operation) -> Result<T, DomainError>
    where
        T: Send + 'static,
        Operation: FnOnce(&Connection) -> rusqlite::Result<T> + Send + 'static,
    {
        let connection = Arc::clone(&self.connection);
        tokio::task::spawn_blocking(move || {
            let guard = connection
                .lock()
                .map_err(|_| DomainError::Repository("sqlite connection lock poisoned".into()))?;
            operation(&guard).map_err(|error| DomainError::Repository(error.to_string()))
        })
        .await
        .map_err(|error| DomainError::Repository(error.to_string()))?
    }
}

fn application_from_row(row: &Row<'_>) -> rusqlite::Result<Application> {
    let raw_status: String = row.get("status")?;
    let status = raw_status.parse::<ApplicationStatus>().map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
    })?;
    Ok(Application {
        id: row.get("id")?,
        yc_slug: row.get("yc_slug")?,
        company_name: row.get("company_name")?,
        website: row.get("website")?,
        contact_email: row.get("contact_email")?,
        status,
        applied_at: row.get("applied_at")?,
    })
}

impl ApplicationRepository for SqliteApplicationRepository {
    async fn find(&self, yc_slug: &str) -> Result<Option<Application>, DomainError> {
        let yc_slug = yc_slug.to_string();
        self.run(move |connection| {
            connection
                .query_row(
                    "SELECT id, yc_slug, company_name, website, contact_email, status, applied_at
                     FROM applications WHERE yc_slug = ?1",
                    params![yc_slug],
                    application_from_row,
                )
                .optional()
        })
        .await
    }

    async fn exists(&self, yc_slug: &str) -> Result<bool, DomainError> {
        let yc_slug = yc_slug.to_string();
        self.run(move |connection| {
            connection
                .query_row(
                    "SELECT 1 FROM applications WHERE yc_slug = ?1",
                    params![yc_slug],
                    |_| Ok(()),
                )
                .optional()
                .map(|found| found.is_some())
        })
        .await
    }

    async fn record(&self, application: NewApplication) -> Result<(), DomainError> {
        self.run(move |connection| {
            connection
                .execute(
                    "INSERT INTO applications (yc_slug, company_name, website, contact_email, status)
                     VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT (yc_slug) DO UPDATE SET
                         company_name = excluded.company_name,
                         website = excluded.website,
                         contact_email = excluded.contact_email,
                         status = excluded.status,
                         applied_at = CURRENT_TIMESTAMP",
                    params![
                        application.yc_slug,
                        application.company_name,
                        application.website,
                        application.contact_email,
                        application.status.as_str(),
                    ],
                )
                .map(|_| ())
        })
        .await
    }

    async fn list(
        &self,
        status: Option<ApplicationStatus>,
    ) -> Result<Vec<Application>, DomainError> {
        self.run(move |connection| {
            let mut statement = connection.prepare(
                "SELECT id, yc_slug, company_name, website, contact_email, status, applied_at
                 FROM applications
                 WHERE ?1 IS NULL OR status = ?1
                 ORDER BY applied_at DESC, id DESC",
            )?;
            statement
                .query_map(
                    params![status.map(|status| status.as_str())],
                    application_from_row,
                )?
                .collect()
        })
        .await
    }

    async fn count_applied_today(&self) -> Result<usize, DomainError> {
        self.run(|connection| {
            connection.query_row(
                "SELECT COUNT(*) FROM applications
                 WHERE status = 'APPLIED' AND date(applied_at) = date('now')",
                [],
                |row| row.get::<_, i64>(0),
            )
        })
        .await
        .map(|count| count.max(0) as usize)
    }

    async fn reset(&self, yc_slug: Option<String>) -> Result<usize, DomainError> {
        self.run(move |connection| match yc_slug {
            Some(yc_slug) => connection.execute(
                "DELETE FROM applications WHERE yc_slug = ?1",
                params![yc_slug],
            ),
            None => connection.execute("DELETE FROM applications", []),
        })
        .await
    }
}
