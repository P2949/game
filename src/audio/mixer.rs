use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crossbeam_queue::ArrayQueue;
use sdl3::audio::{AudioCallback, AudioFormat, AudioSpec, AudioStream, AudioStreamWithCallback};

const AUDIO_SCRATCH_SAMPLES: usize = 4096;
const AUDIO_COMMAND_QUEUE_CAPACITY: usize = 128;
const MAX_VOICES: usize = 32;

pub type SoundId = usize;

#[derive(Debug, Clone, Copy)]
enum AudioCommand {
    Play {
        sound_id: SoundId,
        volume: f32,
        looping: bool,
    },
}

pub struct Sound {
    samples: Vec<f32>,
    channels: u16,
    sample_rate: u32,
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

pub struct Voice {
    sound_id: SoundId,
    cursor: usize,
    volume: f32,
    looping: bool,
}

pub struct Mixer {
    sounds: Vec<Sound>,
    voices: Vec<Voice>,
    master_volume: f32,
    // Expected playback-stream output format (0 means "unspecified / don't
    // check"). The mixer plays sample data verbatim — it does not resample or
    // remap channels — so a sound that doesn't match would play at the wrong
    // speed/pitch. `add_sound` rejects mismatches.
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

    #[allow(dead_code)]
    pub fn output_sample_rate(&self) -> u32 {
        self.output_sample_rate
    }

    #[allow(dead_code)]
    pub fn output_channels(&self) -> u16 {
        self.output_channels
    }

    #[allow(dead_code)]
    pub fn sound_count(&self) -> usize {
        self.sounds.len()
    }

    #[allow(dead_code)]
    pub fn voice_count(&self) -> usize {
        self.voices.len()
    }

    #[allow(dead_code)]
    pub fn max_voices(&self) -> usize {
        MAX_VOICES
    }

    #[allow(dead_code)]
    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = sanitize_volume(volume);
    }

    /// Declares the output stream's channel count and sample rate so `add_sound`
    /// can flag sounds that don't match. Rejects a zero channel count or sample
    /// rate, which are never valid output formats. The default (0/0, set in
    /// `new`) leaves the check disabled.
    pub fn set_output_format(&mut self, channels: u16, sample_rate: u32) -> anyhow::Result<()> {
        if channels == 0 {
            anyhow::bail!("audio output channel count must be nonzero");
        }
        if sample_rate == 0 {
            anyhow::bail!("audio output sample rate must be nonzero");
        }
        self.output_channels = channels;
        self.output_sample_rate = sample_rate;
        Ok(())
    }

    /// Whether `sound` matches the configured output format. Always true when the
    /// output format is unspecified.
    pub fn sound_format_matches(&self, sound: &Sound) -> bool {
        self.output_channels == 0
            || (sound.channels == self.output_channels
                && sound.sample_rate == self.output_sample_rate)
    }

    /// Whether `sound`'s sample buffer is a whole number of frames — i.e. its
    /// length is a multiple of its (nonzero) channel count. A malformed buffer
    /// would drift across channels when looped.
    pub fn sound_is_well_formed(sound: &Sound) -> bool {
        sound.channels != 0 && sound.samples.len() % sound.channels as usize == 0
    }

    pub fn add_sound(&mut self, sound: Sound) -> anyhow::Result<SoundId> {
        if sound.samples.is_empty() {
            anyhow::bail!("sound must contain at least one sample");
        }
        if !Self::sound_is_well_formed(&sound) {
            anyhow::bail!(
                "sound has {} samples, which is not a whole number of {}-channel frames",
                sound.samples.len(),
                sound.channels,
            );
        }
        if sound.sample_rate == 0 {
            anyhow::bail!("sound sample rate must be nonzero");
        }
        if !sound.samples.iter().all(|sample| sample.is_finite()) {
            anyhow::bail!("sound contains non-finite sample data");
        }
        if !self.sound_format_matches(&sound) {
            anyhow::bail!(
                "sound format {}ch/{}Hz does not match mixer output {}ch/{}Hz; resampling is not implemented",
                sound.channels,
                sound.sample_rate,
                self.output_channels,
                self.output_sample_rate,
            );
        }

        let id = self.sounds.len();
        self.sounds.push(sound);
        Ok(id)
    }

