use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crossbeam_queue::ArrayQueue;
use game_core::backend::{
    AudioBackend, AudioCommand as BackendAudioCommand, SoundHandle, SoundLoadRequest,
};
use sdl3::audio::{AudioFormat, AudioSpec, AudioStreamWithCallback};

use super::callback::{AUDIO_SCRATCH_SAMPLES, MixerCallback};
use super::command::{AUDIO_COMMAND_QUEUE_CAPACITY, AudioCommand};
use super::decode::load_file_sound;
use super::stream::{MusicStream, StreamId, open_streamed_music, validate_streamed_music};
use super::{MAX_NAMED_BUSES, MAX_VOICES, Mixer, SoundId, crossed_power_of_two, sine_sound};

pub struct AudioSystem {
    commands: Arc<ArrayQueue<AudioCommand>>,
    blip_sound: SoundId,
    sound_sources: HashMap<SoundHandle, AudioSource>,
    // Owns the reader workers for as long as the audio system lives. The mixer
    // holds only their lock-free shared states.
    _streamed_tracks: Vec<MusicStream>,
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
    output_channels: u16,
    output_sample_rate: u32,
    stream: AudioStreamWithCallback<MixerCallback>,
}

#[derive(Clone, Copy)]
enum AudioSource {
    Static(SoundId),
    Streamed(StreamId),
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

