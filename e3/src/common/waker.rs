use futures::task::AtomicTask;
use std::sync::Arc;

pub(crate) struct Waker
{
    pub(crate) task: Arc<AtomicTask>
}

impl Waker
{
    pub(crate) fn new() -> Self
    {
        Waker {
            task: Arc::new(AtomicTask::new())
        }
    }
}

impl Clone for Waker
{
    fn clone(&self) -> Self
    {
        Self {
            task: Arc::clone(&self.task)
        }
    }
}
