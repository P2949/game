use std::f32::consts::TAU;
use std::fs;
use std::path::Path;

const TEXTURE_SIZE: u32 = 16;
const SAMPLE_RATE: u32 = 48_000;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    let textures = assets.join("textures");
    let sounds = assets.join("sounds");
    fs::create_dir_all(&textures).expect("create template texture directory");
    fs::create_dir_all(&sounds).expect("create template sound directory");

    for kind in [
        StarterTexture::Player,
        StarterTexture::Slime,
        StarterTexture::Coin,
        StarterTexture::Floor,
        StarterTexture::Wall,
        StarterTexture::Door,
        StarterTexture::Bolt,
    ] {
        let path = textures.join(format!("{}.png", kind.name()));
        if !path.exists() {
            fs::write(path, make_png(kind)).expect("write starter texture");
        }
    }

    for sound in [
        StarterSound::new("hit", 160.0, 0.08),
        StarterSound::new("coin", 880.0, 0.14),
        StarterSound::new("shoot", 440.0, 0.10),
    ] {
        let path = sounds.join(format!("{}.wav", sound.name));
        if !path.exists() {
            fs::write(path, make_wav(sound)).expect("write starter sound");
        }
    }
}

#[derive(Clone, Copy)]
enum StarterTexture {
    Player,
    Slime,
    Coin,
    Floor,
    Wall,
    Door,
    Bolt,
}

impl StarterTexture {
    fn name(self) -> &'static str {
        match self {
            Self::Player => "player",
            Self::Slime => "slime",
            Self::Coin => "coin",
            Self::Floor => "floor",
            Self::Wall => "wall",
            Self::Door => "door",
            Self::Bolt => "bolt",
        }
    }
}

#[derive(Clone, Copy)]
struct StarterSound {
    name: &'static str,
    frequency: f32,
    seconds: f32,
}

impl StarterSound {
    const fn new(name: &'static str, frequency: f32, seconds: f32) -> Self {
        Self {
            name,
            frequency,
            seconds,
        }
    }
}

fn make_png(kind: StarterTexture) -> Vec<u8> {
    let mut rgba = Vec::with_capacity((TEXTURE_SIZE * TEXTURE_SIZE * 4) as usize);
    for y in 0..TEXTURE_SIZE {
        for x in 0..TEXTURE_SIZE {
            rgba.extend_from_slice(&pixel(kind, x, y));
        }
    }
    encode_png_rgba(TEXTURE_SIZE, TEXTURE_SIZE, &rgba)
}

fn pixel(kind: StarterTexture, x: u32, y: u32) -> [u8; 4] {
    match kind {
        StarterTexture::Floor => {
            if ((x / 4) + (y / 4)).is_multiple_of(2) {
                [112, 124, 120, 255]
            } else {
                [136, 148, 142, 255]
            }
        }
        StarterTexture::Wall => {
            if x.is_multiple_of(4) || y.is_multiple_of(4) {
                [42, 49, 58, 255]
            } else {
                [74, 84, 96, 255]
            }
        }
        StarterTexture::Door => {
            if x == 2 || x == 13 || y == 1 || y == 14 {
                [56, 34, 23, 255]
            } else if (x, y) == (11, 8) || (x, y) == (12, 8) {
                [245, 198, 82, 255]
            } else {
                [132, 76, 38, 255]
            }
        }
        StarterTexture::Player => {
            let body = (5..=10).contains(&x) && (5..=12).contains(&y);
            let head = (6..=9).contains(&x) && (2..=5).contains(&y);
            let eye = (x, y) == (8, 4);
            if eye {
                [18, 25, 36, 255]
            } else if head {
                [242, 188, 132, 255]
            } else if body {
                [42, 114, 224, 255]
            } else {
                [0, 0, 0, 0]
            }
        }
        StarterTexture::Slime => {
            let dx = x as i32 - 8;
            let dy = y as i32 - 9;
            let in_blob = dx * dx * 3 + dy * dy * 4 < 150;
            let eye = ((x, y) == (6, 8)) || ((x, y) == (10, 8));
            if eye {
                [20, 35, 24, 255]
            } else if in_blob {
                [82, 196, 98, 255]
            } else {
                [0, 0, 0, 0]
            }
        }
        StarterTexture::Coin => {
            let dx = x as i32 - 8;
            let dy = y as i32 - 8;
            let d = dx * dx + dy * dy;
            if d < 30 {
                [255, 221, 74, 255]
            } else if d < 45 {
                [218, 150, 38, 255]
            } else {
                [0, 0, 0, 0]
            }
        }
        StarterTexture::Bolt => {
            if (x + 3 >= y && x <= y + 5 && (4..=12).contains(&y))
                || ((6..=14).contains(&x) && (6..=9).contains(&y))
            {
                [88, 226, 255, 255]
            } else {
                [0, 0, 0, 0]
            }
        }
    }
}

