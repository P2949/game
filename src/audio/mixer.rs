use std::sync::Arc;

use crossbeam_queue::ArrayQueue;
use sdl3::audio::{AudioCallback, AudioFormat, AudioSpec, AudioStream, AudioStreamWithCallback};

const AUDIO_SCRATCH_SAMPLES: usize = 4096;
const AUDIO_COMMAND_QUEUE_CAPACITY: usize = 128;
const MAX_VOICES: usize = 32;

#[derive(Debug, Clone, Copy)]
enum AudioCommand {
    Play {
        sound_id: usize,
        volume: f32,
        looping: bool,
    },
}

pub struct Sound {
    pub samples: Vec<f32>,
    pub channels: u16,
    pub sample_rate: u32,
}

pub struct Voice {
    pub sound_id: usize,
    pub cursor: usize,
    pub volume: f32,
    pub looping: bool,
}

pub struct Mixer {
    pub sounds: Vec<Sound>,
    pub voices: Vec<Voice>,
    pub master_volume: f32,
    // Expected playback-stream output format (0 means "unspecified / don't
    // check"). The mixer plays sample data verbatim — it does not resample or
    // remap channels — so a sound that doesn't match would play at the wrong
    // speed/pitch. `add_sound` warns on a mismatch.
    output_channels: u16,
    output_sample_rate: u32,
}

impl Mixer {
    pub fn new() -> Self {
        Self {
            sounds: Vec::new(),
            voices: Vec::with_capacity(MAX_VOICES),
            master_volume: 1.0,
            output_channels: 0,
            output_sample_rate: 0,
        }
    }

    /// Declares the output stream's channel count and sample rate so `add_sound`
    /// can flag sounds that don't match. The default (0/0) disables the check.
    pub fn set_output_format(&mut self, channels: u16, sample_rate: u32) {
        self.output_channels = channels;
        self.output_sample_rate = sample_rate;
    }

    /// Whether `sound` matches the configured output format. Always true when the
    /// output format is unspecified.
    pub fn sound_format_matches(&self, sound: &Sound) -> bool {
        self.output_channels == 0
            || (sound.channels == self.output_channels
                && sound.sample_rate == self.output_sample_rate)
    }

    pub fn add_sound(&mut self, sound: Sound) -> usize {
        if !self.sound_format_matches(&sound) {
            log::warn!(
                "adding sound ({}ch/{}Hz) that does not match mixer output ({}ch/{}Hz); \
                 it will play at the wrong speed/pitch (the mixer does not resample)",
                sound.channels,
                sound.sample_rate,
                self.output_channels,
                self.output_sample_rate,
            );
        }

        let id = self.sounds.len();
        self.sounds.push(sound);
        id
    }

    pub fn play(&mut self, sound_id: usize, volume: f32, looping: bool) {
        let Some(sound) = self.sounds.get(sound_id) else {
            return;
        };

        if sound.samples.is_empty() {
            return;
        }

        if self.voices.len() >= MAX_VOICES {
            return;
        }

        self.voices.push(Voice {
            sound_id,
            cursor: 0,
            volume: sanitize_volume(volume),
            looping,
        });
    }

    pub fn mix_into(&mut self, out: &mut [f32]) {
        out.fill(0.0);

        // Sanitize once per mix so a poisoned `master_volume` field (set directly
        // on this public struct) can never produce invalid or exploding gain.
        let master_volume = sanitize_volume(self.master_volume);

        for voice in &mut self.voices {
            let sound = &self.sounds[voice.sound_id];
            for sample in out.iter_mut() {
                if voice.cursor >= sound.samples.len() {
                    if voice.looping {
                        voice.cursor = 0;
                    } else {
                        break;
                    }
                }

                *sample += sound.samples[voice.cursor] * voice.volume * master_volume;
                voice.cursor += 1;
            }
        }

        for sample in out.iter_mut() {
            *sample = sample.clamp(-1.0, 1.0);
        }

        self.voices.retain(|voice| {
            voice.looping || voice.cursor < self.sounds[voice.sound_id].samples.len()
        });
    }
}

