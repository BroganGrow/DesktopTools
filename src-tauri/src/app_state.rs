use parking_lot::Mutex;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[derive(Debug)]
pub struct JobControl {
    pub id: String,
    pub cancelled: AtomicBool,
}

impl JobControl {
    pub fn new(id: String) -> Self {
        Self {
            id,
            cancelled: AtomicBool::new(false),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

#[derive(Default)]
pub struct AppState {
    current_job: Mutex<Option<Arc<JobControl>>>,
}

impl AppState {
    pub fn set_current_job(&self, job: Arc<JobControl>) {
        *self.current_job.lock() = Some(job);
    }

    pub fn clear_current_job(&self, job_id: &str) {
        let mut guard = self.current_job.lock();
        if guard.as_ref().map(|job| job.id.as_str()) == Some(job_id) {
            *guard = None;
        }
    }

    pub fn current_job(&self) -> Option<Arc<JobControl>> {
        self.current_job.lock().as_ref().map(Arc::clone)
    }
}
