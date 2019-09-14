use super::signal::{Signal, SignalRx};

use tokio::prelude::*;
use tokio::runtime::current_thread;
use tokio_signal::ctrl_c;

pub(crate) struct GracefulShutdown(Signal);

impl GracefulShutdown
{
    pub(crate) fn new() -> Self
    {
        Self(Signal::new())
    }

    pub(crate) fn wait_for_ctrl_c(&self)
    {
        let ctrl_c = ctrl_c().flatten_stream().take(1).for_each(|_| Ok(()));

        current_thread::block_on_all(ctrl_c).unwrap();
    }

    pub(crate) fn send_termination_signal(&mut self)
    {
        self.0.emit()
    }

    pub(crate) fn add_rx(&self) -> SignalRx
    {
        self.0.add_rx()
    }
}
