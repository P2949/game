use super::{SoundId, StreamId};

pub(super) const AUDIO_COMMAND_QUEUE_CAPACITY: usize = 128;

#[derive(Debug, Clone, Copy)]
pub(super) enum AudioCommand {
    Play {
        sound_id: SoundId,
        volume: f32,
        looping: bool,
        bus: Option<u8>,
    },
    PlayMusic {
        sound_id: SoundId,
        volume: f32,
        fade_in_seconds: Option<f32>,
    },
    PlayStreamedMusic {
        stream_id: StreamId,
        volume: f32,
        fade_in_seconds: Option<f32>,
    },
    CrossfadeMusic {
        sound_id: SoundId,
        volume: f32,
        duration_seconds: f32,
    },
    CrossfadeStreamedMusic {
        stream_id: StreamId,
        volume: f32,
        duration_seconds: f32,
    },
    StopMusic,
    PauseMusic,
    ResumeMusic,
    SetMasterVolume(f32),
    SetSfxVolume(f32),
    SetMusicVolume(f32),
    SetBusVolume {
        bus: u8,
        volume: f32,
    },
    FadeMusicTo {
        volume: f32,
        duration_seconds: f32,
    },
}
