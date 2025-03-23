
pub enum MediaType {
    Video,
    Audio,
}
pub enum FrameType {
    H264,
    H265,
    AAC,
    Opus,
    G711,
    G722,
    G729,
    PCMU,
    PCMA,
    VP8,
    VP9,
    AV1,
    JPEG,
}
pub struct Frame {
    pub media_type: MediaType,
    pub frame_type: FrameType,
    pub data: Vec<u8>,
}

pub trait AsyncReadFrame {
    async fn read_frame<Stream: AsyncReadExt + Unpin>(
        &mut self,
        stream: &mut Stream,
    ) -> Result<Frame>;
}
