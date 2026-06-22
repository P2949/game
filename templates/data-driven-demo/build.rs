use std::fs;
use std::path::Path;

const PLACEHOLDER_PNG: &str = include_str!("assets/template-placeholder.png.base64");

fn main() {
    println!("cargo:rerun-if-changed=assets/template-placeholder.png.base64");
    let textures = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/textures");
    fs::create_dir_all(&textures).expect("create template texture directory");
    let image = decode_base64(PLACEHOLDER_PNG.trim());
    for name in ["player", "slime", "coin", "floor", "wall"] {
        let path = textures.join(format!("{name}.png"));
        if !path.exists() {
            fs::write(path, &image).expect("write template placeholder texture");
        }
    }
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
        let third = chunk.get(2).copied().filter(|byte| *byte != b'=').map(value);
        let fourth = chunk.get(3).copied().filter(|byte| *byte != b'=').map(value);
        output.push((first << 2) | (second >> 4));
        if let Some(third) = third {
            output.push((second << 4) | (third >> 2));
            if let Some(fourth) = fourth {
                output.push((third << 6) | fourth);
            }
        }
    }
    output
}
