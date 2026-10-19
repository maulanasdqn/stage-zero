use std::future::Future;

use crate::entities::{
    Application, ApplicationStatus, BatchInfo, Contact, Founder, HuntCriteria, HuntEvent,
    NewApplication, OutgoingEmail, Startup,
};
use crate::errors::DomainError;

pub trait StartupDirectory: Send + Sync {
    fn batches(&self) -> impl Future<Output = Result<Vec<BatchInfo>, DomainError>> + Send;

    fn fetch(
        &self,
        criteria: &HuntCriteria,
    ) -> impl Future<Output = Result<Vec<Startup>, DomainError>> + Send;
}

pub trait FounderSource: Send + Sync {
    fn founders(
        &self,
        startup: &Startup,
    ) -> impl Future<Output = Result<Vec<Founder>, DomainError>> + Send;
}

pub trait ContactFinder: Send + Sync {
    fn find(
        &self,
        startup: &Startup,
        founders: &[Founder],
    ) -> impl Future<Output = Result<Option<Contact>, DomainError>> + Send;
}

pub trait ApplicationRepository: Send + Sync {
    fn find(
        &self,
        yc_slug: &str,
    ) -> impl Future<Output = Result<Option<Application>, DomainError>> + Send;

    fn exists(&self, yc_slug: &str) -> impl Future<Output = Result<bool, DomainError>> + Send;

    fn record(
        &self,
        application: NewApplication,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    fn list(
        &self,
        status: Option<ApplicationStatus>,
    ) -> impl Future<Output = Result<Vec<Application>, DomainError>> + Send;

    fn count_applied_today(&self) -> impl Future<Output = Result<usize, DomainError>> + Send;

    fn reset(
        &self,
        yc_slug: Option<String>,
    ) -> impl Future<Output = Result<usize, DomainError>> + Send;
}

pub trait Mailer: Send + Sync {
    fn send(
        &self,
        email: &OutgoingEmail,
    ) -> impl Future<Output = Result<String, DomainError>> + Send;
}

pub trait Delay: Send + Sync {
    fn wait(&self) -> impl Future<Output = ()> + Send;
}

pub trait HuntProgress: Send + Sync {
    fn on_discovered(&self, count: usize);

    fn on_event(&self, event: &HuntEvent);
}
