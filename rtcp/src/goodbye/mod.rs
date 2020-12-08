use std::fmt;
use std::io::{Read, Write};

use bytes::BytesMut;
use util::Error;

use byteorder::{BigEndian, ByteOrder, ReadBytesExt, WriteBytesExt};

use super::errors::*;
use super::header::{Header, PacketType};
use crate::{header, util::get_padding};

#[cfg(test)]
mod goodbye_test;

// The Goodbye packet indicates that one or more sources are no longer active.
#[derive(Debug, PartialEq, Default, Clone)]
pub struct Goodbye {
    // The SSRC/CSRC identifiers that are no longer active
    pub sources: Vec<u32>,
    // Optional text indicating the reason for leaving, e.g., "camera malfunction" or "RTP loop detected"
    pub reason: String,
}

impl fmt::Display for Goodbye {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = "Goodbye:\n\tSources:\n".to_string();
        for s in &self.sources {
            out += format!("\t{}\n", *s).as_str();
        }
        out += format!("\tReason: {:?}\n", self.reason).as_str();

        write!(f, "{}", out)
    }
}

impl Goodbye {
    pub fn unmarshal(&mut self, raw_packet: &mut BytesMut) -> Result<(), Error> {
        /*
         *        0                   1                   2                   3
         *        0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
         *       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
         *       |V=2|P|    SC   |   PT=BYE=203  |             length            |
         *       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
         *       |                           SSRC/CSRC                           |
         *       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
         *       :                              ...                              :
         *       +=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+
         * (opt) |     length    |               reason for leaving            ...
         *       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
         */

        let header = Header::default();
        header.unmarshal(&mut raw_packet)?;

        if header.packet_type != PacketType::Goodbye {
            return Err(ERR_WRONG_TYPE.clone());
        }

        if get_padding(raw_packet.len()) != 0 {
            return Err(Error::new("packet too short".to_string()));
        }

        self.sources = vec![0u32; header.count as usize];

        let reason_offset =
            (header::HEADER_LENGTH + header.count as usize * header::SSRC_LENGTH) as usize;

        if reason_offset > raw_packet.len() {
            return Err(Error::new("packet too short".to_string()));
        }

        for i in 0..header.count as usize {
            let offset = header::HEADER_LENGTH + i * header::SSRC_LENGTH;

            self.sources[i] = BigEndian::read_u32(&raw_packet[offset..]);
        }

        if reason_offset < raw_packet.len() {
            let reason_len = raw_packet[reason_offset] as usize;
            let reason_end = reason_offset + 1 + reason_len;

            if reason_end > raw_packet.len() {
                return Err(Error::new("packet too short".to_string()));
            }

            self.reason =
                match String::from_utf8(raw_packet[reason_offset + 1..reason_end].to_vec()) {
                    Ok(e) => e,

                    Err(e) => {
                        return Err(Error::new("error converting byte to string".to_string()));
                    }
                };
        }

        Ok(())
    }

    // Header returns the Header associated with this packet.
    pub fn header(&self) -> Header {
        Header {
            padding: false,
            count: self.sources.len() as u8,
            packet_type: PacketType::Goodbye,
            length: ((self.len() / 4) - 1) as u16,
        }
    }

    fn len(&self) -> usize {
        let srcs_length = self.sources.len() * header::SSRC_LENGTH;
        let reason_length = self.reason.len() + 1;

        let l = header::HEADER_LENGTH + srcs_length + reason_length;

        // align to 32-bit boundary
        return l + get_padding(l);
    }

    // destination_ssrc returns an array of SSRC values that this packet refers to.
    pub fn destination_ssrc(&self) -> Vec<u32> {
        self.sources.to_vec()
    }

    // Marshal encodes the packet in binary.
    pub fn marshal(&self) -> Result<BytesMut, Error> {
        /*
         *        0                   1                   2                   3
         *        0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
         *       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
         *       |V=2|P|    SC   |   PT=BYE=203  |             length            |
         *       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
         *       |                           SSRC/CSRC                           |
         *       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
         *       :                              ...                              :
         *       +=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+
         * (opt) |     length    |               reason for leaving            ...
         *       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
         */

        if self.sources.len() > header::COUNT_MAX {
            return Err(ERR_TOO_MANY_SOURCES.clone());
        }

        let mut raw_packet = vec![0u8; self.len()];
        let mut packet_body = &raw_packet[header::HEADER_LENGTH..];

        if self.sources.len() > header::COUNT_MAX {
            return Err(Error::new("too many sources".to_string()));
        }

        for i in 0..self.sources.len() {
            BigEndian::write_u32(&mut packet_body[i * header::SSRC_LENGTH..], self.sources[i]);
        }

        if self.reason != "" {
            let reason = self.reason.as_bytes();

            if reason.len() > header::SDES_MAX_OCTET_COUNT {
                return Err(Error::new("reason too long".to_string()));
            }

            let reason_offset = self.sources.len() * header::SSRC_LENGTH;

            packet_body[reason_offset] = reason.len() as u8;

            let n = reason_offset + 1;

            packet_body[n..n + reason.len()].copy_from_slice(&reason);
        }

        let header_data = self.header().marshal()?;

        raw_packet[..header_data.len()].copy_from_slice(&header_data);

        Ok(raw_packet[..].into())
    }
}