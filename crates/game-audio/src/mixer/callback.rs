use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crossbeam_queue::ArrayQueue;
use sdl3::audio::{AudioCallback, AudioStream};

use super::Mixer;
use super::command::AudioCommand;

pub(super) const AUDIO_SCRATCH_SAMPLES: usize = 4096;

pub(super) struct MixerCallback {
    pub(super) mixer: Mixer,
    pub(super) commands: Arc<ArrayQueue<AudioCommand>>,
    pub(super) scratch: Vec<f32>,
    // Incremented (never read) here on every `put_data_f32` failure. The main
    // thread reads/logs it via `AudioSystem::poll_dropped_frames`, so this
    // realtime callback never logs, locks, or allocates for diagnostics.
    pub(super) put_failures: Arc<AtomicU64>,
}

impl MixerCallback {
    pub(super) fn drain_commands(&mut self) {
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
                AudioCommand::PlayStreamedMusic {
                    stream_id,
                    volume,
                    fade_in_seconds,
                } => {
                    self.mixer
                        .play_streamed_music(stream_id, volume, fade_in_seconds);
                }
                AudioCommand::CrossfadeMusic {
                    sound_id,
                    volume,
                    duration_seconds,
                } => {
                    self.mixer
                        .crossfade_music(sound_id, volume, duration_seconds);
                }
                AudioCommand::CrossfadeStreamedMusic {
                    stream_id,
                    volume,
                    duration_seconds,
                } => {
                    self.mixer
                        .crossfade_streamed_music(stream_id, volume, duration_seconds);
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
