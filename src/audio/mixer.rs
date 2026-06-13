use std::sync::{Arc, Mutex};

use sdl3::audio::{AudioCallback, AudioFormat, AudioSpec, AudioStream, AudioStreamWithCallback};

pub type SharedMixer = Arc<Mutex<Mixer>>;

const AUDIO_SCRATCH_SAMPLES: usize = 4096;

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
            voices: Vec::new(),
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
    pub mixer: SharedMixer,
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

        let mixer = Arc::new(Mutex::new(mixer));
        let audio = sdl.audio().map_err(anyhow::Error::msg)?;
        let stream = audio
            .open_playback_stream(
                &spec,
                MixerCallback {
                    mixer: Arc::clone(&mixer),
                    scratch: vec![0.0; AUDIO_SCRATCH_SAMPLES],
                },
            )
            .map_err(anyhow::Error::msg)?;
        stream.resume().map_err(anyhow::Error::msg)?;

        log::info!("started SDL audio mixer at {sample_rate} Hz, {channels} channels");

        Ok(Self {
            mixer,
            blip_sound,
            _stream: stream,
        })
    }

    pub fn play_blip(&self) {
        if let Ok(mut mixer) = self.mixer.lock() {
            mixer.play(self.blip_sound, 0.8, false);
        }
    }
}

struct MixerCallback {
    mixer: SharedMixer,
    scratch: Vec<f32>,
}

impl AudioCallback<f32> for MixerCallback {
    fn callback(&mut self, stream: &mut AudioStream, requested: i32) {
        let mut remaining = requested.max(0) as usize;

        if let Ok(mut mixer) = self.mixer.try_lock() {
            while remaining > 0 {
                let chunk_len = remaining.min(self.scratch.len());
                let out = &mut self.scratch[..chunk_len];

                mixer.mix_into(out);
                let _ = stream.put_data_f32(out);

                remaining -= chunk_len;
            }
        } else {
            while remaining > 0 {
                let chunk_len = remaining.min(self.scratch.len());
                let out = &mut self.scratch[..chunk_len];

                out.fill(0.0);
                let _ = stream.put_data_f32(out);

                remaining -= chunk_len;
            }
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
    use super::{Mixer, Sound};

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
}
