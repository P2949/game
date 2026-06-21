use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crossbeam_queue::ArrayQueue;
use game_core::backend::{SoundHandle, SoundLoadRequest};
use sdl3::audio::{AudioCallback, AudioFormat, AudioSpec, AudioStream, AudioStreamWithCallback};

#[cfg(feature = "ogg")]
use std::io::Cursor;
#[cfg(feature = "mp3")]
use std::io::Write;
#[cfg(feature = "mp3")]
use std::process::{Command, Stdio};

const AUDIO_SCRATCH_SAMPLES: usize = 4096;
const AUDIO_COMMAND_QUEUE_CAPACITY: usize = 128;
const MAX_VOICES: usize = 32;
/// Maximum number of custom sound-effect buses. Master, music, and the default
/// SFX group are separate and do not count toward this limit.
const MAX_NAMED_BUSES: usize = 32;
// Upper bound on the per-channel frame count a generated tone may allocate.
// 48 kHz * 60 s = 2.88M frames, so a one-minute tone fits comfortably while an
// absurd `seconds`/`sample_rate` request is rejected before allocating.
const MAX_GENERATED_FRAMES: usize = 1 << 22; // 4,194,304 frames
// Upper bound on a generated tone's channel count, so a huge `channels` cannot
// blow up the interleaved buffer even when the frame count is within cap.
const MAX_GENERATED_CHANNELS: u16 = 8;
// Hard cap on the total interleaved sample count (frames * channels).
const MAX_GENERATED_SAMPLES: usize = MAX_GENERATED_FRAMES * MAX_GENERATED_CHANNELS as usize;

pub type SoundId = usize;

/// Outcome of [`Mixer::play`]. Returned so callers (and tests) can distinguish a
/// started voice from the two silent-drop cases instead of guessing from voice
/// counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayResult {
    Started,
    DroppedVoiceLimit,
    InvalidSoundId,
}

