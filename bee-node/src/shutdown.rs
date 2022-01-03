// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use futures::channel::oneshot;
use log::warn;

pub(crate) type ShutdownTx = oneshot::Sender<()>;
pub(crate) type ShutdownRx = oneshot::Receiver<()>;

#[cfg(unix)]
pub(crate) use tokio::signal::unix::SignalKind;

/// Creates a shutdown listener for Unix platforms.
#[cfg(unix)]
pub(crate) fn shutdown_listener(signals: Vec<SignalKind>) -> ShutdownRx {
    use futures::future;
    use tokio::signal::unix::{signal, Signal};

    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    tokio::spawn(async move {
        let mut signals = signals
            .iter()
            .map(|kind| signal(*kind).unwrap())
            .collect::<Vec<Signal>>();

        let signal_futures = signals.iter_mut().map(|signal| Box::pin(signal.recv()));

        let (signal_event, _, _) = future::select_all(signal_futures).await;

        if signal_event.is_none() {
            panic!("Shutdown signal stream failed, channel may have closed.");
        }

        shutdown_procedure(shutdown_tx);
    });

    shutdown_rx
}

/// Creates a shutdown listener for Non-Unix platforms.
#[cfg(not(unix))]
pub(crate) fn shutdown_listener() -> ShutdownRx {
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    tokio::spawn(async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            panic!("Failed to intercept CTRL-C: {:?}.", e);
        }

        shutdown_procedure(shutdown_tx);
    });

    receiver
}

fn shutdown_procedure(shutdown_tx: ShutdownTx) {
    warn!("Gracefully shutting down the node, this may take some time.");

    if let Err(e) = shutdown_tx.send(()) {
        panic!("Failed to send the shutdown signal: {:?}", e);
    }
}
