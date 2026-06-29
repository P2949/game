use std::fs;
use std::path::Path;

const PLACEHOLDER_PNG: &str = include_str!("assets/template-placeholder.png.base64");

fn main() {
    println!("cargo:rerun-if-changed=assets/template-placeholder.png.base64");

    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    let textures = assets.join("textures");
    let sounds = assets.join("sounds");
    let music = assets.join("music");
    fs::create_dir_all(&textures).expect("create texture directory");
    fs::create_dir_all(&sounds).expect("create sound directory");
    fs::create_dir_all(&music).expect("create music directory");

    let image = decode_base64(PLACEHOLDER_PNG.trim());
    for name in [
        "player",
        "slime",
        "boss",
        "bomber",
        "coin",
        "heart",
        "bolt",
        "spawner",
        "door",
        "danger",
        "checkpoint",
        "floor",
        "wall",
    ] {
        write_if_missing(&textures.join(format!("{name}.png")), &image);
    }

    let blip = silent_wav(0.08);
    write_if_missing(&sounds.join("hit.wav"), &blip);
    let theme = silent_wav(0.25);
    write_if_missing(&music.join("theme.wav"), &theme);
}

fn write_if_missing(path: &Path, bytes: &[u8]) {
    if !path.exists() {
        fs::write(path, bytes).expect("write generated demo asset");
    }
}

fn silent_wav(seconds: f32) -> Vec<u8> {
    let sample_rate = 48_000u32;
    let samples = (sample_rate as f32 * seconds.max(0.01)) as u32;
    let data_bytes = samples * 2;
    let mut out = Vec::with_capacity(44 + data_bytes as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_bytes).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_bytes.to_le_bytes());
    out.resize(44 + data_bytes as usize, 0);
    out
}

fn decode_base64(input: &str) -> Vec<u8> {
    fn value(byte: u8) -> u8 {
        match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => panic!("template placeholder is not valid base64"),
        }
    }

    let mut output = Vec::with_capacity(input.len() * 3 / 4);
    let bytes = input
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect::<Vec<_>>();
    assert!(bytes.len().is_multiple_of(4), "invalid base64 length");
    for chunk in bytes.chunks(4) {
        let first = value(chunk[0]);
        let second = value(chunk[1]);
        let third = chunk
            .get(2)
            .copied()
            .filter(|byte| *byte != b'=')
            .map(value);
        let fourth = chunk
            .get(3)
            .copied()
            .filter(|byte| *byte != b'=')
            .map(value);
        output.push((first << 2) | (second >> 4));
        if let Some(third) = third {
            output.push(((second & 0x0f) << 4) | (third >> 2));
            if let Some(fourth) = fourth {
                output.push(((third & 0x03) << 6) | fourth);
            }
        }
    }
    output
}
