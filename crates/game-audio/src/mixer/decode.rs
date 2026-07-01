use std::fs;
use std::path::Path;

#[cfg(feature = "ogg")]
use std::io::Cursor;
#[cfg(feature = "mp3")]
use std::io::Write;
#[cfg(feature = "mp3")]
use std::process::{Command, Stdio};

use super::{Mixer, Sound};

/// Decodes and normalizes a supported file-backed audio asset without opening
/// an SDL stream. Tooling uses this to validate packaged sounds before shipping.
pub fn validate_file_sound(path: &Path) -> anyhow::Result<()> {
    load_file_sound(path, 2, 48_000).map(|_| ())
}

pub(super) fn load_file_sound(
    path: &Path,
    target_channels: u16,
    target_sample_rate: u32,
) -> anyhow::Result<Sound> {
    let bytes = fs::read(path)
        .map_err(anyhow::Error::from)
        .map_err(|err| anyhow::anyhow!("failed to read sound '{}': {err}", path.display()))?;
    decode_file_sound(path, &bytes, target_channels, target_sample_rate)
}

pub(super) fn decode_file_sound(
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
pub(super) enum SoundFormat {
    Wav,
    Ogg,
    Mp3,
    Unknown,
}

pub(super) fn detect_sound_format(path: &Path, bytes: &[u8]) -> SoundFormat {
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

pub(super) fn normalize_sound(
    sound: Sound,
    target_channels: u16,
    target_sample_rate: u32,
) -> anyhow::Result<Sound> {
    let sound = convert_channels(sound, target_channels)?;
    resample_linear(sound, target_sample_rate)
}

pub(super) fn convert_channels(sound: Sound, target_channels: u16) -> anyhow::Result<Sound> {
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

pub(super) fn resample_linear(sound: Sound, target_sample_rate: u32) -> anyhow::Result<Sound> {
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

pub(super) fn decode_wav_sound(bytes: &[u8]) -> anyhow::Result<Sound> {
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
pub(super) struct WavFormat {
    pub(super) audio_format: u16,
    pub(super) channels: u16,
    pub(super) sample_rate: u32,
    pub(super) block_align: u16,
    pub(super) bits_per_sample: u16,
}

pub(super) fn parse_wav_format(chunk: &[u8]) -> anyhow::Result<WavFormat> {
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
