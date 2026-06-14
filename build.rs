use std::path::{Path, PathBuf};
use std::process::Command;

const SHADER_ROOT: &str = "shaders";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={SHADER_ROOT}");
    // Allow overriding the compiler, and rebuild shaders if that override changes.
    println!("cargo:rerun-if-env-changed=GLSLC");

    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let glslc = std::env::var("GLSLC").unwrap_or_else(|_| "glslc".to_owned());

    for entry in walkdir::WalkDir::new(SHADER_ROOT) {
        let entry = entry.expect("walk shader directory");
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
        compile_shader(&glslc, path, &out_dir);
    }
}

fn compile_shader(glslc: &str, path: &Path, out_dir: &Path) {
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
             stderr:\n{}",
            path.display(),
            output.display(),
            result.status,
            String::from_utf8_lossy(&result.stderr),
        );
    }
}