    pub fn play(&mut self, sound_id: SoundId, volume: f32, looping: bool) {
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
    blip_sound: SoundId,
    // Shared with the audio callback, which only increments it. The main thread
    // reads it via `poll_dropped_frames` so we never log from the callback.
    put_failures: Arc<AtomicU64>,
    reported_failures: AtomicU64,
    command_push_failures: AtomicU64,
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
        mixer.set_master_volume(0.3);
        mixer.set_output_format(channels as u16, sample_rate as u32)?;
        let blip_sound =
            mixer.add_sound(sine_sound(660.0, 0.12, sample_rate as u32, channels as u16))?;
        let music_sound =
            mixer.add_sound(sine_sound(110.0, 1.0, sample_rate as u32, channels as u16))?;
        mixer.play(music_sound, 0.08, true);

        let commands = Arc::new(ArrayQueue::new(AUDIO_COMMAND_QUEUE_CAPACITY));
        let put_failures = Arc::new(AtomicU64::new(0));
        let audio = sdl.audio().map_err(anyhow::Error::msg)?;
        let stream = audio
            .open_playback_stream(
                &spec,
                MixerCallback {
                    mixer,
                    commands: Arc::clone(&commands),
                    scratch: vec![0.0; AUDIO_SCRATCH_SAMPLES],
                    put_failures: Arc::clone(&put_failures),
                },
            )
            .map_err(anyhow::Error::msg)?;
        stream.resume().map_err(anyhow::Error::msg)?;

        log::info!(
            "requested SDL audio: {sample_rate} Hz, {channels} channels, f32; mixer assumes this output format"
        );

        Ok(Self {
            commands,
            blip_sound,
            put_failures,
            reported_failures: AtomicU64::new(0),
            command_push_failures: AtomicU64::new(0),
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
            let total = self.command_push_failures.fetch_add(1, Ordering::Relaxed) + 1;
            if total.is_power_of_two() {
                log::warn!(
                    "audio command queue dropped {total} play request(s) since startup because the queue is full"
                );
            }
        }
    }

    /// Logs any newly-observed audio output drops. Call periodically from the main
    /// thread; logging happens here, never inside the realtime audio callback
    /// (which only bumps an atomic counter).
    ///
    /// A badly-behind stream fails `put_data_f32` on every callback, so a naive
    /// "log whenever the total grew" would emit one warning per main-loop frame.
    /// Instead we only log when the running total crosses a power of two: the very
    /// first drop is always reported, and subsequent reports back off
    /// exponentially (at 1, 2, 4, 8, … total drops) so a persistent fault can't
    /// flood the log. The reported delta is measured since the last *logged*
    /// total, so it stays meaningful across the gaps.
    pub fn poll_dropped_frames(&self) {
        let total = self.put_failures.load(Ordering::Relaxed);
        let reported = self.reported_failures.load(Ordering::Relaxed);
        if total <= reported || !crossed_power_of_two(reported, total) {
            return;
        }
        self.reported_failures.store(total, Ordering::Relaxed);
        log::warn!(
            "audio output dropped {} buffer(s) ({total} total since start); \
             the stream may be falling behind",
            total - reported,
        );
    }
}

/// Whether some power of two lies in the half-open interval `(previous, total]`.
/// Used to back off `poll_dropped_frames` logging: with `previous` as the
/// last-logged total, this is true exactly when the running total has crossed
/// the next power-of-two boundary since the last log.
fn crossed_power_of_two(previous: u64, total: u64) -> bool {
    if total == 0 {
        return false;
    }
    // Largest power of two that is <= total. If it is greater than `previous`,
    // it lies inside (previous, total]; otherwise every power of two <= total is
    // also <= previous, so none is in the interval.
    let highest = 1u64 << (u64::BITS - 1 - total.leading_zeros());
    highest > previous
}

struct MixerCallback {
    mixer: Mixer,
    commands: Arc<ArrayQueue<AudioCommand>>,
    scratch: Vec<f32>,
    // Incremented (never read) here on every `put_data_f32` failure. The main
    // thread reads/logs it via `AudioSystem::poll_dropped_frames`, so this
    // realtime callback never logs, locks, or allocates for diagnostics.
    put_failures: Arc<AtomicU64>,
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
                // Realtime callback: just bump the shared counter. The main thread
                // logs it via AudioSystem::poll_dropped_frames.
                self.put_failures.fetch_add(1, Ordering::Relaxed);
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

