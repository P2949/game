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
    #[allow(dead_code)]
    pub channels: u16,
    #[allow(dead_code)]
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
}

impl Mixer {
    pub fn new() -> Self {
        Self {
            sounds: Vec::new(),
            voices: Vec::with_capacity(MAX_VOICES),
            master_volume: 1.0,
        }
    }

    pub fn add_sound(&mut self, sound: Sound) -> usize {
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
            volume,
            looping,
        });
    }

    pub fn mix_into(&mut self, out: &mut [f32]) {
        out.fill(0.0);

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

                *sample += sound.samples[voice.cursor] * voice.volume * self.master_volume;
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

        let mut remaining = requested.max(0) as usize;

        while remaining > 0 {
            let chunk_len = remaining.min(self.scratch.len());
            let out = &mut self.scratch[..chunk_len];

            self.mixer.mix_into(out);
            let _ = stream.put_data_f32(out);

            remaining -= chunk_len;
        }
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

    use super::{AudioCommand, Mixer, MixerCallback, Sound};

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
        };

        callback.drain_commands();

        assert!(commands.is_empty());
        assert_eq!(callback.mixer.voices.len(), super::MAX_VOICES);
        assert!(callback.mixer.voices.capacity() >= super::MAX_VOICES);
    }
}