fn encode_png_rgba(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    let mut raw = Vec::with_capacity((height * (1 + width * 4)) as usize);
    for row in 0..height as usize {
        raw.push(0);
        let start = row * width as usize * 4;
        let end = start + width as usize * 4;
        raw.extend_from_slice(&rgba[start..end]);
    }

    let mut png = Vec::new();
    png.extend_from_slice(b"\x89PNG\r\n\x1a\n");
    push_chunk(&mut png, b"IHDR", &png_ihdr(width, height));
    push_chunk(&mut png, b"IDAT", &zlib_uncompressed(&raw));
    push_chunk(&mut png, b"IEND", &[]);
    png
}

fn png_ihdr(width: u32, height: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity(13);
    data.extend_from_slice(&width.to_be_bytes());
    data.extend_from_slice(&height.to_be_bytes());
    data.extend_from_slice(&[8, 6, 0, 0, 0]);
    data
}

fn push_chunk(png: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    png.extend_from_slice(&(data.len() as u32).to_be_bytes());
    png.extend_from_slice(kind);
    png.extend_from_slice(data);
    let mut crc_data = Vec::with_capacity(kind.len() + data.len());
    crc_data.extend_from_slice(kind);
    crc_data.extend_from_slice(data);
    png.extend_from_slice(&crc32(&crc_data).to_be_bytes());
}

fn zlib_uncompressed(raw: &[u8]) -> Vec<u8> {
    let mut data = vec![0x78, 0x01];
    let mut offset = 0;
    while offset < raw.len() {
        let remaining = raw.len() - offset;
        let len = remaining.min(u16::MAX as usize);
        let final_block = u8::from(offset + len == raw.len());
        data.push(final_block);
        data.extend_from_slice(&(len as u16).to_le_bytes());
        data.extend_from_slice(&(!(len as u16)).to_le_bytes());
        data.extend_from_slice(&raw[offset..offset + len]);
        offset += len;
    }
    data.extend_from_slice(&adler32(raw).to_be_bytes());
    data
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

fn adler32(bytes: &[u8]) -> u32 {
    const MOD: u32 = 65_521;
    let mut a = 1u32;
    let mut b = 0u32;
    for byte in bytes {
        a = (a + u32::from(*byte)) % MOD;
        b = (b + a) % MOD;
    }
    (b << 16) | a
}

fn make_wav(sound: StarterSound) -> Vec<u8> {
    let samples = (SAMPLE_RATE as f32 * sound.seconds) as usize;
    let mut pcm = Vec::with_capacity(samples * 2);
    for index in 0..samples {
        let t = index as f32 / SAMPLE_RATE as f32;
        let fade = 1.0 - (index as f32 / samples as f32);
        let wobble = 1.0 + 0.08 * (TAU * 12.0 * t).sin();
        let sample = (TAU * sound.frequency * wobble * t).sin() * fade * 0.35;
        let sample = (sample * i16::MAX as f32) as i16;
        pcm.extend_from_slice(&sample.to_le_bytes());
    }

    let mut wav = Vec::with_capacity(44 + pcm.len());
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + pcm.len() as u32).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    wav.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(pcm.len() as u32).to_le_bytes());
    wav.extend_from_slice(&pcm);
    wav
}
