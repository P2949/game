use super::{SoundId, StreamId};

#[derive(Debug, Clone, Copy)]
pub(crate) struct MusicFade {
    pub(crate) from: f32,
    pub(crate) to: f32,
    pub(crate) elapsed_seconds: f32,
    pub(crate) duration_seconds: f32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct VoiceFade {
    pub(crate) from: f32,
    pub(crate) to: f32,
    pub(crate) elapsed_seconds: f32,
    pub(crate) duration_seconds: f32,
}

// Static voices are created only after sound-id validation; streamed music uses
// a separately validated bounded-reader source.
pub(crate) struct Voice {
    pub(crate) source: VoiceSource,
    pub(crate) cursor: usize,
    pub(crate) volume: f32,
    pub(crate) looping: bool,
    pub(crate) music: bool,
    pub(crate) bus: Option<u8>,
    pub(crate) fade_volume: f32,
    pub(crate) fade: Option<VoiceFade>,
    pub(crate) remove_when_faded: bool,
}

pub(crate) enum VoiceSource {
    Static(SoundId),
    Streamed {
        stream_id: StreamId,
        generation: u64,
    },
}
