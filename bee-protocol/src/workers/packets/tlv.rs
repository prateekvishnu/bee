// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Type-length-value encoding on top of the packets.

use crate::workers::packets::{HeaderPacket, Packet, HEADER_SIZE};

#[allow(clippy::enum_variant_names)]
#[allow(dead_code)] // TODO
#[derive(Debug)]
pub(crate) enum Error {
    InvalidAdvertisedType {
        found: u8,
        advertised: u8,
    },
    InvalidAdvertisedLength {
        type_id: u8,
        advertised: usize,
        found: usize,
    },
    InvalidLength {
        type_id: u8,
        len: usize,
    },
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
pub(crate) fn tlv_from_bytes<P: Packet>(header: &HeaderPacket, bytes: &[u8]) -> Result<P, Error> {
    if header.packet_type != P::ID {
        return Err(Error::InvalidAdvertisedType {
            found: P::ID,
            advertised: header.packet_type,
        });
    }

    if header.packet_length as usize != bytes.len() {
        return Err(Error::InvalidAdvertisedLength {
            type_id: header.packet_type,
            advertised: header.packet_length as usize,
            found: bytes.len(),
        });
    }

    if !P::size_range().contains(&bytes.len()) {
        return Err(Error::InvalidLength {
            type_id: header.packet_type,
            len: bytes.len(),
        });
    }

    Ok(P::from_bytes(bytes))
}

/// Serializes a TLV header and a packet to a byte buffer.
///
/// # Arguments
///
/// * `packet` -   The packet to serialize.
pub(crate) fn tlv_to_bytes<P: Packet>(packet: &P) -> Vec<u8> {
    let size = packet.size();
    let mut bytes = vec![0u8; HEADER_SIZE + size];
    let (header, payload) = bytes.split_at_mut(HEADER_SIZE);

    HeaderPacket {
        packet_type: P::ID,
        packet_length: size as u16,
    }
    .to_bytes(header);
    packet.to_bytes(payload);

    bytes
}

#[cfg(test)]
mod tests {

    use rand::Rng;

    use super::*;
    use crate::workers::packets::{
        HeartbeatPacket, MessagePacket, MessageRequestPacket, MilestoneRequestPacket, Packet,
    };

    fn invalid_advertised_type<P: Packet>() {
        match tlv_from_bytes::<P>(
            &HeaderPacket {
                packet_type: P::ID + 1,
                packet_length: P::size_range().start as u16,
            },
            &Vec::with_capacity(P::size_range().start),
        ) {
            Err(Error::InvalidAdvertisedType { advertised, found }) => {
                assert_eq!(advertised, P::ID + 1);
                assert_eq!(found, P::ID);
            }
            _ => unreachable!(),
        }
    }

    fn invalid_advertised_length<P: Packet>() {
        match tlv_from_bytes::<P>(
            &HeaderPacket {
                packet_type: P::ID,
                packet_length: P::size_range().start as u16,
            },
            &vec![0u8; P::size_range().start + 1],
        ) {
            Err(Error::InvalidAdvertisedLength {
                type_id,
                advertised,
                found,
            }) => {
                assert_eq!(type_id, P::ID);
                assert_eq!(advertised, P::size_range().start);
                assert_eq!(found, P::size_range().start + 1);
            }
            _ => unreachable!(),
        }
    }

    fn length_out_of_range<P: Packet>() {
        match tlv_from_bytes::<P>(
            &HeaderPacket {
                packet_type: P::ID,
                packet_length: P::size_range().start as u16 - 1,
            },
            &vec![0u8; P::size_range().start - 1],
        ) {
            Err(Error::InvalidLength { type_id, len }) => {
                assert_eq!(type_id, P::ID);
                assert_eq!(len, P::size_range().start - 1);
            }
            _ => unreachable!(),
        }

        match tlv_from_bytes::<P>(
            &HeaderPacket {
                packet_type: P::ID,
                packet_length: P::size_range().end as u16,
            },
            &vec![0u8; P::size_range().end],
        ) {
            Err(Error::InvalidLength { type_id, len }) => {
                assert_eq!(type_id, P::ID);
                assert_eq!(len, P::size_range().end);
            }
            _ => unreachable!(),
        }
    }

    fn fuzz<P: Packet>() {
        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            let length = rng.gen_range(P::size_range());
            let bytes_from: Vec<u8> = (0..length).map(|_| rand::random::<u8>()).collect();
            let packet = tlv_from_bytes::<P>(
                &HeaderPacket {
                    packet_type: P::ID,
                    packet_length: length as u16,
                },
                &bytes_from,
            )
            .unwrap();
            let bytes_to = tlv_to_bytes(&packet);

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
        MilestoneRequestPacket,
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
        MessageRequestPacket,
        invalid_advertised_type_message_request,
        invalid_advertised_length_message_request,
        length_out_of_range_message_request,
        fuzz_message_request
    );

    implement_tlv_tests!(
        HeartbeatPacket,
        invalid_advertised_type_heartbeat,
        invalid_advertised_length_heartbeat,
        length_out_of_range_heartbeat,
        fuzz_range_heartbeat
    );
}
