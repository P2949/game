use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=shaders");

    for entry in walkdir::WalkDir::new("shaders") {
        let entry = entry.expect("walk shader directory");
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else {
            continue;
        };

        if !matches!(ext, "vert" | "frag" | "comp") {
            continue;
        }

        println!("cargo:rerun-if-changed={}", path.display());
        compile_shader(path);
    }
}

fn compile_shader(path: &Path) {
    let output = path.with_extension(format!(
        "{}.spv",
        path.extension().unwrap().to_string_lossy()
    ));

    let status = Command::new("glslc")
        .arg(path)
        .arg("-o")
        .arg(&output)
        .status()
        .expect("failed to run glslc; install shaderc or adjust build.rs");

    if !status.success() {
        panic!("glslc failed for {}", path.display());
    }
}
