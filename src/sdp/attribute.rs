enum Codec {
    H264,
    H265,
    AAC,
    PCMU,
    PCMA,
    OPUS,
    Unknown(String),
}

struct RtpMap {
    payload_type: u8,
    codec: Codec,
    timebase: u32,
}

struct Fmtp {
    payload_type : u8,
}
