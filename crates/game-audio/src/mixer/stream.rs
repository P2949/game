use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam_queue::ArrayQueue;

use super::decode::parse_wav_format;

/// Four seconds of stereo 48 kHz audio: bounded regardless of source track size.
const STREAM_BUFFER_SAMPLES: usize = 48_000 * 2 * 4;
const STREAM_READ_BYTES: usize = 16 * 1024;
const STREAM_WORKER_IDLE: Duration = Duration::from_millis(2);

pub(crate) type StreamId = usize;

#[derive(Clone, Copy, Debug)]
pub(crate) struct StreamSample {
    pub(crate) generation: u64,
    pub(crate) value: f32,
}

pub(crate) struct StreamState {
    pub(crate) samples: ArrayQueue<StreamSample>,
    pub(crate) generation: AtomicU64,
    running: AtomicBool,
    pub(crate) underruns: AtomicU64,
}

impl StreamState {
    pub(crate) fn new() -> Self {
        Self {
            samples: ArrayQueue::new(STREAM_BUFFER_SAMPLES),
            generation: AtomicU64::new(0),
            running: AtomicBool::new(true),
            underruns: AtomicU64::new(0),
        }
    }

    pub(crate) fn restart(&self) -> u64 {
        self.generation.fetch_add(1, Ordering::AcqRel) + 1
    }
}

/// A bounded music reader. Its worker performs file I/O and PCM conversion away
/// from SDL's callback; the callback only pops predecoded samples from the
/// lock-free queue.
pub(crate) struct MusicStream {
    pub(crate) state: Arc<StreamState>,
    worker: Option<JoinHandle<()>>,
}

impl MusicStream {
    pub(crate) fn state(&self) -> Arc<StreamState> {
        Arc::clone(&self.state)
    }
}

