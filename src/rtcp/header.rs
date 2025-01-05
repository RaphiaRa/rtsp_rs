use std::io;

pub enum PacketType {
    Unknown = 0,
    SenderReport = 200,
    ReceiverReport = 201,
    SourceDescription = 202,
    Goodbye = 203,
    ApplicationDefined = 204,
    TransportLayerFeedback = 205,
    PayloadSpecificFeedback = 206,
    ExtendedReport = 207,
}

pub type Version = u8;

pub struct Header<'a> {
    buf: &'a [u8],
}

/// RTCP Common Header
/// All RTCP packets MUST start with a fixed header that contains the following fields:
/// - version (V): 2 bits
/// - padding (P): 1 bit
/// - report count (RC): 5 bits
/// - packet type (PT): 8 bits
/// - length: 16 bits
impl<'a> Header<'a> {
    pub fn new(buf: &'a [u8]) -> Result<Self, io::Error> {
        if buf.len() < 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid RTCP Data",
            ));
        }
        Ok(Self { buf })
    }
    pub fn version(&self) -> u8 {
        self.buf[0] >> 6
    }

    pub fn padding(&self) -> bool {
        self.buf[0] & 0x20 != 0
    }

    pub fn count(&self) -> usize {
        (self.buf[0] & 0x1F) as usize
    }

    pub fn packet_type(&self) -> PacketType {
        match self.buf[1] {
            200 => PacketType::SenderReport,
            201 => PacketType::ReceiverReport,
            202 => PacketType::SourceDescription,
            203 => PacketType::Goodbye,
            204 => PacketType::ApplicationDefined,
            205 => PacketType::TransportLayerFeedback,
            206 => PacketType::PayloadSpecificFeedback,
            207 => PacketType::ExtendedReport,
            _ => PacketType::Unknown,
        }
    }

    pub fn length(&self) -> usize {
        u16::from_be_bytes([self.buf[2], self.buf[3]]) as usize
    }
}
