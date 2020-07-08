// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

//! A module that deals with asynchronous workers in general.

use thiserror::Error;

/// Errors, that might occur during the lifetime of asynchronous workers.
#[derive(Error, Debug)]
pub enum Error {
    /// Occurs, when there is some asynchronous I/O error.
    #[error("An asynchronous operation failed.")]
    AsynchronousOperationFailed(#[from] std::io::Error),

    /// Occurs, when a message couldn't be sent over a `futures::channel::mpsc` channel.
    #[error("Sending a message to a task failed.")]
    SendingMessageFailed(#[from] futures::channel::mpsc::SendError),
}