    Sound::new(samples, channels, sample_rate)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::AtomicU64;

    use crossbeam_queue::ArrayQueue;

    use super::{AudioCommand, Mixer, MixerCallback, Sound, crossed_power_of_two, sanitize_volume};

    #[test]
    fn crossed_power_of_two_reports_each_boundary_once() {
        // No growth, or no power-of-two boundary in (previous, total].
        assert!(!crossed_power_of_two(0, 0));
        assert!(!crossed_power_of_two(4, 4));
        assert!(!crossed_power_of_two(4, 5));
        assert!(!crossed_power_of_two(4, 7));
        assert!(!crossed_power_of_two(8, 15));

        // First drop and each later power-of-two crossing report exactly once.
        assert!(crossed_power_of_two(0, 1));
        assert!(crossed_power_of_two(1, 2));
        assert!(crossed_power_of_two(2, 4));
        assert!(crossed_power_of_two(4, 8));
        assert!(crossed_power_of_two(8, 16));

        // Stepping the total up one-at-a-time fires only on 1, 2, 4, 8, 16.
        let mut reported = 0u64;
        let mut fired = Vec::new();
        for total in 1..=16u64 {
            if total > reported && crossed_power_of_two(reported, total) {
                fired.push(total);
                reported = total;
            }
        }
        assert_eq!(fired, vec![1, 2, 4, 8, 16]);
    }

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
        let matching = Sound::new(vec![0.0], 2, 48_000);
        let mismatched = Sound::new(vec![0.0], 1, 44_100);

        // Unspecified output format accepts anything.
        assert!(mixer.sound_format_matches(&matching));
        assert!(mixer.sound_format_matches(&mismatched));

        mixer.set_output_format(2, 48_000).unwrap();
        assert!(mixer.sound_format_matches(&matching));
        assert!(!mixer.sound_format_matches(&mismatched));
    }

    #[test]
    fn set_output_format_rejects_zero_values() {
        let mut mixer = Mixer::new();
        assert!(mixer.set_output_format(0, 48_000).is_err());
        assert!(mixer.set_output_format(2, 0).is_err());
        assert!(mixer.set_output_format(2, 48_000).is_ok());
    }

    #[test]
    fn sound_well_formed_check_requires_whole_frames() {
        let stereo_ok = Sound::new(vec![0.0; 8], 2, 48_000);
        let stereo_bad = Sound::new(vec![0.0; 7], 2, 48_000);
        let zero_channels = Sound::new(vec![0.0; 4], 0, 48_000);
        assert!(Mixer::sound_is_well_formed(&stereo_ok));
        assert!(!Mixer::sound_is_well_formed(&stereo_bad));
        assert!(!Mixer::sound_is_well_formed(&zero_channels));
    }

    #[test]
    fn add_sound_rejects_malformed_sounds() {
        let mut mixer = Mixer::new();

        assert!(mixer.add_sound(Sound::new(Vec::new(), 1, 48_000)).is_err());
        assert!(
            mixer
                .add_sound(Sound::new(vec![0.0, 0.0, 0.0], 2, 48_000))
                .is_err()
        );
        assert!(mixer.add_sound(Sound::new(vec![0.0], 1, 0)).is_err());
        assert!(
            mixer
                .add_sound(Sound::new(vec![f32::NAN], 1, 48_000))
                .is_err()
        );
        assert_eq!(mixer.sound_count(), 0);
    }

    #[test]
    fn add_sound_rejects_output_format_mismatch() {
        let mut mixer = Mixer::new();
        mixer.set_output_format(2, 48_000).unwrap();

        assert!(
            mixer
                .add_sound(Sound::new(vec![0.0, 0.0], 1, 48_000))
                .is_err()
        );
        assert!(
            mixer
                .add_sound(Sound::new(vec![0.0, 0.0], 2, 44_100))
                .is_err()
        );
        assert_eq!(mixer.sound_count(), 0);
    }