        let mut sound_sources = HashMap::new();
        let mut streamed_tracks = Vec::new();
        for (handle, request) in sound_loads {
            let source = match request {
                SoundLoadRequest::Generated { .. } => AudioSource::Static(blip_sound),
                SoundLoadRequest::File { path } => {
                    let sound = load_file_sound(
                        &asset_root.join(&path),
                        channels as u16,
                        sample_rate as u32,
                    )?;
                    AudioSource::Static(mixer.add_sound(sound)?)
                }
                SoundLoadRequest::StreamedFile { path } => {
                    let track = open_streamed_music(
                        &asset_root.join(&path),
                        channels as u16,
                        sample_rate as u32,
                    )?;
                    let stream_id = mixer.add_stream(track.state());
                    streamed_tracks.push(track);
                    AudioSource::Streamed(stream_id)
                }
            };
            sound_sources.insert(handle, source);
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
            sound_sources,
            _streamed_tracks: streamed_tracks,
            bus_ids: Mutex::new(HashMap::new()),
            put_failures,
            reported_failures: AtomicU64::new(0),
            command_push_failures: AtomicU64::new(0),
            dropped_voices,
            reported_voice_drops: AtomicU64::new(0),
            output_channels: channels as u16,
            output_sample_rate: sample_rate as u32,
            stream,
        })
    }

    /// Reloads file-backed sounds while retaining their public `SoundHandle`s.
    /// Existing voices using a replaced sample are stopped; subsequent plays use
    /// the freshly decoded sound. Generated sounds are intentionally unchanged.
    pub fn reload_file_sounds(
        &mut self,
        asset_root: &Path,
        sound_loads: &[(SoundHandle, SoundLoadRequest)],
    ) -> anyhow::Result<usize> {
        let mut replacements = Vec::new();
        let mut streams_to_restart = Vec::new();
        for (handle, request) in sound_loads {
            match request {
                SoundLoadRequest::Generated { .. } => {}
                SoundLoadRequest::File { path } => {
                    let sound = load_file_sound(
                        &asset_root.join(path),
                        self.output_channels,
                        self.output_sample_rate,
                    )?;
                    let AudioSource::Static(id) =
                        self.sound_sources.get(handle).copied().ok_or_else(|| {
                            anyhow::anyhow!("sound reload has no source for {handle:?}")
                        })?
                    else {
                        anyhow::bail!("sound reload expected static source for {handle:?}");
                    };
                    replacements.push((id, sound));
                }
                SoundLoadRequest::StreamedFile { path } => {
                    let AudioSource::Streamed(id) =
                        self.sound_sources.get(handle).copied().ok_or_else(|| {
                            anyhow::anyhow!("sound reload has no source for {handle:?}")
                        })?
                    else {
                        anyhow::bail!("sound reload expected streamed source for {handle:?}");
                    };
                    // Validate the replacement header before asking its worker to
                    // restart. The worker will then reopen at the new generation.
                    validate_streamed_music(
                        &asset_root.join(path),
                        self.output_channels,
                        self.output_sample_rate,
                    )?;
                    streams_to_restart.push(id);
                }
            }
        }
        if replacements.is_empty() && streams_to_restart.is_empty() {
            return Ok(0);
        }
        let mut callback = self
            .stream
            .lock()
            .ok_or_else(|| anyhow::anyhow!("could not lock SDL audio stream for reload"))?;
        for (id, sound) in replacements {
            callback.mixer.replace_sound(id, sound)?;
        }
        for id in streams_to_restart {
            callback.mixer.restart_stream(id)?;
        }
        Ok(sound_loads
            .iter()
            .filter(|(_, request)| {
                matches!(
                    request,
                    SoundLoadRequest::File { .. } | SoundLoadRequest::StreamedFile { .. }
                )
            })
            .count())
    }

    pub fn play_blip(&self) {
        self.enqueue_play(self.blip_sound, 0.8, false, None);
    }

    pub fn play(&self, sound: SoundHandle, volume: f32, looping: bool) {
        self.play_on_bus(sound, volume, looping, None);
    }

    pub fn play_on_bus(&self, sound: SoundHandle, volume: f32, looping: bool, bus: Option<&str>) {
        let Some(source) = self.sound_sources.get(&sound).copied() else {
            log::warn!("ignoring play request for unknown sound handle {:?}", sound);
            return;
        };
        let AudioSource::Static(sound_id) = source else {
            log::warn!(
                "ignoring SFX play request for streamed music handle {:?}",
                sound
            );
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
        let Some(source) = self.sound_sources.get(&sound).copied() else {
            log::warn!(
                "ignoring crossfade_music request for unknown sound handle {:?}",
                sound
            );
            return;
        };
        match source {
            AudioSource::Static(sound_id) => {
                self.enqueue_audio_command(AudioCommand::CrossfadeMusic {
                    sound_id,
                    volume,
                    duration_seconds,
                })
            }
            AudioSource::Streamed(stream_id) => {
                self.enqueue_audio_command(AudioCommand::CrossfadeStreamedMusic {
                    stream_id,
                    volume,
                    duration_seconds,
                })
            }
        }
    }

    fn play_music_with_fade(&self, sound: SoundHandle, volume: f32, fade_in_seconds: Option<f32>) {
        let Some(source) = self.sound_sources.get(&sound).copied() else {
            log::warn!(
                "ignoring play_music request for unknown sound handle {:?}",
                sound
            );
            return;
        };
        match source {
            AudioSource::Static(sound_id) => self.enqueue_audio_command(AudioCommand::PlayMusic {
                sound_id,
                volume,
                fade_in_seconds,
            }),
            AudioSource::Streamed(stream_id) => {
                self.enqueue_audio_command(AudioCommand::PlayStreamedMusic {
                    stream_id,
                    volume,
                    fade_in_seconds,
                })
            }
        }
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

    /// Submits a content-facing audio command to the lock-free mixer queue.
    pub fn submit_command(&self, command: BackendAudioCommand) {
        match command {
            BackendAudioCommand::Play {
                sound,
                volume,
                looping,
                bus,
            } => self.play_on_bus(sound, volume, looping, bus.as_deref()),
            BackendAudioCommand::PlayMusic {
                sound,
                volume,
                fade_in_seconds,
            } => match fade_in_seconds {
                Some(duration_seconds) => self.play_music_fade_in(sound, volume, duration_seconds),
                None => self.play_music(sound, volume),
            },
            BackendAudioCommand::CrossfadeMusic {
                sound,
                volume,
                duration_seconds,
            } => self.crossfade_music(sound, volume, duration_seconds),
            BackendAudioCommand::StopMusic => self.stop_music(),
            BackendAudioCommand::PauseMusic => self.pause_music(),
            BackendAudioCommand::ResumeMusic => self.resume_music(),
            BackendAudioCommand::SetMasterVolume { volume } => self.set_master_volume(volume),
            BackendAudioCommand::SetSfxVolume { volume } => self.set_sfx_volume(volume),
            BackendAudioCommand::SetMusicVolume { volume } => self.set_music_volume(volume),
            BackendAudioCommand::SetBusVolume { bus, volume } => self.set_bus_volume(&bus, volume),
            BackendAudioCommand::FadeMusicTo {
                volume,
                duration_seconds,
            } => self.fade_music_to(volume, duration_seconds),
        }
    }

    #[allow(dead_code)]
    pub fn dropped_voices(&self) -> u64 {
        self.dropped_voices.load(Ordering::Relaxed)
    }
}

impl AudioBackend for AudioSystem {
    fn reload_sounds(
        &mut self,
        asset_root: &Path,
        sounds: &[(SoundHandle, SoundLoadRequest)],
    ) -> anyhow::Result<usize> {
        self.reload_file_sounds(asset_root, sounds)
    }

    fn submit(&self, command: BackendAudioCommand) {
        self.submit_command(command);
    }

    fn poll_diagnostics(&self) {
        self.poll_dropped_frames();
        self.poll_dropped_voices();
    }
}
