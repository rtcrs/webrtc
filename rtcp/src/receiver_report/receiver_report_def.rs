use std::io::Write;
use std::{fmt, vec};

use byteorder::{BigEndian, ByteOrder, WriteBytesExt};

use bytes::BytesMut;
use util::Error;

use crate::reception_report::ReceptionReport;
use crate::{
    header, header::Header, header::PacketType, receiver_report, reception_report,
    util::get_padding,
};

// A ReceiverReport (RR) packet provides reception quality feedback for an RTP stream
#[derive(Debug, PartialEq, Default, Clone)]
pub struct ReceiverReport {
    // The synchronization source identifier for the originator of this RR packet.
    pub ssrc: u32,
    // Zero or more reception report blocks depending on the number of other
    // sources heard by this sender since the last report. Each reception report
    // block conveys statistics on the reception of RTP packets from a
    // single synchronization source.
    pub reports: Vec<ReceptionReport>,
    // Extension contains additional, payload-specific information that needs to
    // be reported regularly about the receiver.
    pub profile_extensions: Vec<u8>,
}

impl fmt::Display for ReceiverReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = format!("ReceiverReport from {}\n", self.ssrc);
        out += "\tSSRC    \tLost\tLastSequence\n";
        for rep in &self.reports {
            out += format!(
                "\t{:x}\t{}/{}\t{}\n",
                rep.ssrc, rep.fraction_lost, rep.total_lost, rep.last_sequence_number
            )
            .as_str();
        }
        out += format!("\tProfile Extension Data: {:?}\n", self.profile_extensions).as_str();

        write!(f, "{}", out)
    }
}

impl ReceiverReport {
    fn len(&self) -> usize {
        let mut reps_length = 0;
        for rep in &self.reports {
            reps_length += rep.size();
        }
        header::HEADER_LENGTH + header::SSRC_LENGTH + reps_length + self.profile_extensions.len()
    }
    // Unmarshal decodes the ReceiverReport from binary
    pub fn unmarshal(&self, raw_packet: &mut BytesMut) -> Result<(), Error> {
        /*
         *         0                   1                   2                   3
         *         0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
         *        +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
         * header |V=2|P|    RC   |   PT=RR=201   |             length            |
         *        +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
         *        |                     SSRC of packet sender                     |
         *        +=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+
         * report |                 SSRC_1 (SSRC of first source)                 |
         * block  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
         *   1    | fraction lost |       cumulative number of packets lost       |
         *        +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
         *        |           extended highest sequence number received           |
         *        +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
         *        |                      interarrival jitter                      |
         *        +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
         *        |                         last SR (LSR)                         |
         *        +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
         *        |                   delay since last SR (DLSR)                  |
         *        +=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+
         * report |                 SSRC_2 (SSRC of second source)                |
         * block  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
         *   2    :                               ...                             :
         *        +=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+
         *        |                  profile-specific extensions                  |
         *        +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
         */

        if raw_packet.len() < (header::HEADER_LENGTH + receiver_report::SSRC_LENGTH) {
            return Err(Error::new("packet too short".to_string()));
        }

        let mut header = Header::default();
        header.unmarshal(&mut raw_packet)?;

        if header.packet_type != header::PacketType::ReceiverReport {
            return Err(Error::new("wrong packet type".to_string()));
        }

        self.ssrc = BigEndian::read_u32(&raw_packet[super::RR_SSRC_OFFSET..]);

        let mut i = super::RR_REPORT_OFFSET;

        while i < raw_packet.len() && self.reports.len() < header.count as usize {
            let mut rr = reception_report::ReceptionReport::default();

            rr.unmarshal(&mut raw_packet[i..].into())?;

            self.reports.push(rr);
            i += reception_report::RECEPTION_REPORT_LENGTH;
        }

        self.profile_extensions = raw_packet[super::RR_REPORT_OFFSET
            + (self.reports.len() * reception_report::RECEPTION_REPORT_LENGTH)..]
            .to_vec();

        if self.reports.len() != header.count as usize {
            return Err(Error::new("invalid header".to_string()));
        }

        Ok(())
    }

    // Header returns the Header associated with this packet.
    pub fn header(&self) -> Header {
        let l = self.len() + get_padding(self.len());
        Header {
            padding: false,
            count: self.reports.len() as u8,
            packet_type: PacketType::ReceiverReport,
            length: ((l / 4) - 1) as u16,
        }
    }

    // destination_ssrc returns an array of SSRC values that this packet refers to.
    pub fn destination_ssrc(&self) -> Vec<u32> {
        self.reports.iter().map(|x| x.ssrc).collect()
    }

    // Marshal encodes the packet in binary.
    pub fn marshal(&self) -> Result<BytesMut, Error> {
        /*
         *         0                   1                   2                   3
         *         0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
         *        +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
         * header |V=2|P|    RC   |   PT=RR=201   |             length            |
         *        +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
         *        |                     SSRC of packet sender                     |
         *        +=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+
         * report |                 SSRC_1 (SSRC of first source)                 |
         * block  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
         *   1    | fraction lost |       cumulative number of packets lost       |
         *        +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
         *        |           extended highest sequence number received           |
         *        +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
         *        |                      interarrival jitter                      |
         *        +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
         *        |                         last SR (LSR)                         |
         *        +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
         *        |                   delay since last SR (DLSR)                  |
         *        +=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+
         * report |                 SSRC_2 (SSRC of second source)                |
         * block  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
         *   2    :                               ...                             :
         *        +=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+=+
         *        |                  profile-specific extensions                  |
         *        +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
         */

        let mut raw_packet = vec![0u8; self.len()];
        let mut packet_body = &raw_packet[header::HEADER_LENGTH..];

        BigEndian::write_u32(&mut packet_body, self.ssrc);

        for i in 0..self.reports.len() {
            let data = self.reports[i].marshal()?;

            let offset =
                receiver_report::SSRC_LENGTH + (reception_report::RECEPTION_REPORT_LENGTH * i);

            packet_body[offset..offset + data.len()].copy_from_slice(&data);
        }

        if self.reports.len() > header::COUNT_MAX {
            return Err(Error::new("too many reports".to_string()));
        }

        let mut pe = vec![0u8; self.profile_extensions.len()];
        pe.copy_from_slice(&self.profile_extensions);

        // if the length of the profile extensions isn't devisible
        // by 4, we need to pad the end.
        while (pe.len() & 0x3) != 0 {
            pe.push(0);
        }

        raw_packet.append(&mut pe);

        let header_data = self.header().marshal()?;

        raw_packet[..header_data.len()].copy_from_slice(&header_data);

        Ok(raw_packet[..].into())
    }
}