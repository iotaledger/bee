#![no_main]

use bee_common::packable::Packable;
use bee_message::Message;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = Message::unpack(&mut data.to_vec().as_slice());
});