impl Default for Mixer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AudioSystem {
    commands: Arc<ArrayQueue<AudioCommand>>,
    blip_sound: usize,
    _stream: AudioStreamWithCallback<MixerCallback>,
}

impl AudioSystem {
    pub fn new(sdl: &sdl3::Sdl) -> anyhow::Result<Self> {
        let sample_rate = 48_000;
        let channels = 2;
        let spec = AudioSpec {
            freq: Some(sample_rate),
            channels: Some(channels),
            format: Some(AudioFormat::f32_sys()),
        };

        let mut mixer = Mixer::new();
        mixer.master_volume = 0.3;
        mixer.set_output_format(channels as u16, sample_rate as u32);
        let blip_sound =
            mixer.add_sound(sine_sound(660.0, 0.12, sample_rate as u32, channels as u16));
        let music_sound =
            mixer.add_sound(sine_sound(110.0, 1.0, sample_rate as u32, channels as u16));
        mixer.play(music_sound, 0.08, true);

        let commands = Arc::new(ArrayQueue::new(AUDIO_COMMAND_QUEUE_CAPACITY));
        let audio = sdl.audio().map_err(anyhow::Error::msg)?;
        let stream = audio
            .open_playback_stream(
                &spec,
                MixerCallback {
                    mixer,
                    commands: Arc::clone(&commands),
                    scratch: vec![0.0; AUDIO_SCRATCH_SAMPLES],
                    put_failures: 0,
                },
            )
            .map_err(anyhow::Error::msg)?;
        stream.resume().map_err(anyhow::Error::msg)?;

        log::info!("started SDL audio mixer at {sample_rate} Hz, {channels} channels");

        Ok(Self {
            commands,
            blip_sound,
            _stream: stream,
        })
    }

    pub fn play_blip(&self) {
        let command = AudioCommand::Play {
            sound_id: self.blip_sound,
            volume: 0.8,
            looping: false,
        };

        if self.commands.push(command).is_err() {
            log::warn!("dropping audio command because the queue is full");
        }
    }
}

struct MixerCallback {
    mixer: Mixer,
    commands: Arc<ArrayQueue<AudioCommand>>,
    scratch: Vec<f32>,
    // Counts `put_data_f32` failures so a silently-dropping stream produces some
    // signal. Read inside the (throttled) warning in `callback`.
    put_failures: u64,
}

impl MixerCallback {
    fn drain_commands(&mut self) {
        while let Some(command) = self.commands.pop() {
            match command {
                AudioCommand::Play {
                    sound_id,
                    volume,
                    looping,
                } => {
                    self.mixer.play(sound_id, volume, looping);
                }
            }
        }
    }
}

impl AudioCallback<f32> for MixerCallback {
    fn callback(&mut self, stream: &mut AudioStream, requested: i32) {
        self.drain_commands();

        // `requested` is already a per-`Channel` sample count, NOT a byte count:
        // the sdl3 callback shim divides SDL's byte amount by `size_of::<Channel>()`
        // before calling us (sdl3 audio.rs: `len / size_of::<Channel>()`). Do not
        // add a bytes->samples conversion here. `put_data_f32` below likewise
        // consumes f32 samples, keeping the whole path in sample units.
        let mut remaining = requested.max(0) as usize;

        while remaining > 0 {
            let chunk_len = remaining.min(self.scratch.len());
            let out = &mut self.scratch[..chunk_len];

            self.mixer.mix_into(out);
            if stream.put_data_f32(out).is_err() {
                self.put_failures += 1;
                // Logging from an audio callback is normally avoided, but a rare,
                // throttled warning (first failure, then every 1000th) is worth
                // the signal if audio output silently stops.
                if self.put_failures == 1 || self.put_failures.is_multiple_of(1000) {
                    log::warn!(
                        "audio stream put_data_f32 failed ({} total); output may be dropping",
                        self.put_failures
                    );
                }
            }

            remaining -= chunk_len;
        }
    }
}

