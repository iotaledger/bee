//! Waker

use futures::task::AtomicTask;
use std::sync::Arc;

/// A waker used to wake up sleeping asynchronous tasks.
pub struct Waker {
    /// Internal reference counted thread-safe task.
    pub task: Arc<AtomicTask>,
}

impl Waker {
    /// Creates a new waker.
    pub fn new() -> Self {
        Waker { task: Arc::new(AtomicTask::new()) }
    }
}

impl Clone for Waker {
    fn clone(&self) -> Self {
        Self { task: Arc::clone(&self.task) }
    }
}