#[derive(Debug, Clone, Copy)]
enum AudioCommand {
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
    CrossfadeMusic {
        sound_id: SoundId,
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

#[derive(Debug, Clone, Copy)]
struct MusicFade {
    from: f32,
    to: f32,
    elapsed_seconds: f32,
    duration_seconds: f32,
}

#[derive(Debug, Clone, Copy)]
struct VoiceFade {
    from: f32,
    to: f32,
    elapsed_seconds: f32,
    duration_seconds: f32,
}

#[derive(Debug)]
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

// Voices are created only by Mixer::play after sound_id validation, so mix_into
// can index self.sounds by voice.sound_id without revalidating every sample.
pub struct Voice {
    sound_id: SoundId,
    cursor: usize,
    volume: f32,
    looping: bool,
    music: bool,
    bus: Option<u8>,
    fade_volume: f32,
    fade: Option<VoiceFade>,
    remove_when_faded: bool,
}

pub struct Mixer {
    sounds: Vec<Sound>,
    voices: Vec<Voice>,
    master_volume: f32,
    sfx_volume: f32,
    music_volume: f32,
    music_fade_volume: f32,
    music_fade: Option<MusicFade>,
    music_paused: bool,
    bus_volumes: [f32; MAX_NAMED_BUSES],
    // Expected playback-stream output format (0 means "unspecified / don't
    // check"). File-backed WAVs are normalized to this format before insertion;
    // `add_sound` still rejects mismatches to catch generated/internal misuse.
    output_channels: u16,
    output_sample_rate: u32,
    // Incremented whenever `play` drops a request because all voices are in use.
    // Shared (via `dropped_voices_handle`) with the main thread so missing SFX
    // caused by the voice cap can be diagnosed without logging from the realtime
    // mix callback.
    dropped_voices: Arc<AtomicU64>,
}

impl Mixer {
    pub fn new() -> Self {
        Self {
            sounds: Vec::new(),
            voices: Vec::with_capacity(MAX_VOICES),
            master_volume: 1.0,
            sfx_volume: 1.0,
            music_volume: 1.0,
            music_fade_volume: 1.0,
            music_fade: None,
            music_paused: false,
            bus_volumes: [1.0; MAX_NAMED_BUSES],
            output_channels: 0,
            output_sample_rate: 0,
            dropped_voices: Arc::new(AtomicU64::new(0)),
        }
    }

    /// A shared handle to the voice-drop counter, for reading from another thread
    /// (the audio callback owns the `Mixer`, the main thread polls this).
    pub fn dropped_voices_handle(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.dropped_voices)
    }

    #[allow(dead_code)]
    pub fn dropped_voice_count(&self) -> u64 {
        self.dropped_voices.load(Ordering::Relaxed)
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

    pub fn sfx_volume(&self) -> f32 {
        self.sfx_volume
    }

    pub fn music_volume(&self) -> f32 {
        self.music_volume
    }

    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.sfx_volume = sanitize_volume(volume);
    }

    pub fn set_music_volume(&mut self, volume: f32) {
        self.music_volume = sanitize_volume(volume);
    }

    pub fn set_bus_volume(&mut self, bus: u8, volume: f32) {
        if let Some(bus_volume) = self.bus_volumes.get_mut(bus as usize) {
            *bus_volume = sanitize_volume(volume);
        }
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
        sound.channels != 0 && sound.samples.len().is_multiple_of(sound.channels as usize)
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
                "sound format {}ch/{}Hz does not match mixer output {}ch/{}Hz; normalize it before adding to the mixer",
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

    pub fn play(&mut self, sound_id: SoundId, volume: f32, looping: bool) -> PlayResult {
        self.play_on_bus(sound_id, volume, looping, None)
    }

    pub fn play_on_bus(
        &mut self,
        sound_id: SoundId,
        volume: f32,
        looping: bool,
        bus: Option<u8>,
    ) -> PlayResult {
        self.push_voice(sound_id, volume, looping, false, bus)
    }

    pub fn play_music(
        &mut self,
        sound_id: SoundId,
        volume: f32,
        fade_in_seconds: Option<f32>,
    ) -> PlayResult {
        self.stop_music();
        self.music_paused = false;
        match fade_in_seconds {
            Some(duration_seconds) => {
                self.music_fade_volume = 0.0;
                self.fade_music_to(1.0, duration_seconds);
            }
            None => {
                self.music_fade_volume = 1.0;
                self.music_fade = None;
            }
        }
        self.push_voice(sound_id, volume, true, true, None)
    }

    /// Blends every active music voice out while fading a new looping track in.
    /// Unlike `play_music`, this deliberately keeps the old voices alive until
    /// their per-voice fade completes.
    pub fn crossfade_music(
        &mut self,
        sound_id: SoundId,
        volume: f32,
        duration_seconds: f32,
    ) -> PlayResult {
        if !duration_seconds.is_finite() || duration_seconds <= 0.0 {
            return self.play_music(sound_id, volume, None);
        }

        self.music_paused = false;
        self.music_fade = None;
        self.music_fade_volume = 1.0;
        for voice in self.voices.iter_mut().filter(|voice| voice.music) {
            voice.fade = Some(VoiceFade {
                from: voice.fade_volume,
                to: 0.0,
                elapsed_seconds: 0.0,
                duration_seconds,
            });
            voice.remove_when_faded = true;
        }

        let result = self.push_voice(sound_id, volume, true, true, None);
        if matches!(result, PlayResult::Started) {
            let voice = self
                .voices
                .last_mut()
                .expect("a started voice is appended to the mixer");
            voice.fade_volume = 0.0;
            voice.fade = Some(VoiceFade {
                from: 0.0,
                to: 1.0,
                elapsed_seconds: 0.0,
                duration_seconds,
            });
        }
        result
    }

    pub fn stop_music(&mut self) {
        self.voices.retain(|voice| !voice.music);
        self.music_paused = false;
        self.music_fade = None;
        self.music_fade_volume = 1.0;
    }

    pub fn pause_music(&mut self) {
        self.music_paused = true;
    }

    pub fn resume_music(&mut self) {
        self.music_paused = false;
    }

    pub fn fade_music_to(&mut self, volume: f32, duration_seconds: f32) {
        let target = sanitize_volume(volume);
        if !duration_seconds.is_finite() || duration_seconds <= 0.0 {
            self.music_fade_volume = target;
            self.music_fade = None;
            return;
        }
        self.music_fade = Some(MusicFade {
            from: self.music_fade_volume,
            to: target,
            elapsed_seconds: 0.0,
            duration_seconds,
        });
    }

    fn push_voice(
        &mut self,
        sound_id: SoundId,
        volume: f32,
        looping: bool,
        music: bool,
        bus: Option<u8>,
    ) -> PlayResult {
        let Some(sound) = self.sounds.get(sound_id) else {
            return PlayResult::InvalidSoundId;
        };

        if sound.samples.is_empty() {
            return PlayResult::InvalidSoundId;
        }

        if self.voices.len() >= MAX_VOICES {
            self.dropped_voices.fetch_add(1, Ordering::Relaxed);
            return PlayResult::DroppedVoiceLimit;
        }

        self.voices.push(Voice {
            sound_id,
            cursor: 0,
            volume: sanitize_volume(volume),
            looping,
            music,
            bus,
            fade_volume: 1.0,
            fade: None,
            remove_when_faded: false,
        });
        PlayResult::Started
    }

    pub fn mix_into(&mut self, out: &mut [f32]) {
        out.fill(0.0);

        self.advance_music_fade(out.len());
        self.advance_voice_fades(out.len());

        // Sanitize once per mix as a final defense against internal misuse or
        // tests that bypass set_*_volume.
        let master_volume = sanitize_volume(self.master_volume);
        let sfx_volume = sanitize_volume(self.sfx_volume);
        let music_volume = sanitize_volume(self.music_volume) * self.music_fade_volume;

        for voice in &mut self.voices {
            if voice.music && self.music_paused {
                continue;
            }
            let sound = &self.sounds[voice.sound_id];
            let group_volume = if voice.music {
                music_volume
            } else {
                sfx_volume
                    * voice
                        .bus
                        .and_then(|bus| self.bus_volumes.get(bus as usize).copied())
                        .unwrap_or(1.0)
            };
            for sample in out.iter_mut() {
                if voice.cursor >= sound.samples.len() {
                    if voice.looping {
                        voice.cursor = 0;
                    } else {
                        break;
                    }
                }

                *sample += sound.samples[voice.cursor]
                    * voice.volume
                    * voice.fade_volume
                    * group_volume
                    * master_volume;
                voice.cursor += 1;
            }
        }

        for sample in out.iter_mut() {
            *sample = sample.clamp(-1.0, 1.0);
        }

        self.voices.retain(|voice| {
            (voice.looping || voice.cursor < self.sounds[voice.sound_id].samples.len())
                && !(voice.remove_when_faded
                    && voice.fade.is_none()
                    && voice.fade_volume <= f32::EPSILON)
        });
    }

    fn advance_music_fade(&mut self, samples: usize) {
        let Some(mut fade) = self.music_fade else {
            return;
        };
        let frames = if self.output_channels == 0 {
            samples as f32
        } else {
            samples as f32 / self.output_channels as f32
        };
        let sample_rate = self.output_sample_rate.max(1) as f32;
        fade.elapsed_seconds += frames / sample_rate;
        let progress = (fade.elapsed_seconds / fade.duration_seconds).clamp(0.0, 1.0);
        self.music_fade_volume = fade.from + (fade.to - fade.from) * progress;
        self.music_fade = (progress < 1.0).then_some(fade);
    }

    fn advance_voice_fades(&mut self, samples: usize) {
        let frames = if self.output_channels == 0 {
            samples as f32
        } else {
            samples as f32 / self.output_channels as f32
        };
        let elapsed_seconds = frames / self.output_sample_rate.max(1) as f32;
        for voice in &mut self.voices {
            let Some(mut fade) = voice.fade else {
                continue;
            };
            fade.elapsed_seconds += elapsed_seconds;
            let progress = (fade.elapsed_seconds / fade.duration_seconds).clamp(0.0, 1.0);
            voice.fade_volume = fade.from + (fade.to - fade.from) * progress;
            voice.fade = (progress < 1.0).then_some(fade);
        }
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
    sound_ids: HashMap<SoundHandle, SoundId>,
    /// Maps friendly bus names to compact identifiers on the main thread. The
    /// callback only receives those identifiers, so it does not allocate or
    /// hash strings while mixing audio.
    bus_ids: Mutex<HashMap<String, u8>>,
    // Shared with the audio callback, which only increments it. The main thread
    // reads it via `poll_dropped_frames` so we never log from the callback.
    put_failures: Arc<AtomicU64>,
    reported_failures: AtomicU64,
    command_push_failures: AtomicU64,
    // Shared with the mixer (owned by the callback). The mixer bumps it when a
    // play request is dropped at the voice cap; the main thread reports it via
    // `poll_dropped_voices`.
    dropped_voices: Arc<AtomicU64>,
    reported_voice_drops: AtomicU64,
    _stream: AudioStreamWithCallback<MixerCallback>,
}

impl AudioSystem {
    pub fn new(
        sdl: &sdl3::Sdl,
        asset_root: &Path,
        sound_loads: Vec<(SoundHandle, SoundLoadRequest)>,
    ) -> anyhow::Result<Self> {
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
        let dropped_voices = mixer.dropped_voices_handle();
        let blip_sound = mixer.add_sound(sine_sound(
            660.0,
            0.12,
            sample_rate as u32,
            channels as u16,
        )?)?;

        let mut sound_ids = HashMap::new();
        for (handle, request) in sound_loads {
            let sound_id = match request {
                SoundLoadRequest::Generated { .. } => blip_sound,
                SoundLoadRequest::File { path } => {
                    let sound = load_file_sound(
                        &asset_root.join(&path),
                        channels as u16,
                        sample_rate as u32,
                    )?;
                    mixer.add_sound(sound)?
                }
            };
            sound_ids.insert(handle, sound_id);
        }

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
            sound_ids,
            bus_ids: Mutex::new(HashMap::new()),
            put_failures,
            reported_failures: AtomicU64::new(0),
            command_push_failures: AtomicU64::new(0),
            dropped_voices,
            reported_voice_drops: AtomicU64::new(0),
            _stream: stream,
        })
    }

    pub fn play_blip(&self) {
        self.enqueue_play(self.blip_sound, 0.8, false, None);
    }

    pub fn play(&self, sound: SoundHandle, volume: f32, looping: bool) {
        self.play_on_bus(sound, volume, looping, None);
    }

    pub fn play_on_bus(&self, sound: SoundHandle, volume: f32, looping: bool, bus: Option<&str>) {
        let Some(sound_id) = self.sound_ids.get(&sound).copied() else {
            log::warn!("ignoring play request for unknown sound handle {:?}", sound);
            return;
        };
        self.enqueue_play(
            sound_id,
            volume,
            looping,
            bus.and_then(|bus| self.bus_id(bus)),
        );
    }

    pub fn play_music(&self, sound: SoundHandle, volume: f32) {
        self.play_music_with_fade(sound, volume, None);
    }

    pub fn play_music_fade_in(&self, sound: SoundHandle, volume: f32, duration_seconds: f32) {
        self.play_music_with_fade(sound, volume, Some(duration_seconds));
    }

    pub fn crossfade_music(&self, sound: SoundHandle, volume: f32, duration_seconds: f32) {
        let Some(sound_id) = self.sound_ids.get(&sound).copied() else {
            log::warn!(
                "ignoring crossfade_music request for unknown sound handle {:?}",
                sound
            );
            return;
        };
        self.enqueue_audio_command(AudioCommand::CrossfadeMusic {
            sound_id,
            volume,
            duration_seconds,
        });
    }

    fn play_music_with_fade(&self, sound: SoundHandle, volume: f32, fade_in_seconds: Option<f32>) {
        let Some(sound_id) = self.sound_ids.get(&sound).copied() else {
            log::warn!(
                "ignoring play_music request for unknown sound handle {:?}",
                sound
            );
            return;
        };
        self.enqueue_audio_command(AudioCommand::PlayMusic {
            sound_id,
            volume,
            fade_in_seconds,
        });
    }

    pub fn stop_music(&self) {
        self.enqueue_audio_command(AudioCommand::StopMusic);
    }

    pub fn pause_music(&self) {
        self.enqueue_audio_command(AudioCommand::PauseMusic);
    }

    pub fn resume_music(&self) {
        self.enqueue_audio_command(AudioCommand::ResumeMusic);
    }

    pub fn set_master_volume(&self, volume: f32) {
        self.enqueue_audio_command(AudioCommand::SetMasterVolume(volume));
    }

    pub fn set_sfx_volume(&self, volume: f32) {
        self.enqueue_audio_command(AudioCommand::SetSfxVolume(volume));
    }

    pub fn set_music_volume(&self, volume: f32) {
        self.enqueue_audio_command(AudioCommand::SetMusicVolume(volume));
    }

    pub fn set_bus_volume(&self, bus: &str, volume: f32) {
        let Some(bus) = self.bus_id(bus) else {
            return;
        };
        self.enqueue_audio_command(AudioCommand::SetBusVolume { bus, volume });
    }

    pub fn fade_music_to(&self, volume: f32, duration_seconds: f32) {
        self.enqueue_audio_command(AudioCommand::FadeMusicTo {
            volume,
            duration_seconds,
        });
    }

    fn enqueue_play(&self, sound_id: SoundId, volume: f32, looping: bool, bus: Option<u8>) {
        self.enqueue_audio_command(AudioCommand::Play {
            sound_id,
            volume,
            looping,
            bus,
        });
    }

    fn bus_id(&self, name: &str) -> Option<u8> {
        let name = name.trim();
        if name.is_empty() {
            log::warn!("ignoring audio bus with an empty name");
            return None;
        }

        let mut buses = self
            .bus_ids
            .lock()
            .expect("audio bus registry lock poisoned");
        if let Some(id) = buses.get(name) {
            return Some(*id);
        }
        if buses.len() >= MAX_NAMED_BUSES {
            log::warn!(
                "ignoring audio bus '{name}': at most {MAX_NAMED_BUSES} named buses are supported"
            );
            return None;
        }
        let id = buses.len() as u8;
        buses.insert(name.to_owned(), id);
        Some(id)
    }

    fn enqueue_audio_command(&self, command: AudioCommand) {
        if self.commands.push(command).is_err() {
            let total = self.command_push_failures.fetch_add(1, Ordering::Relaxed) + 1;
            if total.is_power_of_two() {
                log::warn!(
                    "audio command queue dropped {total} command(s) since startup because the queue is full"
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

    /// Logs any newly-observed voice-cap drops. Like [`Self::poll_dropped_frames`]
    /// this runs on the main thread (the mixer only bumps an atomic from the
    /// realtime callback) and backs off on a power-of-two boundary so a sustained
    /// flood of dropped SFX cannot flood the log.
    pub fn poll_dropped_voices(&self) {
        let total = self.dropped_voices.load(Ordering::Relaxed);
        let reported = self.reported_voice_drops.load(Ordering::Relaxed);
        if total <= reported || !crossed_power_of_two(reported, total) {
            return;
        }
        self.reported_voice_drops.store(total, Ordering::Relaxed);
        log::warn!(
            "audio dropped {} sound(s) at the {MAX_VOICES}-voice cap ({total} total since start)",
            total - reported,
        );
    }

    #[allow(dead_code)]
    pub fn dropped_voices(&self) -> u64 {
        self.dropped_voices.load(Ordering::Relaxed)
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
                    bus,
                } => {
                    self.mixer.play_on_bus(sound_id, volume, looping, bus);
                }
                AudioCommand::PlayMusic {
                    sound_id,
                    volume,
                    fade_in_seconds,
                } => {
                    self.mixer.play_music(sound_id, volume, fade_in_seconds);
                }
                AudioCommand::CrossfadeMusic {
                    sound_id,
                    volume,
                    duration_seconds,
                } => {
                    self.mixer
                        .crossfade_music(sound_id, volume, duration_seconds);
                }
                AudioCommand::StopMusic => self.mixer.stop_music(),
                AudioCommand::PauseMusic => self.mixer.pause_music(),
                AudioCommand::ResumeMusic => self.mixer.resume_music(),
                AudioCommand::SetMasterVolume(volume) => self.mixer.set_master_volume(volume),
                AudioCommand::SetSfxVolume(volume) => self.mixer.set_sfx_volume(volume),
                AudioCommand::SetMusicVolume(volume) => self.mixer.set_music_volume(volume),
                AudioCommand::SetBusVolume { bus, volume } => {
                    self.mixer.set_bus_volume(bus, volume)
                }
                AudioCommand::FadeMusicTo {
                    volume,
                    duration_seconds,
                } => self.mixer.fade_music_to(volume, duration_seconds),
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

/// Validates the shared parameters of the generated-tone helpers, returning
/// `(frames, total_samples)` to synthesize (`total_samples = frames * channels`).
/// Rejects non-finite/non-positive inputs, channel counts and frame counts past
/// their caps, frequencies above Nyquist, and any request whose interleaved
/// sample count would exceed [`MAX_GENERATED_SAMPLES`] — so no bad parameter can
/// allocate an absurd buffer or drive the cycle/phase math non-finite.
fn generated_tone_frames(
    freq: f32,
    seconds: f32,
    sample_rate: u32,
    channels: u16,
) -> anyhow::Result<(usize, usize)> {
    if !freq.is_finite() || freq <= 0.0 {
        anyhow::bail!("tone frequency must be finite and positive, got {freq}");
    }
    if !seconds.is_finite() || seconds <= 0.0 {
        anyhow::bail!("tone duration must be finite and positive, got {seconds}");
    }
    if sample_rate == 0 {
        anyhow::bail!("tone sample rate must be nonzero");
    }
    if channels == 0 {
        anyhow::bail!("tone channel count must be nonzero");
    }
    if channels > MAX_GENERATED_CHANNELS {
        anyhow::bail!("tone channel count {channels} exceeds maximum {MAX_GENERATED_CHANNELS}");
    }

    // Above Nyquist a tone only aliases; rejecting it also bounds `freq`, which
    // keeps the per-frame phase (and `loop_sine_sound`'s cycle count) finite.
    let nyquist = sample_rate as f32 * 0.5;
    if freq > nyquist {
        anyhow::bail!("tone frequency {freq} exceeds Nyquist {nyquist} for {sample_rate} Hz");
    }

    let frames = (seconds * sample_rate as f32) as usize;
    if frames == 0 {
        anyhow::bail!("tone duration {seconds}s at {sample_rate} Hz yields zero frames");
    }
    if frames > MAX_GENERATED_FRAMES {
        anyhow::bail!(
            "tone would synthesize {frames} frames, exceeding maximum {MAX_GENERATED_FRAMES}"
        );
    }

    let total_samples = frames
        .checked_mul(channels as usize)
        .ok_or_else(|| anyhow::anyhow!("tone sample count overflow ({frames} x {channels})"))?;
    if total_samples > MAX_GENERATED_SAMPLES {
        anyhow::bail!(
            "tone would synthesize {total_samples} samples, exceeding maximum {MAX_GENERATED_SAMPLES}"
        );
    }

    Ok((frames, total_samples))
}

/// A one-shot decaying sine tone (full amplitude at the start, fading to silence
/// at the end). Suitable for transient SFX like the action blip; not suitable for
/// looping, since the amplitude discontinuity at the loop seam clicks (use
/// [`loop_sine_sound`] for sustained/looping tones).
fn sine_sound(freq: f32, seconds: f32, sample_rate: u32, channels: u16) -> anyhow::Result<Sound> {
    let (frames, total_samples) = generated_tone_frames(freq, seconds, sample_rate, channels)?;
    let mut samples = Vec::with_capacity(total_samples);

    for frame in 0..frames {
        let t = frame as f32 / sample_rate as f32;
        let envelope = 1.0 - frame as f32 / frames as f32;
        let sample = (t * freq * std::f32::consts::TAU).sin() * envelope;
        for _ in 0..channels {
            samples.push(sample);
        }
    }

    Ok(Sound::new(samples, channels, sample_rate))
}

/// A seamless looping sine tone. The frequency is snapped to the nearest value
/// that fits a whole number of cycles in the buffer, so the sample *after* the
/// last one is exactly the first sample again: no value or slope discontinuity at
/// the loop seam, and no decay envelope. This avoids the periodic click a looped
/// one-shot ([`sine_sound`]) produces.
#[allow(dead_code)]
fn loop_sine_sound(
    freq: f32,
    seconds: f32,
    sample_rate: u32,
    channels: u16,
) -> anyhow::Result<Sound> {
    let (frames, total_samples) = generated_tone_frames(freq, seconds, sample_rate, channels)?;

    // Whole-cycle count closest to the requested frequency (at least one), so
    // sample[frames] == sample[0] and the loop wraps continuously. `freq` is
    // bounded by Nyquist and `frames` by its cap, so this is always finite.
    let cycles = (freq * frames as f32 / sample_rate as f32).round().max(1.0);
    let mut samples = Vec::with_capacity(total_samples);

    for frame in 0..frames {
        let phase = std::f32::consts::TAU * cycles * frame as f32 / frames as f32;
        let sample = phase.sin();
        for _ in 0..channels {
            samples.push(sample);
        }
    }

    Ok(Sound::new(samples, channels, sample_rate))
}

fn load_file_sound(
    path: &Path,
    target_channels: u16,
    target_sample_rate: u32,
) -> anyhow::Result<Sound> {
    let bytes = fs::read(path)
        .map_err(anyhow::Error::from)
        .map_err(|err| anyhow::anyhow!("failed to read sound '{}': {err}", path.display()))?;
    decode_file_sound(path, &bytes, target_channels, target_sample_rate)
}

fn decode_file_sound(
    path: &Path,
    bytes: &[u8],
    target_channels: u16,
    target_sample_rate: u32,
) -> anyhow::Result<Sound> {
    match detect_sound_format(path, bytes) {
        SoundFormat::Wav => {
            let sound = decode_wav_sound(bytes)
                .map_err(|err| unsupported_wav_error(path, target_sample_rate, err))?;
            normalize_sound(sound, target_channels, target_sample_rate)
                .map_err(|err| unsupported_wav_error(path, target_sample_rate, err))
        }
        SoundFormat::Ogg => {
            #[cfg(feature = "ogg")]
            {
                let sound = decode_ogg_sound(bytes)
                    .map_err(|err| unsupported_ogg_error(path, target_sample_rate, err))?;
                normalize_sound(sound, target_channels, target_sample_rate)
                    .map_err(|err| unsupported_ogg_error(path, target_sample_rate, err))
            }
            #[cfg(not(feature = "ogg"))]
            {
                let _ = (bytes, target_channels, target_sample_rate);
                Err(ogg_feature_required_error(path))
            }
        }
        SoundFormat::Mp3 => {
            #[cfg(feature = "mp3")]
            {
                let sound = decode_mp3_sound(bytes)
                    .map_err(|err| unsupported_mp3_error(path, target_sample_rate, err))?;
                normalize_sound(sound, target_channels, target_sample_rate)
                    .map_err(|err| unsupported_mp3_error(path, target_sample_rate, err))
            }
            #[cfg(not(feature = "mp3"))]
            {
                let _ = (bytes, target_channels, target_sample_rate);
                Err(mp3_feature_required_error(path))
            }
        }
        SoundFormat::Unknown => Err(unsupported_sound_format_error(path)),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SoundFormat {
    Wav,
    Ogg,
    Mp3,
    Unknown,
}

fn detect_sound_format(path: &Path, bytes: &[u8]) -> SoundFormat {
    if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WAVE" {
        return SoundFormat::Wav;
    }
    if bytes.starts_with(b"OggS") {
        return SoundFormat::Ogg;
    }
    if bytes.starts_with(b"ID3")
        || bytes
            .get(0..2)
            .is_some_and(|header| header[0] == 0xff && header[1] & 0xe0 == 0xe0)
    {
        return SoundFormat::Mp3;
    }

    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .as_deref()
    {
        Some("wav") => SoundFormat::Wav,
        Some("ogg") => SoundFormat::Ogg,
        Some("mp3") => SoundFormat::Mp3,
        _ => SoundFormat::Unknown,
    }
}

fn unsupported_sound_format_error(path: &Path) -> anyhow::Error {
    #[cfg(all(feature = "ogg", feature = "mp3"))]
    let supported = "WAV, OGG Vorbis, or MP3";
    #[cfg(all(feature = "ogg", not(feature = "mp3")))]
    let supported = "WAV or OGG Vorbis (MP3 requires the `mp3` feature)";
    #[cfg(all(not(feature = "ogg"), feature = "mp3"))]
    let supported = "WAV or MP3 (OGG Vorbis requires the `ogg` feature)";
    #[cfg(all(not(feature = "ogg"), not(feature = "mp3")))]
    let supported = "WAV (or OGG Vorbis / MP3 with their optional features enabled)";

    anyhow::anyhow!(
        "Sound file '{}' uses an unsupported format.\n\nSupported here: {supported}.\n\nTry converting with:\n    ffmpeg -i input.ext -ac 2 -ar 48000 {}",
        path.display(),
        path.display(),
    )
}

#[cfg(not(feature = "ogg"))]
fn ogg_feature_required_error(path: &Path) -> anyhow::Error {
    anyhow::anyhow!(
        "OGG audio requires the `ogg` feature.\n\nEither enable the feature or convert to WAV:\n    ffmpeg -i {} -ac 2 -ar 48000 {}",
        path.display(),
        path.with_extension("wav").display(),
    )
}

#[cfg(not(feature = "mp3"))]
fn mp3_feature_required_error(path: &Path) -> anyhow::Error {
    anyhow::anyhow!(
        "MP3 audio requires the optional `mp3` feature.\n\nEither enable it (with ffmpeg available on PATH) or convert to WAV:\n    ffmpeg -i {} -ac 2 -ar 48000 {}",
        path.display(),
        path.with_extension("wav").display(),
    )
}

fn unsupported_wav_error(
    path: &Path,
    target_sample_rate: u32,
    err: anyhow::Error,
) -> anyhow::Error {
    anyhow::anyhow!(
        "Sound file '{}' uses unsupported format.\n\nSupported today:\n- WAV\n- mono or stereo\n- PCM16 or float32 samples\n- any sample rate will be converted to {target_sample_rate} Hz\n\nTry converting with:\n    ffmpeg -i input.wav -ac 2 -ar {target_sample_rate} {}\n\nDetails: {err}",
        path.display(),
        path.display(),
    )
}

#[cfg(feature = "ogg")]
fn unsupported_ogg_error(
    path: &Path,
    target_sample_rate: u32,
    err: anyhow::Error,
) -> anyhow::Error {
    anyhow::anyhow!(
        "Sound file '{}' could not be decoded as OGG Vorbis.\n\nSupported OGG input:\n- mono or stereo Vorbis\n- any sample rate will be converted to {target_sample_rate} Hz\n\nTry converting with:\n    ffmpeg -i input.ogg -ac 2 -ar {target_sample_rate} {}\n\nDetails: {err}",
        path.display(),
        path.display(),
    )
}

#[cfg(feature = "mp3")]
fn decode_mp3_sound(bytes: &[u8]) -> anyhow::Result<Sound> {
    let mut child = Command::new("ffmpeg")
        .args([
            "-v",
            "error",
            "-i",
            "pipe:0",
            "-f",
            "wav",
            "-acodec",
            "pcm_f32le",
            "pipe:1",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| anyhow::anyhow!("could not start ffmpeg: {err}"))?;
    child
        .stdin
        .take()
        .expect("piped ffmpeg stdin is available")
        .write_all(bytes)
        .map_err(|err| anyhow::anyhow!("could not send MP3 data to ffmpeg: {err}"))?;
    let output = child
        .wait_with_output()
        .map_err(|err| anyhow::anyhow!("could not read ffmpeg output: {err}"))?;
    if !output.status.success() {
        let diagnostic = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffmpeg could not decode the MP3: {}", diagnostic.trim());
    }
    decode_wav_sound(&output.stdout)
}

#[cfg(feature = "mp3")]
fn unsupported_mp3_error(
    path: &Path,
    target_sample_rate: u32,
    err: anyhow::Error,
) -> anyhow::Error {
    anyhow::anyhow!(
        "Sound file '{}' could not be decoded as MP3.\n\nThe optional `mp3` feature uses ffmpeg at asset-load time; install ffmpeg or convert the file:\n    ffmpeg -i {} -ac 2 -ar {target_sample_rate} {}\n\nDetails: {err}",
        path.display(),
        path.display(),
        path.with_extension("ogg").display(),
    )
}

fn normalize_sound(
    sound: Sound,
    target_channels: u16,
    target_sample_rate: u32,
) -> anyhow::Result<Sound> {
    let sound = convert_channels(sound, target_channels)?;
    resample_linear(sound, target_sample_rate)
}

fn convert_channels(sound: Sound, target_channels: u16) -> anyhow::Result<Sound> {
    if target_channels == 0 {
        anyhow::bail!("target channel count must be nonzero");
    }
    if sound.channels == target_channels {
        return Ok(sound);
    }
    if sound.channels == 0 {
        anyhow::bail!("channel count must be nonzero");
    }
    if !Mixer::sound_is_well_formed(&sound) {
        anyhow::bail!(
            "sound has {} samples, which is not a whole number of {}-channel frames",
            sound.samples.len(),
            sound.channels,
        );
    }

    let frames = sound.samples.len() / sound.channels as usize;
    let mut samples = Vec::with_capacity(frames * target_channels as usize);
    match (sound.channels, target_channels) {
        (1, 2) => {
            for sample in sound.samples {
                samples.push(sample);
                samples.push(sample);
            }
        }
        (2, 1) => {
            for frame in sound.samples.chunks_exact(2) {
                samples.push((frame[0] + frame[1]) * 0.5);
            }
        }
        (channels, _) if channels > 2 => {
            anyhow::bail!(
                "unsupported WAV channel count {channels}; supported today: mono or stereo"
            );
        }
        (_, channels) if channels > 2 => {
            anyhow::bail!(
                "unsupported mixer channel count {channels}; supported today: mono or stereo"
            );
        }
        (source, target) => {
            anyhow::bail!("cannot convert {source}-channel audio to {target} channels");
        }
    }

    Ok(Sound::new(samples, target_channels, sound.sample_rate))
}

fn resample_linear(sound: Sound, target_sample_rate: u32) -> anyhow::Result<Sound> {
    if target_sample_rate == 0 {
        anyhow::bail!("target sample rate must be nonzero");
    }
    if sound.sample_rate == target_sample_rate {
        return Ok(sound);
    }
    if sound.sample_rate == 0 {
        anyhow::bail!("sound sample rate must be nonzero");
    }
    if !Mixer::sound_is_well_formed(&sound) {
        anyhow::bail!(
            "sound has {} samples, which is not a whole number of {}-channel frames",
            sound.samples.len(),
            sound.channels,
        );
    }

    let channels = sound.channels as usize;
    let source_frames = sound.samples.len() / channels;
    if source_frames == 0 {
        anyhow::bail!("sound must contain at least one frame");
    }
    let target_frames = ((source_frames as f64 * target_sample_rate as f64
        / sound.sample_rate as f64)
        .round() as usize)
        .max(1);
    let mut samples = Vec::with_capacity(target_frames * channels);

    for frame in 0..target_frames {
        let source_pos = frame as f64 * sound.sample_rate as f64 / target_sample_rate as f64;
        let base = source_pos.floor() as usize;
        let next = (base + 1).min(source_frames - 1);
        let t = (source_pos - base as f64) as f32;
        let base = base.min(source_frames - 1);

        for channel in 0..channels {
            let a = sound.samples[base * channels + channel];
            let b = sound.samples[next * channels + channel];
            samples.push(a + (b - a) * t);
        }
    }

    Ok(Sound::new(samples, sound.channels, target_sample_rate))
}

fn decode_wav_sound(bytes: &[u8]) -> anyhow::Result<Sound> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        anyhow::bail!("expected RIFF/WAVE header");
    }

    let mut fmt: Option<WavFormat> = None;
    let mut data: Option<&[u8]> = None;
    let mut offset = 12usize;
    while offset + 8 <= bytes.len() {
        let id = &bytes[offset..offset + 4];
        let size = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap()) as usize;
        offset += 8;
        if offset + size > bytes.len() {
            anyhow::bail!("chunk {:?} extends past end of file", id);
        }
        let chunk = &bytes[offset..offset + size];
        match id {
            b"fmt " => fmt = Some(parse_wav_format(chunk)?),
            b"data" => data = Some(chunk),
            _ => {}
        }
        offset += size + (size % 2);
    }

    let fmt = fmt.ok_or_else(|| anyhow::anyhow!("missing fmt chunk"))?;
    let data = data.ok_or_else(|| anyhow::anyhow!("missing data chunk"))?;
    if fmt.channels == 0 {
        anyhow::bail!("channel count must be nonzero");
    }
    if fmt.sample_rate == 0 {
        anyhow::bail!("sample rate must be nonzero");
    }
    if fmt.block_align == 0 || data.len() % fmt.block_align as usize != 0 {
        anyhow::bail!(
            "data chunk is not aligned to {} byte frames",
            fmt.block_align
        );
    }

    let samples = match (fmt.audio_format, fmt.bits_per_sample) {
        (1, 16) => data
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]) as f32 / 32768.0)
            .collect(),
        (3, 32) => data
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect(),
        (format, bits) => {
            anyhow::bail!(
                "unsupported WAV format {format} with {bits} bits per sample; supported: PCM16 and float32"
            );
        }
    };

    Ok(Sound::new(samples, fmt.channels, fmt.sample_rate))
}

#[cfg(feature = "ogg")]
fn decode_ogg_sound(bytes: &[u8]) -> anyhow::Result<Sound> {
    use lewton::inside_ogg::OggStreamReader;

    let mut reader = OggStreamReader::new(Cursor::new(bytes))
        .map_err(|err| anyhow::anyhow!("invalid OGG stream: {err}"))?;
    let channels = u16::from(reader.ident_hdr.audio_channels);
    let sample_rate = reader.ident_hdr.audio_sample_rate;
    if channels == 0 {
        anyhow::bail!("OGG stream reports zero channels");
    }
    if sample_rate == 0 {
        anyhow::bail!("OGG stream reports a zero sample rate");
    }

    let mut samples = Vec::new();
    while let Some(packet) = reader
        .read_dec_packet_itl()
        .map_err(|err| anyhow::anyhow!("failed to decode OGG packet: {err}"))?
    {
        samples.extend(packet.into_iter().map(|sample| sample as f32 / 32768.0));
    }
    if samples.is_empty() {
        anyhow::bail!("OGG stream contains no decoded audio samples");
    }
    Ok(Sound::new(samples, channels, sample_rate))
}

#[derive(Clone, Copy)]
struct WavFormat {
    audio_format: u16,
    channels: u16,
    sample_rate: u32,
    block_align: u16,
    bits_per_sample: u16,
}

fn parse_wav_format(chunk: &[u8]) -> anyhow::Result<WavFormat> {
    if chunk.len() < 16 {
        anyhow::bail!("fmt chunk must be at least 16 bytes");
    }
    Ok(WavFormat {
        audio_format: u16::from_le_bytes([chunk[0], chunk[1]]),
        channels: u16::from_le_bytes([chunk[2], chunk[3]]),
        sample_rate: u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]),
        block_align: u16::from_le_bytes([chunk[12], chunk[13]]),
        bits_per_sample: u16::from_le_bytes([chunk[14], chunk[15]]),
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::AtomicU64;

    use crossbeam_queue::ArrayQueue;

    use super::{
        AudioCommand, Mixer, MixerCallback, PlayResult, Sound, SoundFormat, convert_channels,
        crossed_power_of_two, decode_wav_sound, detect_sound_format, normalize_sound,
        resample_linear, sanitize_volume,
    };

    #[cfg(not(feature = "ogg"))]
    use super::decode_file_sound;

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
    fn convert_channels_duplicates_mono_to_stereo() {
        let sound = Sound::new(vec![0.25, -0.5], 1, 48_000);
        let sound = convert_channels(sound, 2).unwrap();

        assert_eq!(sound.channels, 2);
        assert_eq!(sound.samples, vec![0.25, 0.25, -0.5, -0.5]);
    }

    #[test]
    fn convert_channels_rejects_surround_wav() {
        let sound = Sound::new(vec![0.0; 6], 3, 48_000);
        let err = convert_channels(sound, 2).unwrap_err();

        assert!(err.to_string().contains("mono or stereo"));
    }

    #[test]
    fn resample_linear_changes_sample_rate_and_frame_count() {
        let sound = Sound::new(vec![0.0, 1.0, 0.0, 1.0], 1, 4);
        let sound = resample_linear(sound, 8).unwrap();

        assert_eq!(sound.sample_rate, 8);
        assert_eq!(sound.channels, 1);
        assert_eq!(sound.samples.len(), 8);
        assert!(sound.samples.iter().all(|sample| sample.is_finite()));
    }

    #[test]
    fn normalize_sound_prepares_mono_44100_for_stereo_48000_mixer() {
        let wav = test_wav_pcm16_with_format(1, 44_100, &[0, i16::MAX]);
        let decoded = decode_wav_sound(&wav).unwrap();
        let normalized = normalize_sound(decoded, 2, 48_000).unwrap();

        assert_eq!(normalized.channels, 2);
        assert_eq!(normalized.sample_rate, 48_000);
        assert!(normalized.samples.len().is_multiple_of(2));

        let mut mixer = Mixer::new();
        mixer.set_output_format(2, 48_000).unwrap();
        assert!(mixer.add_sound(normalized).is_ok());
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
    fn decode_wav_sound_accepts_pcm16() {
        let wav = test_wav_pcm16(&[0, i16::MAX, i16::MIN, 0]);
        let sound = decode_wav_sound(&wav).unwrap();

        assert_eq!(sound.channels, 2);
        assert_eq!(sound.sample_rate, 48_000);
        assert_eq!(sound.samples.len(), 4);
        assert_eq!(sound.samples[0], 0.0);
        assert!(sound.samples[1] > 0.99);
        assert_eq!(sound.samples[2], -1.0);
    }

    #[test]
    fn sound_format_detection_uses_magic_bytes_then_extension() {
        assert_eq!(
            detect_sound_format(std::path::Path::new("sound.bin"), b"RIFFxxxxWAVE"),
            SoundFormat::Wav
        );
        assert_eq!(
            detect_sound_format(std::path::Path::new("sound.bin"), b"OggS\0\x02"),
            SoundFormat::Ogg
        );
        assert_eq!(
            detect_sound_format(std::path::Path::new("theme.ogg"), b"not a header"),
            SoundFormat::Ogg
        );
        assert_eq!(
            detect_sound_format(std::path::Path::new("theme.mp3"), b"not a header"),
            SoundFormat::Mp3
        );
        assert_eq!(
            detect_sound_format(std::path::Path::new("sound.data"), b"not a header"),
            SoundFormat::Unknown
        );
    }

    #[cfg(not(feature = "ogg"))]
    #[test]
    fn ogg_files_explain_how_to_enable_or_convert_when_feature_is_disabled() {
        let error = decode_file_sound(
            std::path::Path::new("music/theme.ogg"),
            b"OggS\0\x02",
            2,
            48_000,
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("OGG audio requires the `ogg` feature"));
        assert!(error.contains("ffmpeg -i music/theme.ogg"));
        assert!(error.contains("music/theme.wav"));
    }

    #[cfg(not(feature = "mp3"))]
    #[test]
    fn mp3_files_explain_how_to_enable_or_convert_when_feature_is_disabled() {
        let error = decode_file_sound(
            std::path::Path::new("music/theme.mp3"),
            b"ID3\x04\0\0",
            2,
            48_000,
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("MP3 audio requires the optional `mp3` feature"));
        assert!(error.contains("ffmpeg -i music/theme.mp3"));
    }

    fn test_wav_pcm16(samples: &[i16]) -> Vec<u8> {
        test_wav_pcm16_with_format(2, 48_000, samples)
    }

    fn test_wav_pcm16_with_format(channels: u16, sample_rate: u32, samples: &[i16]) -> Vec<u8> {
        let data_len = samples.len() * 2;
        let riff_len = 4 + (8 + 16) + (8 + data_len);
        let byte_rate = sample_rate * channels as u32 * 2;
        let block_align = channels * 2;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(riff_len as u32).to_le_bytes());
        bytes.extend_from_slice(b"WAVE");
        bytes.extend_from_slice(b"fmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&channels.to_le_bytes());
        bytes.extend_from_slice(&sample_rate.to_le_bytes());
        bytes.extend_from_slice(&byte_rate.to_le_bytes());
        bytes.extend_from_slice(&block_align.to_le_bytes());
        bytes.extend_from_slice(&16u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&(data_len as u32).to_le_bytes());
        for sample in samples {
            bytes.extend_from_slice(&sample.to_le_bytes());
        }
        bytes
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
            .add_sound(super::loop_sine_sound(110.0, 1.0, sample_rate, channels).unwrap())
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
    fn music_replaces_existing_music_and_can_stop() {
        let mut mixer = Mixer::new();
        let first = mixer.add_sound(Sound::new(vec![0.25], 1, 48_000)).unwrap();
        let second = mixer.add_sound(Sound::new(vec![0.5], 1, 48_000)).unwrap();
        let sfx = mixer.add_sound(Sound::new(vec![0.75], 1, 48_000)).unwrap();

        mixer.play(sfx, 1.0, true);
        mixer.play_music(first, 0.5, None);
        mixer.play_music(second, 0.5, None);

        assert_eq!(mixer.voice_count(), 2);
        assert!(mixer.voices.iter().any(|voice| !voice.music));
        assert_eq!(
            mixer
                .voices
                .iter()
                .filter(|voice| voice.music)
                .map(|voice| voice.sound_id)
                .collect::<Vec<_>>(),
            vec![second]
        );

        mixer.stop_music();

        assert_eq!(mixer.voice_count(), 1);
        assert!(mixer.voices.iter().all(|voice| !voice.music));
    }

    #[test]
    fn volume_groups_scale_sfx_and_music_independently() {
        let mut mixer = Mixer::new();
        let sound = mixer.add_sound(Sound::new(vec![1.0], 1, 48_000)).unwrap();
        mixer.set_master_volume(0.5);
        mixer.set_sfx_volume(0.4);
        mixer.play(sound, 1.0, true);

        let mut out = [0.0; 1];
        mixer.mix_into(&mut out);
        assert_eq!(out, [0.2]);

        let mut mixer = Mixer::new();
        let music = mixer.add_sound(Sound::new(vec![1.0], 1, 48_000)).unwrap();
        mixer.set_master_volume(0.5);
        mixer.set_music_volume(0.4);
        mixer.play_music(music, 1.0, None);

        let mut out = [0.0; 1];
        mixer.mix_into(&mut out);
        assert_eq!(out, [0.2]);
    }

    #[test]
    fn named_bus_multiplies_the_standard_sfx_group() {
        let mut mixer = Mixer::new();
        let sound = mixer.add_sound(Sound::new(vec![1.0], 1, 48_000)).unwrap();
        mixer.set_sfx_volume(0.8);
        mixer.set_bus_volume(3, 0.5);
        mixer.play_on_bus(sound, 1.0, true, Some(3));

        let mut out = [0.0; 1];
        mixer.mix_into(&mut out);

        assert_eq!(out, [0.4]);
    }

    #[test]
    fn crossfade_keeps_old_music_until_the_new_track_has_faded_in() {
        let mut mixer = Mixer::new();
        mixer.set_output_format(1, 10).unwrap();
        let first = mixer.add_sound(Sound::new(vec![1.0], 1, 10)).unwrap();
        let second = mixer.add_sound(Sound::new(vec![0.5], 1, 10)).unwrap();
        mixer.play_music(first, 1.0, None);

        assert_eq!(mixer.crossfade_music(second, 1.0, 1.0), PlayResult::Started);
        assert_eq!(mixer.voices.iter().filter(|voice| voice.music).count(), 2);

        let mut out = [0.0; 10];
        mixer.mix_into(&mut out);

        assert_eq!(out, [0.5; 10]);
        assert_eq!(mixer.voices.iter().filter(|voice| voice.music).count(), 1);
        assert_eq!(mixer.voices[0].sound_id, second);
    }

    #[test]
    fn music_can_pause_resume_and_fade() {
        let mut mixer = Mixer::new();
        mixer.set_output_format(1, 10).unwrap();
        let music = mixer.add_sound(Sound::new(vec![0.5], 1, 10)).unwrap();
        mixer.play_music(music, 1.0, None);

        mixer.pause_music();
        let mut out = [0.0; 1];
        mixer.mix_into(&mut out);
        assert_eq!(out, [0.0]);

        mixer.resume_music();
        mixer.fade_music_to(0.0, 1.0);
        let mut out = [0.0; 5];
        mixer.mix_into(&mut out);
        assert_eq!(out, [0.25; 5]);

        let mut out = [0.0; 5];
        mixer.mix_into(&mut out);
        assert_eq!(out, [0.0; 5]);
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
            assert_eq!(mixer.play(sound_id, 1.0, true), PlayResult::Started);
        }

        assert_eq!(mixer.voice_count(), super::MAX_VOICES);

        assert_eq!(
            mixer.play(sound_id, 1.0, true),
            PlayResult::DroppedVoiceLimit
        );

        assert_eq!(mixer.voice_count(), super::MAX_VOICES);
    }

    #[test]
    fn voice_cap_drop_increments_counter() {
        let mut mixer = Mixer::new();
        let sound_id = mixer.add_sound(Sound::new(vec![0.25], 1, 48_000)).unwrap();

        for _ in 0..super::MAX_VOICES {
            mixer.play(sound_id, 1.0, true);
        }
        assert_eq!(mixer.dropped_voice_count(), 0);

        mixer.play(sound_id, 1.0, true);
        mixer.play(sound_id, 1.0, true);

        assert_eq!(mixer.dropped_voice_count(), 2);
    }

    #[test]
    fn invalid_sound_id_does_not_increment_voice_cap_counter() {
        let mut mixer = Mixer::new();

        assert_eq!(mixer.play(42, 1.0, false), PlayResult::InvalidSoundId);

        assert_eq!(mixer.dropped_voice_count(), 0);
    }

    #[test]
    fn counter_can_be_read_from_shared_handle() {
        // Mirrors how `AudioSystem` reads the mixer's counter from the main thread
        // while the callback owns the `Mixer`.
        let mut mixer = Mixer::new();
        let handle = mixer.dropped_voices_handle();
        let sound_id = mixer.add_sound(Sound::new(vec![0.25], 1, 48_000)).unwrap();

        for _ in 0..(super::MAX_VOICES + 3) {
            mixer.play(sound_id, 1.0, true);
        }

        assert_eq!(handle.load(std::sync::atomic::Ordering::Relaxed), 3);
    }

    #[test]
    fn sine_sound_rejects_nan_frequency() {
        assert!(super::sine_sound(f32::NAN, 0.1, 48_000, 1).is_err());
        assert!(super::sine_sound(f32::INFINITY, 0.1, 48_000, 1).is_err());
        assert!(super::sine_sound(-440.0, 0.1, 48_000, 1).is_err());
    }

    #[test]
    fn sine_sound_rejects_zero_duration() {
        assert!(super::sine_sound(440.0, 0.0, 48_000, 1).is_err());
        assert!(super::sine_sound(440.0, -1.0, 48_000, 1).is_err());
    }

    #[test]
    fn sine_sound_rejects_excessive_duration() {
        // 48 kHz for an hour is well past MAX_GENERATED_FRAMES.
        assert!(super::sine_sound(440.0, 3600.0, 48_000, 1).is_err());
    }

    #[test]
    fn sine_sound_rejects_zero_rate_or_channels() {
        assert!(super::sine_sound(440.0, 0.1, 0, 1).is_err());
        assert!(super::sine_sound(440.0, 0.1, 48_000, 0).is_err());
    }

    #[test]
    fn sine_sound_rejects_excessive_channels() {
        // A huge channel count must be rejected before allocating the buffer.
        assert!(super::sine_sound(440.0, 0.1, 48_000, u16::MAX).is_err());
        assert!(super::loop_sine_sound(440.0, 0.1, 48_000, u16::MAX).is_err());
        assert!(super::sine_sound(440.0, 0.1, 48_000, super::MAX_GENERATED_CHANNELS).is_ok());
    }

    #[test]
    fn sine_sound_rejects_frequency_above_nyquist() {
        // Above Nyquist (sample_rate / 2) the tone only aliases; reject it (and an
        // extreme finite frequency that would otherwise overflow the phase math).
        assert!(super::sine_sound(30_000.0, 0.1, 48_000, 1).is_err());
        assert!(super::loop_sine_sound(f32::MAX, 0.1, 48_000, 1).is_err());
        assert!(super::sine_sound(24_000.0, 0.1, 48_000, 1).is_ok());
    }

    #[test]
    fn loop_sine_sound_samples_are_finite() {
        let sound = super::loop_sine_sound(110.0, 1.0, 48_000, 2).unwrap();
        assert!(sound.samples.iter().all(|s| s.is_finite()));
    }

    #[test]
    fn sine_sound_accepts_valid_input() {
        let sound = super::sine_sound(440.0, 0.1, 48_000, 2).unwrap();
        let frames = (0.1f32 * 48_000.0) as usize;
        assert_eq!(sound.samples.len(), frames * 2);
        assert!(sound.samples.iter().all(|s| s.is_finite()));
    }

    #[test]
    fn one_shot_sound_still_decays() {
        // The one-shot envelope means the late half of the tone is quieter than
        // the early half.
        let sound = super::sine_sound(440.0, 0.2, 48_000, 1).unwrap();
        let half = sound.samples.len() / 2;
        let peak = |s: &[f32]| s.iter().fold(0.0f32, |m, v| m.max(v.abs()));
        let early = peak(&sound.samples[..half]);
        let late = peak(&sound.samples[half..]);
        assert!(late < early, "expected decay: early={early}, late={late}");
    }

    #[test]
    fn loop_sound_start_and_end_are_continuous() {
        // A seamless loop must have no jump at the wrap seam: the step from the
        // last sample back to the first must be no larger than the largest step
        // between adjacent interior samples.
        let sound = super::loop_sine_sound(110.0, 1.0, 48_000, 1).unwrap();
        let s = &sound.samples;
        assert!(s.len() > 2);

        let max_interior_step = s
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .fold(0.0f32, f32::max);
        let wrap_step = (s[0] - s[s.len() - 1]).abs();

        assert!(
            wrap_step <= max_interior_step * 1.5 + 1e-6,
            "loop seam discontinuity: wrap={wrap_step}, max_interior={max_interior_step}"
        );
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
                bus: None,
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
                    bus: None,
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