/// Clamps a gain multiplier into the valid `0.0..=1.0` range, mapping any
/// non-finite value (NaN/inf) to silence. Keeps the mixer from producing
/// invalid or runaway gain from public volume inputs.
fn sanitize_volume(volume: f32) -> f32 {
    if volume.is_finite() {
        volume.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn sine_sound(freq: f32, seconds: f32, sample_rate: u32, channels: u16) -> Sound {
    let frames = (seconds * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(frames * channels as usize);

    for frame in 0..frames {
        let t = frame as f32 / sample_rate as f32;
        let envelope = 1.0 - frame as f32 / frames.max(1) as f32;
        let sample = (t * freq * std::f32::consts::TAU).sin() * envelope;
        for _ in 0..channels {
            samples.push(sample);
        }
    }

    Sound {
        samples,
        channels,
        sample_rate,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crossbeam_queue::ArrayQueue;

    use super::{AudioCommand, Mixer, MixerCallback, Sound, sanitize_volume};

    #[test]
    fn sanitize_volume_clamps_and_neutralizes_non_finite() {
        assert_eq!(sanitize_volume(0.5), 0.5);
        assert_eq!(sanitize_volume(0.0), 0.0);
        assert_eq!(sanitize_volume(1.0), 1.0);
        assert_eq!(sanitize_volume(-1.0), 0.0);
        assert_eq!(sanitize_volume(2.0), 1.0);
        assert_eq!(sanitize_volume(f32::NAN), 0.0);
        assert_eq!(sanitize_volume(f32::INFINITY), 0.0);
        assert_eq!(sanitize_volume(f32::NEG_INFINITY), 0.0);
    }

    #[test]
    fn sound_format_matches_respects_configured_output() {
        let mut mixer = Mixer::new();
        let matching = Sound {
            samples: vec![0.0],
            channels: 2,
            sample_rate: 48_000,
        };
        let mismatched = Sound {
            samples: vec![0.0],
            channels: 1,
            sample_rate: 44_100,
        };

        // Unspecified output format accepts anything.
        assert!(mixer.sound_format_matches(&matching));
        assert!(mixer.sound_format_matches(&mismatched));

        mixer.set_output_format(2, 48_000);
        assert!(mixer.sound_format_matches(&matching));
        assert!(!mixer.sound_format_matches(&mismatched));
    }

    #[test]
    fn play_sanitizes_voice_volume() {
        let mut mixer = Mixer::new();
        let sound_id = mixer.add_sound(Sound {
            samples: vec![1.0],
            channels: 1,
            sample_rate: 48_000,
        });

        mixer.play(sound_id, f32::NAN, true);

        assert_eq!(mixer.voices.len(), 1);
        assert_eq!(mixer.voices[0].volume, 0.0);
    }

    #[test]
    fn mix_into_ignores_non_finite_master_volume() {
        let mut mixer = Mixer::new();
        mixer.master_volume = f32::NAN;
        let sound_id = mixer.add_sound(Sound {
            samples: vec![0.5, 0.5],
            channels: 1,
            sample_rate: 48_000,
        });
        mixer.play(sound_id, 1.0, false);

        let mut out = [0.0; 2];
        mixer.mix_into(&mut out);

        // NaN master volume is neutralized to 0.0 gain, not propagated into output.
        assert_eq!(&out, &[0.0, 0.0]);
    }

    #[test]
    fn mixer_preallocates_voice_capacity() {
        let mixer = Mixer::new();

        assert!(mixer.voices.capacity() >= super::MAX_VOICES);
    }

    #[test]
    fn finished_voice_is_removed() {
        let mut mixer = Mixer::new();
        let sound_id = mixer.add_sound(Sound {
            samples: vec![0.5, 0.5],
            channels: 1,
            sample_rate: 48_000,
        });
        mixer.play(sound_id, 1.0, false);

        let mut out = [0.0; 4];
        mixer.mix_into(&mut out);

        assert_eq!(&out, &[0.5, 0.5, 0.0, 0.0]);
        assert!(mixer.voices.is_empty());
    }

    #[test]
    fn overlapping_voices_sum_and_clamp() {
        let mut mixer = Mixer::new();
        let sound_id = mixer.add_sound(Sound {
            samples: vec![0.75],
            channels: 1,
            sample_rate: 48_000,
        });
        mixer.play(sound_id, 1.0, true);
        mixer.play(sound_id, 1.0, true);

        let mut out = [0.0; 1];
        mixer.mix_into(&mut out);

        assert_eq!(out[0], 1.0);
    }

    #[test]
    fn looping_music_produces_nonzero_samples() {
        let sample_rate = 48_000;
        let channels = 2;

        let mut mixer = Mixer::new();
        mixer.master_volume = 0.3;

        let music_sound = mixer.add_sound(super::sine_sound(110.0, 1.0, sample_rate, channels));
        mixer.play(music_sound, 0.08, true);

        let mut out = vec![0.0; 1024];
        mixer.mix_into(&mut out);

        assert!(
            out.iter().any(|sample| sample.abs() > 0.0001),
            "looping music mixed only silence"
        );
    }

    #[test]
    fn empty_sound_is_not_played() {
        let mut mixer = Mixer::new();
        let sound_id = mixer.add_sound(Sound {
            samples: Vec::new(),
            channels: 1,
            sample_rate: 48_000,
        });

        mixer.play(sound_id, 1.0, true);

        assert!(mixer.voices.is_empty());
    }

    #[test]
    fn invalid_sound_id_is_ignored() {
        let mut mixer = Mixer::new();

        mixer.play(123, 1.0, false);

        assert!(mixer.voices.is_empty());
    }

    #[test]
    fn play_drops_new_voice_when_voice_cap_is_reached() {
        let mut mixer = Mixer::new();
        let sound_id = mixer.add_sound(Sound {
            samples: vec![0.25],
            channels: 1,
            sample_rate: 48_000,
        });

        for _ in 0..super::MAX_VOICES {
            mixer.play(sound_id, 1.0, true);
        }

        assert_eq!(mixer.voices.len(), super::MAX_VOICES);

        mixer.play(sound_id, 1.0, true);

        assert_eq!(mixer.voices.len(), super::MAX_VOICES);
    }

    #[test]
    fn callback_drains_play_commands() {
        let mut mixer = Mixer::new();
        let sound_id = mixer.add_sound(Sound {
            samples: vec![0.25],
            channels: 1,
            sample_rate: 48_000,
        });
        let commands = Arc::new(ArrayQueue::new(4));
        commands
            .push(AudioCommand::Play {
                sound_id,
                volume: 0.5,
                looping: false,
            })
            .unwrap();
        let mut callback = MixerCallback {
            mixer,
            commands: Arc::clone(&commands),
            scratch: vec![0.0; super::AUDIO_SCRATCH_SAMPLES],
            put_failures: 0,
        };

        callback.drain_commands();

        assert!(commands.is_empty());
        assert_eq!(callback.mixer.voices.len(), 1);
        assert_eq!(callback.mixer.voices[0].sound_id, sound_id);
        assert_eq!(callback.mixer.voices[0].volume, 0.5);
    }

    #[test]
    fn callback_drained_play_commands_respect_voice_cap() {
        let mut mixer = Mixer::new();
        let sound_id = mixer.add_sound(Sound {
            samples: vec![0.25],
            channels: 1,
            sample_rate: 48_000,
        });

        let commands = Arc::new(ArrayQueue::new(super::MAX_VOICES + 8));
        for _ in 0..(super::MAX_VOICES + 8) {
            commands
                .push(AudioCommand::Play {
                    sound_id,
                    volume: 0.5,
                    looping: true,
                })
                .unwrap();
        }

        let mut callback = MixerCallback {
            mixer,
            commands: Arc::clone(&commands),
            scratch: vec![0.0; super::AUDIO_SCRATCH_SAMPLES],
            put_failures: 0,
        };

        callback.drain_commands();

        assert!(commands.is_empty());
        assert_eq!(callback.mixer.voices.len(), super::MAX_VOICES);
        assert!(callback.mixer.voices.capacity() >= super::MAX_VOICES);
    }
}