impl Drop for MusicStream {
    fn drop(&mut self) {
        self.state.running.store(false, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

pub(super) struct StreamedPcm16Wav {
    file: File,
    data_start: u64,
    data_end: u64,
}

impl StreamedPcm16Wav {
    pub(super) fn open(
        path: &Path,
        target_channels: u16,
        target_sample_rate: u32,
    ) -> anyhow::Result<Self> {
        let mut file = File::open(path).map_err(|error| {
            anyhow::anyhow!(
                "failed to open streamed music '{}': {error}",
                path.display()
            )
        })?;
        let mut header = [0_u8; 12];
        file.read_exact(&mut header).map_err(|error| {
            streamed_music_error(path, format!("could not read a RIFF/WAVE header: {error}"))
        })?;
        if &header[..4] != b"RIFF" || &header[8..] != b"WAVE" {
            return Err(streamed_music_error(
                path,
                "detected a non-WAV file".to_owned(),
            ));
        }

        let mut format = None;
        let mut data = None;
        loop {
            let mut chunk_header = [0_u8; 8];
            if file.read_exact(&mut chunk_header).is_err() {
                break;
            }
            let size = u32::from_le_bytes(chunk_header[4..].try_into().unwrap()) as u64;
            let chunk_start = file.stream_position().map_err(anyhow::Error::from)?;
            match &chunk_header[..4] {
                b"fmt " => {
                    if size > 4096 {
                        return Err(streamed_music_error(
                            path,
                            format!("fmt chunk is unexpectedly large ({size} bytes)"),
                        ));
                    }
                    let mut chunk = vec![0_u8; size as usize];
                    file.read_exact(&mut chunk).map_err(|error| {
                        streamed_music_error(path, format!("could not read fmt chunk: {error}"))
                    })?;
                    format = Some(parse_wav_format(&chunk).map_err(|error| {
                        streamed_music_error(path, format!("invalid WAV fmt chunk: {error}"))
                    })?);
                }
                b"data" => {
                    data = Some((chunk_start, size));
                    break;
                }
                _ => {
                    let skip = size + size % 2;
                    file.seek(SeekFrom::Current(skip as i64))
                        .map_err(anyhow::Error::from)?;
                }
            }
            if &chunk_header[..4] == b"fmt " && !size.is_multiple_of(2) {
                file.seek(SeekFrom::Current(1))
                    .map_err(anyhow::Error::from)?;
            }
        }

        let format =
            format.ok_or_else(|| streamed_music_error(path, "missing fmt chunk".to_owned()))?;
        let (data_start, data_len) =
            data.ok_or_else(|| streamed_music_error(path, "missing data chunk".to_owned()))?;
        if format.audio_format != 1
            || format.bits_per_sample != 16
            || format.channels != target_channels
            || format.sample_rate != target_sample_rate
            || format.block_align != target_channels * 2
        {
            return Err(streamed_music_error(
                path,
                format!(
                    "detected WAV format {} / {} channels / {} Hz / {} bits (streaming needs PCM16 / {target_channels} channels / {target_sample_rate} Hz)",
                    format.audio_format,
                    format.channels,
                    format.sample_rate,
                    format.bits_per_sample,
                ),
            ));
        }
        if data_len == 0 || data_len % u64::from(format.block_align) != 0 {
            return Err(streamed_music_error(
                path,
                "data chunk is empty or is not aligned to complete PCM frames".to_owned(),
            ));
        }
        let data_end = data_start.checked_add(data_len).ok_or_else(|| {
            streamed_music_error(
                path,
                "data chunk length overflows the file offset".to_owned(),
            )
        })?;
        file.seek(SeekFrom::Start(data_start))
            .map_err(anyhow::Error::from)?;
        Ok(Self {
            file,
            data_start,
            data_end,
        })
    }

    fn rewind(&mut self) -> std::io::Result<()> {
        self.file.seek(SeekFrom::Start(self.data_start))?;
        Ok(())
    }

    fn read_chunk(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        let position = self.file.stream_position()?;
        if position >= self.data_end {
            return Ok(0);
        }
        let remaining = (self.data_end - position) as usize;
        let length = bytes.len().min(remaining);
        self.file.read(&mut bytes[..length])
    }
}

fn streamed_music_error(path: &Path, details: impl std::fmt::Display) -> anyhow::Error {
    anyhow::anyhow!(
        "Streamed music '{}' {details}.\n\nStreaming currently supports a 48 kHz stereo PCM16 WAV file so it can use a bounded background reader. Convert with:\n    ffmpeg -i {} -ac 2 -ar 48000 -c:a pcm_s16le {}",
        path.display(),
        path.display(),
        path.with_extension("wav").display(),
    )
}

pub(super) fn validate_streamed_music(
    path: &Path,
    target_channels: u16,
    target_sample_rate: u32,
) -> anyhow::Result<()> {
    StreamedPcm16Wav::open(path, target_channels, target_sample_rate).map(|_| ())
}

pub(super) fn open_streamed_music(
    path: &Path,
    target_channels: u16,
    target_sample_rate: u32,
) -> anyhow::Result<MusicStream> {
    validate_streamed_music(path, target_channels, target_sample_rate)?;
    let path = path.to_path_buf();
    let state = Arc::new(StreamState::new());
    let worker_state = Arc::clone(&state);
    let worker = thread::Builder::new()
        .name("game-audio-stream".to_owned())
        .spawn(move || {
            streamed_music_worker(path, target_channels, target_sample_rate, worker_state)
        })
        .map_err(|error| anyhow::anyhow!("could not start streamed music worker: {error}"))?;
    Ok(MusicStream {
        state,
        worker: Some(worker),
    })
}

fn streamed_music_worker(
    path: std::path::PathBuf,
    target_channels: u16,
    target_sample_rate: u32,
    state: Arc<StreamState>,
) {
    let mut source = match StreamedPcm16Wav::open(&path, target_channels, target_sample_rate) {
        Ok(source) => source,
        Err(error) => {
            log::warn!("streamed music worker stopped: {error}");
            return;
        }
    };
    let mut bytes = vec![0_u8; STREAM_READ_BYTES];
    let mut generation = state.generation.load(Ordering::Acquire);

    while state.running.load(Ordering::Acquire) {
        let requested_generation = state.generation.load(Ordering::Acquire);
        if requested_generation != generation {
            if let Err(error) = source.rewind() {
                log::warn!(
                    "could not rewind streamed music '{}': {error}",
                    path.display()
                );
                return;
            }
            generation = requested_generation;
        }
        if state.samples.is_full() {
            thread::sleep(STREAM_WORKER_IDLE);
            continue;
        }
        let read = match source.read_chunk(&mut bytes) {
            Ok(0) => {
                if let Err(error) = source.rewind() {
                    log::warn!(
                        "could not loop streamed music '{}': {error}",
                        path.display()
                    );
                    return;
                }
                continue;
            }
            Ok(read) => read - read % 2,
            Err(error) => {
                log::warn!(
                    "could not read streamed music '{}': {error}",
                    path.display()
                );
                return;
            }
        };
        for sample in bytes[..read].chunks_exact(2) {
            while state
                .samples
                .push(StreamSample {
                    generation,
                    value: i16::from_le_bytes([sample[0], sample[1]]) as f32 / 32768.0,
                })
                .is_err()
            {
                if !state.running.load(Ordering::Acquire) {
                    return;
                }
                if state.generation.load(Ordering::Acquire) != generation {
                    break;
                }
                thread::sleep(STREAM_WORKER_IDLE);
            }
            if state.generation.load(Ordering::Acquire) != generation {
                break;
            }
        }
    }
}
