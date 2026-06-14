use std::path::{Path, PathBuf};
use std::process::Command;

const SHADER_ROOT: &str = "shaders";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={SHADER_ROOT}");
    // Allow overriding the compiler, and rebuild shaders if that override changes.
    println!("cargo:rerun-if-env-changed=GLSLC");
    // Optional SPIR-V validator override.
    println!("cargo:rerun-if-env-changed=SPIRV_VAL");

    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let glslc = std::env::var("GLSLC").unwrap_or_else(|_| "glslc".to_owned());
    let spirv_val = resolve_spirv_val();
    let shader_flag = match std::env::var("PROFILE").as_deref() {
        Ok("release") => "-O",
        _ => "-g",
    };

    for entry in walkdir::WalkDir::new(SHADER_ROOT) {
        let entry = entry.unwrap_or_else(|err| {
            panic!("failed to walk shader directory '{SHADER_ROOT}': {err}");
        });
        let path = entry.path();

        if path.is_dir() {
            println!("cargo:rerun-if-changed={}", path.display());
            continue;
        }

        if !path.is_file() {
            continue;
        }

        let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else {
            continue;
        };

        // glslc infers the shader stage from these extensions. Cover the full set
        // of graphics/compute/mesh/ray-tracing stages so adding, say, a geometry
        // or ray-gen shader later just works without revisiting this filter.
        if !matches!(
            ext,
            "vert"
                | "frag"
                | "comp"
                | "geom"
                | "tesc"
                | "tese"
                | "mesh"
                | "task"
                | "rgen"
                | "rint"
                | "rahit"
                | "rchit"
                | "rmiss"
                | "rcall"
        ) {
            continue;
        }

        println!("cargo:rerun-if-changed={}", path.display());
        compile_shader(&glslc, shader_flag, path, &out_dir, spirv_val.as_deref());
    }
}

fn resolve_spirv_val() -> Option<String> {
    match std::env::var("SPIRV_VAL") {
        Ok(value) if spirv_val_disabled(&value) => {
            println!("cargo:warning=SPIRV_VAL disabled; skipping optional SPIR-V validation");
            None
        }
        Ok(value) => Some(value.trim().to_owned()),
        Err(_) if command_exists("spirv-val") => Some("spirv-val".to_owned()),
        Err(_) => {
            println!("cargo:warning=spirv-val not found; skipping optional SPIR-V validation");
            None
        }
    }
}

fn spirv_val_disabled(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "" | "0" | "false" | "no" | "off" | "none" | "disabled"
    )
}

fn command_exists(command: &str) -> bool {
    Command::new(command).arg("--version").output().is_ok()
}

fn compile_shader(
    glslc: &str,
    shader_flag: &str,
    path: &Path,
    out_dir: &Path,
    spirv_val: Option<&str>,
) {
    // Mirror the shader's path *relative to the shader root* into OUT_DIR, so two
    // shaders with the same file name in different subdirectories (e.g.
    // `ui/sprite.vert` and `world/sprite.vert`) compile to distinct outputs
    // instead of silently clobbering each other. Top-level shaders keep their
    // existing `<name>.<stage>.spv` output name, so `include_bytes!` paths in the
    // renderer stay valid.
    let relative = path
        .strip_prefix(SHADER_ROOT)
        .expect("shader path is under the shader root");
    let mut output = out_dir.join(relative);
    let spv_name = format!(
        "{}.spv",
        output
            .file_name()
            .expect("shader path has a file name")
            .to_string_lossy()
    );
    output.set_file_name(spv_name);

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent).unwrap_or_else(|err| {
            panic!(
                "failed to create shader output directory {}: {err}",
                parent.display()
            )
        });
    }

    let result = Command::new(glslc)
        .arg(shader_flag)
        .arg(path)
        .arg("-o")
        .arg(&output)
        .output()
        .unwrap_or_else(|err| {
            panic!(
                "failed to run shader compiler '{glslc}' for {}: {err}\n\
                 install glslc (shaderc / the Vulkan SDK) or set GLSLC=/path/to/glslc",
                path.display()
            )
        });

    if !result.status.success() {
        panic!(
            "shader compilation failed\n  \
             compiler: {glslc}\n  \
             source:   {}\n  \
             output:   {}\n  \
             status:   {}\n  \
             stderr:\n{}\n  \
             stdout:\n{}",
            path.display(),
            output.display(),
            result.status,
            String::from_utf8_lossy(&result.stderr),
            String::from_utf8_lossy(&result.stdout),
        );
    }

    if let Some(spirv_val) = spirv_val {
        validate_spirv(spirv_val, &output);
    }
}

fn validate_spirv(spirv_val: &str, output: &Path) {
    let result = Command::new(spirv_val)
        .arg(output)
        .output()
        .unwrap_or_else(|err| {
            panic!(
                "failed to run SPIR-V validator '{spirv_val}' for {}: {err}",
                output.display()
            )
        });

    if !result.status.success() {
        panic!(
            "SPIR-V validation failed\n  \
             validator: {spirv_val}\n  \
             input:     {}\n  \
             status:    {}\n  \
             stderr:\n{}\n  \
             stdout:\n{}",
            output.display(),
            result.status,
            String::from_utf8_lossy(&result.stderr),
            String::from_utf8_lossy(&result.stdout),
        );
    }
}
