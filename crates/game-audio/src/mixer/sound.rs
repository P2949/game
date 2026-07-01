/// Stable index for a loaded/generated sound in the mixer.
pub type SoundId = usize;

/// Outcome of [`super::Mixer::play`]. Returned so callers (and tests) can
/// distinguish a started voice from the two silent-drop cases instead of
/// guessing from voice counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayResult {
    Started,
    DroppedVoiceLimit,
    InvalidSoundId,
}

#[derive(Debug)]
pub struct Sound {
    pub(crate) samples: Vec<f32>,
    pub(crate) channels: u16,
    pub(crate) sample_rate: u32,
}

impl Sound {
    pub fn new(samples: Vec<f32>, channels: u16, sample_rate: u32) -> Self {
        Self {
            samples,
            channels,
            sample_rate,
        }
    }
}

pub(crate) fn sanitize_volume(volume: f32) -> f32 {
    if volume.is_finite() {
        volume.clamp(0.0, 1.0)
    } else {
        0.0
    }
}
