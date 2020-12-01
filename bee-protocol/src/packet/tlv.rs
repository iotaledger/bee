// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Type-length-value encoding on top of the packets.

use crate::packet::{Header, Packet, HEADER_SIZE};

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub(crate) enum TlvError {
    InvalidAdvertisedType(u8, u8),
    InvalidAdvertisedLength(usize, usize),
    InvalidLength(usize),
}

/// Deserializes a TLV header and a byte buffer into a packet.
///
/// # Arguments
///
/// * `header`  -   The TLV header to deserialize from.
/// * `bytes`   -   The byte buffer to deserialize from.
///
/// # Errors
///
/// * The advertised packet type does not match the required packet type.
/// * The advertised packet length does not match the buffer length.
/// * The buffer length is not within the allowed size range of the required packet type.
pub(crate) fn tlv_from_bytes<P: Packet>(header: &Header, bytes: &[u8]) -> Result<P, TlvError> {
    if header.packet_type != P::ID {
        return Err(TlvError::InvalidAdvertisedType(header.packet_type, P::ID));
    }

    if header.packet_length as usize != bytes.len() {
        return Err(TlvError::InvalidAdvertisedLength(
            header.packet_length as usize,
            bytes.len(),
        ));
    }

    if !P::size_range().contains(&bytes.len()) {
        return Err(TlvError::InvalidLength(bytes.len()));
    }

    Ok(P::from_bytes(bytes))
}

/// Serializes a TLV header and a packet into a byte buffer.
///
/// # Arguments
///
/// * `packet` -   The packet to serialize.
pub(crate) fn tlv_into_bytes<P: Packet>(packet: P) -> Vec<u8> {
    let size = packet.size();
    let mut bytes = vec![0u8; HEADER_SIZE + size];
    let (header, payload) = bytes.split_at_mut(HEADER_SIZE);

    Header {
        packet_type: P::ID,
        packet_length: size as u16,
    }
    .to_bytes(header);
    packet.into_bytes(payload);

    bytes
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::packet::{Heartbeat, Message as MessagePacket, MessageRequest, MilestoneRequest, Packet};

    use rand::Rng;

    use std::convert::TryInto;

    fn invalid_advertised_type<P: Packet>() {
        match tlv_from_bytes::<P>(
            &Header {
                packet_type: P::ID + 1,
                packet_length: P::size_range().start as u16,
            },
            &Vec::with_capacity(P::size_range().start),
        ) {
            Err(TlvError::InvalidAdvertisedType(advertised_type, actual_type)) => {
                assert_eq!(advertised_type, P::ID + 1);
                assert_eq!(actual_type, P::ID);
            }
            _ => unreachable!(),
        }
    }

    fn invalid_advertised_length<P: Packet>() {
        match tlv_from_bytes::<P>(
            &Header {
                packet_type: P::ID,
                packet_length: P::size_range().start as u16,
            },
            &vec![0u8; P::size_range().start + 1],
        ) {
            Err(TlvError::InvalidAdvertisedLength(advertised_length, actual_length)) => {
                assert_eq!(advertised_length, P::size_range().start);
                assert_eq!(actual_length, P::size_range().start + 1);
            }
            _ => unreachable!(),
        }
    }

    fn length_out_of_range<P: Packet>() {
        match tlv_from_bytes::<P>(
            &Header {
                packet_type: P::ID,
                packet_length: P::size_range().start as u16 - 1,
            },
            &vec![0u8; P::size_range().start - 1],
        ) {
            Err(TlvError::InvalidLength(length)) => assert_eq!(length, P::size_range().start - 1),
            _ => unreachable!(),
        }

        match tlv_from_bytes::<P>(
            &Header {
                packet_type: P::ID,
                packet_length: P::size_range().end as u16,
            },
            &vec![0u8; P::size_range().end],
        ) {
            Err(TlvError::InvalidLength(length)) => assert_eq!(length, P::size_range().end),
            _ => unreachable!(),
        }
    }

    fn fuzz<P: Packet>() {
        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            let length = rng.gen_range(P::size_range().start, P::size_range().end);
            let bytes_from: Vec<u8> = (0..length).map(|_| rand::random::<u8>()).collect();
            let packet = tlv_from_bytes::<P>(
                &Header {
                    packet_type: P::ID,
                    packet_length: length as u16,
                },
                &bytes_from,
            )
            .unwrap();
            let bytes_to = tlv_into_bytes(packet);

            assert_eq!(bytes_to[0], P::ID);
            assert_eq!(u16::from_le_bytes(bytes_to[1..3].try_into().unwrap()), length as u16);
            assert!(bytes_from.eq(&bytes_to[3..].to_vec()));
        }
    }

    macro_rules! implement_tlv_tests {
        ($type:ty, $iat:tt, $ial:tt, $loor:tt, $fuzz:tt) => {
            #[test]
            fn $iat() {
                invalid_advertised_type::<$type>();
            }

            #[test]
            fn $ial() {
                invalid_advertised_length::<$type>();
            }

            #[test]
            fn $loor() {
                length_out_of_range::<$type>();
            }

            #[test]
            fn $fuzz() {
                fuzz::<$type>();
            }
        };
    }

    implement_tlv_tests!(
        MilestoneRequest,
        invalid_advertised_type_milestone_request,
        invalid_advertised_length_milestone_request,
        length_out_of_range_milestone_request,
        fuzz_milestone_request
    );

    implement_tlv_tests!(
        MessagePacket,
        invalid_advertised_type_message,
        invalid_advertised_length_message,
        length_out_of_range_message,
        fuzz_message
    );

    implement_tlv_tests!(
        MessageRequest,
        invalid_advertised_type_message_request,
        invalid_advertised_length_message_request,
        length_out_of_range_message_request,
        fuzz_message_request
    );

    implement_tlv_tests!(
        Heartbeat,
        invalid_advertised_type_heartbeat,
        invalid_advertised_length_heartbeat,
        length_out_of_range_heartbeat,
        fuzz_range_heartbeat
    );
}
