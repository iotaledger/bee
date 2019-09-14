use crate::constants::messages::*;

use tokio::sync::watch::{self, Receiver, Sender};

pub(crate) struct SignalRx(pub(crate) Receiver<bool>);

pub(crate) struct Signal
{
    tx: Sender<bool>,
    rx: Receiver<bool>
}

impl Signal
{
    pub(crate) fn new() -> Self
    {
        let (tx, rx) = watch::channel(false);

        Self { tx, rx }
    }

    pub(crate) fn add_rx(&self) -> SignalRx
    {
        SignalRx(self.rx.clone())
    }

    pub(crate) fn emit(&mut self)
    {
        self.tx.broadcast(true).expect(EMIT_SIGNAL_ERROR);
    }
}
