use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use serde::Serialize;
use stagezero_core::entities::{HuntCriteria, HuntEvent, HuntReport};
use stagezero_core::traits::HuntProgress;
use stagezero_infra::container::{
    LiveHuntUseCase, LiveResetUseCase, LiveServices, LiveStatusUseCase,
};

#[derive(Debug, Clone, Default, Serialize)]
pub struct HuntRun {
    pub running: bool,
    pub criteria: Option<HuntCriteria>,
    pub discovered: Option<usize>,
    pub events: Vec<HuntEvent>,
    pub report: Option<HuntReport>,
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct AppState {
    pub hunt: Arc<LiveHuntUseCase>,
    pub status: Arc<LiveStatusUseCase>,
    pub reset: Arc<LiveResetUseCase>,
    pub mailer_configured: bool,
    pub resume_loaded: bool,
    pub hunt_running: Arc<AtomicBool>,
    pub hunt_run: Arc<Mutex<HuntRun>>,
}

impl AppState {
    pub fn new(services: LiveServices) -> Self {
        Self {
            hunt: Arc::new(services.hunt),
            status: Arc::new(services.status),
            reset: Arc::new(services.reset),
            mailer_configured: services.mailer_configured,
            resume_loaded: services.resume_loaded,
            hunt_running: Arc::new(AtomicBool::new(false)),
            hunt_run: Arc::new(Mutex::new(HuntRun::default())),
        }
    }

    pub fn try_claim_hunt(&self) -> bool {
        self.hunt_running
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }

    pub fn release_hunt(&self) {
        self.hunt_running.store(false, Ordering::SeqCst);
    }

    pub fn current_run(&self) -> MutexGuard<'_, HuntRun> {
        lock_run(&self.hunt_run)
    }
}

pub fn lock_run(run: &Mutex<HuntRun>) -> MutexGuard<'_, HuntRun> {
    run.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub struct SharedRunProgress {
    pub run: Arc<Mutex<HuntRun>>,
}

impl HuntProgress for SharedRunProgress {
    fn on_discovered(&self, count: usize) {
        lock_run(&self.run).discovered = Some(count);
    }

    fn on_event(&self, event: &HuntEvent) {
        lock_run(&self.run).events.push(event.clone());
    }
}
