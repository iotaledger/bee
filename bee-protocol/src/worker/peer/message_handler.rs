// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::packet::{Header, HEADER_SIZE};

use bee_network::Multiaddr;

use futures::{
    channel::oneshot,
    future::{self, FutureExt},
    select,
    stream::StreamExt,
};

use log::trace;

type EventRecv = flume::r#async::RecvStream<'static, std::vec::Vec<u8>>;
type ShutdownRecv = future::Fuse<oneshot::Receiver<()>>;

/// The read state of the message handler.
///
/// This type is used by `MessageHandler` to decide what should be read next when handling an
/// event.
enum ReadState {
    /// `MessageHandler` should read a header.
    Header,
    /// `MessageHandler` should read a payload based on a header.
    Payload(Header),
}

/// A message handler.
///
/// It takes care of processing events into messages that can be processed by the workers.
pub(super) struct MessageHandler {
    events: EventHandler,
    // FIXME: see if we can implement `Stream` for the `MessageHandler` and use the
    // `ShutdownStream` type instead.
    shutdown: ShutdownRecv,
    state: ReadState,
    /// The address of the peer. This field is only here for logging purposes.
    address: Multiaddr,
}

impl MessageHandler {
    /// Create a new message handler from an event receiver, a shutdown receiver and the peer's
    /// address.
    pub(super) fn new(receiver: EventRecv, shutdown: ShutdownRecv, address: Multiaddr) -> Self {
        Self {
            events: EventHandler::new(receiver),
            shutdown,
            // The handler should read a header first.
            state: ReadState::Header,
            address,
        }
    }
    /// Fetch the header and payload of a message.
    ///
    /// This method only returns `None` if a shutdown signal is received.
    pub(super) async fn fetch_message(&mut self) -> Option<(Header, &[u8])> {
        // loop until we can return the header and payload
        loop {
            match &self.state {
                // Read a header.
                ReadState::Header => {
                    // We need `HEADER_SIZE` bytes to read a header.
                    let bytes = self
                        .events
                        .fetch_bytes_or_shutdown(&mut self.shutdown, HEADER_SIZE)
                        .await?;
                    trace!("[{}] Reading Header...", self.address);
                    let header = Header::from_bytes(bytes);
                    // Now we are ready to read a payload.
                    self.state = ReadState::Payload(header);
                }
                // Read a payload.
                ReadState::Payload(header) => {
                    // We read the quantity of bytes stated by the header.
                    let bytes = self
                        .events
                        .fetch_bytes_or_shutdown(&mut self.shutdown, header.packet_length.into())
                        .await?;
                    // FIXME: Avoid this clone
                    let header = header.clone();
                    // Now we are ready to read the next message's header.
                    self.state = ReadState::Header;
                    // We return the current message's header and payload.
                    return Some((header, bytes));
                }
            }
        }
    }
}

// An event handler.
//
// This type takes care of actually receiving the events and appending them to an inner buffer so
// they can be used seamlessly by the `MessageHandler`.
struct EventHandler {
    receiver: EventRecv,
    buffer: Vec<u8>,
    offset: usize,
}

impl EventHandler {
    /// Create a new event handler from an event receiver.
    fn new(receiver: EventRecv) -> Self {
        Self {
            receiver,
            buffer: vec![],
            offset: 0,
        }
    }

    /// Push a new event into the buffer.
    ///
    /// This method also removes the `..self.offset` range from the buffer and sets the offset back
    /// to zero. Which means that this should only be called when the buffer is empty or when there
    /// are not enough bytes to read a new header or payload.
    fn push_event(&mut self, mut bytes: Vec<u8>) {
        // Remove the already read bytes from the buffer.
        self.buffer = self.buffer.split_off(self.offset);
        // Reset the offset.
        self.offset = 0;
        // Append the bytes of the new event
        if self.buffer.is_empty() {
            self.buffer = bytes;
        } else {
            self.buffer.append(&mut bytes);
        }
    }
    /// Fetch a slice of bytes of a determined length.
    ///
    /// The future returned by this method will be ready until there are enough bytes to fullfill
    /// the request.
    async fn fetch_bytes(&mut self, len: usize) -> &[u8] {
        // We need to be sure that we have enough bytes in the buffer.
        while self.offset + len > self.buffer.len() {
            // If there are not enough bytes in the buffer, we must receive new events
            if let Some(event) = self.receiver.next().await {
                // If we received an event, we push it to the buffer.
                self.push_event(event);
            }
        }
        // Get the requested bytes. This will not panic because the loop above only exists if we
        // have enough bytes to do this step.
        let bytes = &self.buffer[self.offset..(self.offset + len)];
        // Increase the offset by the length of the byte slice.
        self.offset += len;
        bytes
    }