    #[test]
    fn play_sanitizes_voice_volume() {
        let mut mixer = Mixer::new();
        let sound_id = mixer.add_sound(Sound::new(vec![1.0], 1, 48_000)).unwrap();

        mixer.play(sound_id, f32::NAN, true);

        assert_eq!(mixer.voices.len(), 1);
        assert_eq!(mixer.voices[0].volume, 0.0);
    }

    #[test]
    fn mix_into_ignores_non_finite_master_volume() {
        let mut mixer = Mixer::new();
        mixer.master_volume = f32::NAN;
        let sound_id = mixer
            .add_sound(Sound::new(vec![0.5, 0.5], 1, 48_000))
            .unwrap();
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
        let sound_id = mixer
            .add_sound(Sound::new(vec![0.5, 0.5], 1, 48_000))
            .unwrap();
        mixer.play(sound_id, 1.0, false);

        let mut out = [0.0; 4];
        mixer.mix_into(&mut out);

        assert_eq!(&out, &[0.5, 0.5, 0.0, 0.0]);
        assert!(mixer.voices.is_empty());
    }

    #[test]
    fn overlapping_voices_sum_and_clamp() {
        let mut mixer = Mixer::new();
        let sound_id = mixer.add_sound(Sound::new(vec![0.75], 1, 48_000)).unwrap();
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
        mixer.set_master_volume(0.3);

        let music_sound = mixer
            .add_sound(super::sine_sound(110.0, 1.0, sample_rate, channels))
            .unwrap();
        mixer.play(music_sound, 0.08, true);

        let mut out = vec![0.0; 1024];
        mixer.mix_into(&mut out);

        assert!(
            out.iter().any(|sample| sample.abs() > 0.0001),
            "looping music mixed only silence"
        );
    }

    #[test]
    fn empty_sound_is_rejected() {
        let mut mixer = Mixer::new();
        assert!(mixer.add_sound(Sound::new(Vec::new(), 1, 48_000)).is_err());
        assert_eq!(mixer.sound_count(), 0);
    }

    #[test]
    fn invalid_sound_id_is_ignored() {
        let mut mixer = Mixer::new();

        mixer.play(123, 1.0, false);

        assert_eq!(mixer.voice_count(), 0);
    }

    #[test]
    fn invalid_sound_id_keeps_voice_count_stable() {
        let mut mixer = Mixer::new();
        let sound_id = mixer.add_sound(Sound::new(vec![0.25], 1, 48_000)).unwrap();
        mixer.play(sound_id, 1.0, true);
        assert_eq!(mixer.voice_count(), 1);

        mixer.play(sound_id + 100, 1.0, false);

        assert_eq!(mixer.voice_count(), 1);
    }

    #[test]
    fn play_drops_new_voice_when_voice_cap_is_reached() {
        let mut mixer = Mixer::new();
        let sound_id = mixer.add_sound(Sound::new(vec![0.25], 1, 48_000)).unwrap();

        for _ in 0..super::MAX_VOICES {
            mixer.play(sound_id, 1.0, true);
        }

        assert_eq!(mixer.voice_count(), super::MAX_VOICES);

        mixer.play(sound_id, 1.0, true);

        assert_eq!(mixer.voice_count(), super::MAX_VOICES);
    }

    #[test]
    fn callback_drains_play_commands() {
        let mut mixer = Mixer::new();
        let sound_id = mixer.add_sound(Sound::new(vec![0.25], 1, 48_000)).unwrap();
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
            put_failures: Arc::new(AtomicU64::new(0)),
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
        let sound_id = mixer.add_sound(Sound::new(vec![0.25], 1, 48_000)).unwrap();

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
            put_failures: Arc::new(AtomicU64::new(0)),
        };

        callback.drain_commands();

        assert!(commands.is_empty());
        assert_eq!(callback.mixer.voices.len(), super::MAX_VOICES);
        assert!(callback.mixer.voices.capacity() >= super::MAX_VOICES);
    }
}