    /// Helper method to be able to shutdown when fetching bytes for a message.
    ///
    /// This method returns `None` if a shutdown signal is received, otherwise it returns the
    /// requested bytes.
    async fn fetch_bytes_or_shutdown<'a>(
        &'a mut self,
        mut shutdown: &'a mut ShutdownRecv,
        len: usize,
    ) -> Option<&'a [u8]> {
        select! {
            // Always select `shutdown` first, otherwise you can end with an infinite loop.
            _ = shutdown => None,
            bytes = self.fetch_bytes(len).fuse() => Some(bytes),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{channel::oneshot, future::FutureExt};
    use std::time::Duration;
    use tokio::{spawn, time::delay_for};

    /// Generate a vector of events filled with messages of a desired length.
    fn gen_events(event_len: usize, msg_size: usize, n_msg: usize) -> Vec<Vec<u8>> {
        // Bytes of all the messages.
        let mut msgs = vec![0u8; msg_size * n_msg];
        // We need 3 bytes for the header. Thus the message length stored in the header should be 3
        // bytes shorter.
        let msg_len = ((msg_size - 3) as u16).to_le_bytes();
        // We write the bytes that correspond to the message length in the header.
        for i in (0..n_msg).map(|i| i * msg_size + 1) {
            msgs[i] = msg_len[0];
            msgs[i + 1] = msg_len[1];
        }
        // Finally, we split all the bytes into events.
        msgs.chunks(event_len).map(Vec::from).collect()
    }

    /// Test if the `MessageHandler` can produce an exact number of messages of a desired length,
    /// divided in events of an specified length. This test checks that:
    /// - The header and payload of all the messages have the right content.
    /// - The number of produced messages is the desired one.
    async fn test(event_size: usize, msg_size: usize, msg_count: usize) {
        let msg_len = msg_size - 3;
        // Produce the events
        let events = gen_events(event_size, msg_size, msg_count);
        // Create a new message handler
        let (sender_shutdown, receiver_shutdown) = oneshot::channel::<()>();
        let (sender, receiver) = flume::unbounded::<Vec<u8>>();
        let mut msg_handler = MessageHandler::new(
            receiver.into_stream(),
            receiver_shutdown.fuse(),
            "127.0.0.1:8080".parse().unwrap(),
        );
        // Create the task that does the checks of the test.
        let handle = spawn(async move {
            // The messages are expected to be filled with zeroes except for the message length
            // field of the header.
            let expected_bytes = vec![0u8; msg_len];
            let expected_msg = (
                Header {
                    packet_type: 0,
                    packet_length: msg_len as u16,
                },
                expected_bytes.as_slice(),
            );
            // Count how many messages can be fetched.
            let mut counter = 0;
            while let Some(msg) = msg_handler.fetch_message().await {
                // Assert that the messages' content is correct.
                assert_eq!(msg, expected_msg);
                counter += 1;
            }
            // Assert that the number of messages is correct.
            assert_eq!(msg_count, counter);
            // Return back the message handler to avoid dropping the channels.
            msg_handler
        });
        // Send all the events to the message handler.
        for event in events {
            sender.send(event).unwrap();
            delay_for(Duration::from_millis(1)).await;
        }
        // Sleep to be sure the handler had time to produce all the messages.
        delay_for(Duration::from_millis(1)).await;
        // Send a shutdown signal.
        sender_shutdown.send(()).unwrap();
        // Await for the task with the checks to be completed.
        assert!(handle.await.is_ok());
    }

    /// Test that messages are produced correctly when they are divided into one byte events.
    #[tokio::test]
    async fn one_byte_events() {
        test(1, 5, 10).await;
    }

    /// Test that messages are produced correctly when each mes// let peer_id: PeerId =
    /// Url::from_url_str("tcp://[::1]:16000").await.unwrap().into();sage fits exactly into an event.
    #[tokio::test]
    async fn one_message_per_event() {
        test(5, 5, 10).await;
    }

    /// Test that messages are produced correctly when two messages fit exactly into an event.
    #[tokio::test]
    async fn two_messages_per_event() {
        test(10, 5, 10).await;
    }

    /// Test that messages are produced correctly when a message fits exactly into two events.
    #[tokio::test]
    async fn two_events_per_message() {
        test(5, 10, 10).await;
    }

    /// Test that messages are produced correctly when a message does not fit in a single event and
    /// it is not aligned either.
    #[tokio::test]
    async fn misaligned_messages() {
        test(3, 5, 10).await;
    }

    /// Test that the handler stops producing messages after receiving the shutdown signal.
    ///
    /// This test is basically the same as the `one_message_per_event` test. But the last event is
    /// sent after the shutdown signal. As a consequence, the last message is not produced by the
    /// message handler.
    #[tokio::test]
    async fn shutdown() {
        let event_size = 5;
        let msg_size = event_size;
        let msg_count = 10;

        let msg_len = msg_size - 3;

        let mut events = gen_events(event_size, msg_size, msg_count);
        // Put the last event into its own variable.
        let last_event = events.pop().unwrap();

        let (sender_shutdown, receiver_shutdown) = oneshot::channel::<()>();
        let (sender, receiver) = flume::unbounded::<Vec<u8>>();

        let mut msg_handler = MessageHandler::new(
            receiver.into_stream(),
            receiver_shutdown.fuse(),
            "127.0.0.1:8080".parse().unwrap(),
        );

        let handle = spawn(async move {
            let expected_bytes = vec![0u8; msg_len];
            let expected_msg = (
                Header {
                    packet_type: 0,
                    packet_length: msg_len as u16,
                },
                expected_bytes.as_slice(),
            );

            let mut counter = 0;
            while let Some(msg) = msg_handler.fetch_message().await {
                assert_eq!(msg, expected_msg);
                counter += 1;
            }
            // Assert that we are missing one message
            assert_eq!(msg_count - 1, counter);

            msg_handler
        });

        for event in events {
            sender.send(event).unwrap();
            delay_for(Duration::from_millis(1)).await;
        }

        sender_shutdown.send(()).unwrap();
        delay_for(Duration::from_millis(1)).await;
        // Send the last event after the shutdown signal
        sender.send(last_event).unwrap();

        assert!(handle.await.is_ok());
    }
}
