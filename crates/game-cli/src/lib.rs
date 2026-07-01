use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};
use fontdue::{Font, FontSettings};
use image::ImageReader;
use walkdir::WalkDir;

const RELEASE_GAME_STARTER_DEPENDENCY: &str =
    r#"{ git = "https://github.com/P2949/game", tag = "v0.2.0", package = "game-starter" }"#;

struct TemplateFile {
    path: &'static str,
    contents: &'static str,
}

const SIMPLE_TEMPLATE: &[TemplateFile] = &[
    TemplateFile {
        path: "Cargo.toml",
        contents: include_str!("../../../templates/simple-demo/Cargo.toml"),
    },
    TemplateFile {
        path: "README.md",
        contents: include_str!("../../../templates/simple-demo/README.md"),
    },
    TemplateFile {
        path: "build.rs",
        contents: include_str!("../../../templates/simple-demo/build.rs"),
    },
    TemplateFile {
        path: "src/main.rs",
        contents: include_str!("../../../templates/simple-demo/src/main.rs"),
    },
    TemplateFile {
        path: "assets/maps/level_1.txt",
        contents: include_str!("../../../templates/simple-demo/assets/maps/level_1.txt"),
    },
];

const DATA_DRIVEN_TEMPLATE: &[TemplateFile] = &[
    TemplateFile {
        path: "Cargo.toml",
        contents: include_str!("../../../templates/data-driven-demo/Cargo.toml"),
    },
    TemplateFile {
        path: "README.md",
        contents: include_str!("../../../templates/data-driven-demo/README.md"),
    },
    TemplateFile {
        path: "build.rs",
        contents: include_str!("../../../templates/data-driven-demo/build.rs"),
    },
    TemplateFile {
        path: "src/main.rs",
        contents: include_str!("../../../templates/data-driven-demo/src/main.rs"),
    },
    TemplateFile {
        path: "assets/game.ron",
        contents: include_str!("../../../templates/data-driven-demo/assets/game.ron"),
    },
    TemplateFile {
        path: "assets/maps/level_1.txt",
        contents: include_str!("../../../templates/data-driven-demo/assets/maps/level_1.txt"),
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DemoTemplate {
    Simple,
    DataDriven,
}

impl DemoTemplate {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "simple" => Ok(Self::Simple),
            "data-driven" => Ok(Self::DataDriven),
            other => bail!("unknown template '{other}'; expected simple or data-driven"),
        }
    }

    fn files(self) -> &'static [TemplateFile] {
        match self {
            Self::Simple => SIMPLE_TEMPLATE,
            Self::DataDriven => DATA_DRIVEN_TEMPLATE,
        }
    }

    fn is_data_driven(self) -> bool {
        matches!(self, Self::DataDriven)
    }
}

pub fn run(args: impl IntoIterator<Item = String>) -> Result<()> {
    let mut args = args.into_iter();
    match args.next().as_deref() {
        Some("new") => {
            let path = args.next().ok_or_else(|| {
                anyhow!("usage: game-dev new <path> [--template simple|data-driven]")
            })?;
            let template = parse_template_args(args)?;
            let destination = absolutize_from_current(Path::new(&path))?;
            new_project(&destination, template, RELEASE_GAME_STARTER_DEPENDENCY)
        }
        Some("doctor") => {
            let options = parse_doctor_options(args)?;
            doctor(options);
            Ok(())
        }
        Some("run") => {
            reject_extra(args, "run")?;
            run_project()
        }
        Some("check") => {
            let options = parse_check_options(args)?;
            check_project(&options)
        }
        Some("package") => package_project_command(args),
        Some("asset-check") => {
            reject_extra(args, "asset-check")?;
            validate_assets_dir(&env::current_dir()?.join("assets"), false)?;
            println!("assets look valid");
            Ok(())
        }
        Some("validate-data") => {
            let path = args.next().unwrap_or_else(|| "game.ron".to_string());
            reject_extra(args, "validate-data")?;
            let asset_root = configured_asset_root();
            game_kit::data::validate_beginner_game_file(normalize_validate_data_path(
                &path,
                &asset_root,
            ))?;
            println!("beginner data file is valid");
            Ok(())
        }
        _ => bail!(
            "usage:\n    game-dev new <path> [--template simple|data-driven]\n    game-dev doctor\n    game-dev check [--features feature-list]\n    game-dev run\n    game-dev package --release --out <directory> [--features feature-list] [--zip]\n    game-dev asset-check\n    game-dev validate-data [game.ron]"
        ),
    }
}

pub fn run_xtask(args: impl IntoIterator<Item = String>) -> Result<()> {
    let workspace = workspace_root()?;
    let mut args = args.into_iter();
    match args.next().as_deref() {
        Some("new-demo") => {
            let name = args.next().ok_or_else(|| {
                anyhow!(
                    "usage: cargo xtask new-demo <name-or-path> [--template simple|data-driven]"
                )
            })?;
            let template = parse_template_args(args)?;
            let destination = xtask_demo_destination(&workspace, &name)?;
            let game_path = game_path_from_destination(&workspace, &destination)?;
            let dependency = format!(r#"{{ path = "{game_path}/crates/game-starter" }}"#);
            new_project(&destination, template, &dependency)
        }
        Some("doctor") => {
            let options = parse_doctor_options(args)?;
            doctor(options);
            Ok(())
        }
        Some("release-check") => release_check_command(args, &workspace),
        Some("package-demo") => package_workspace_demo_command(args, &workspace),
        _ => bail!(
            "usage:\n    cargo xtask new-demo <name-or-path> [--template simple|data-driven]\n    cargo xtask new-demo <name-or-path> --data-driven\n    cargo xtask release-check [--skip-smoke] [--skip-generated] [--features feature-list]\n    cargo xtask package-demo --release --out <directory> [--features feature-list]\n    cargo xtask doctor\n\nCreates an outside-workspace beginner demo, runs release-candidate checks, packages the bundled playable demo, or checks local graphics prerequisites."
        ),
    }
}

fn parse_template_args(args: impl IntoIterator<Item = String>) -> Result<DemoTemplate> {
    let mut template = DemoTemplate::Simple;
    let mut args = args.into_iter();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--data-driven" => template = DemoTemplate::DataDriven,
            "--template" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("--template needs simple or data-driven"))?;
                template = DemoTemplate::parse(&value)?;
            }
            extra => bail!(
                "unexpected template argument '{extra}'; expected --template simple|data-driven"
            ),
        }
    }
    Ok(template)
}

fn reject_extra(mut args: impl Iterator<Item = String>, command: &str) -> Result<()> {
    if let Some(extra) = args.next() {
        bail!("unexpected argument for {command}: '{extra}'");
    }
    Ok(())
}

struct CheckOptions {
    features: Vec<String>,
}

fn parse_check_options(mut args: impl Iterator<Item = String>) -> Result<CheckOptions> {
    let mut features = Vec::new();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--features" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("--features needs a comma-separated feature list"))?;
                features.push(value);
            }
            extra => bail!("unexpected check argument '{extra}'; expected --features <list>"),
        }
    }
    Ok(CheckOptions { features })
}

fn configured_asset_root() -> PathBuf {
    env::var_os("GAME_ASSET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("assets"))
}

fn normalize_validate_data_path(path: impl AsRef<Path>, asset_root: &Path) -> PathBuf {
    let path = path.as_ref();
    if path.is_absolute() || asset_root.is_absolute() {
        return path.to_path_buf();
    }

    path.strip_prefix(asset_root)
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| path.to_path_buf())
}

#[derive(Clone, Copy)]
struct DoctorOptions {
    explain: bool,
}

fn parse_doctor_options(args: impl IntoIterator<Item = String>) -> Result<DoctorOptions> {
    let mut explain = false;
    for argument in args {
        match argument.as_str() {
            "--explain" => explain = true,
            extra => bail!("unexpected doctor argument '{extra}'; expected --explain"),
        }
    }
    Ok(DoctorOptions { explain })
}

fn package_project_command(args: impl Iterator<Item = String>) -> Result<()> {
    let PackageOptions {
        release,
        output,
        zip,
        features,
    } = parse_package_options(args, "package")?;
    if !release {
        bail!("game-dev package currently requires --release");
    }
    let output = output.ok_or_else(|| anyhow!("game-dev package requires --out <directory>"))?;
    package_current_project(&output, zip, &features)
}

fn package_workspace_demo_command(
    args: impl Iterator<Item = String>,
    workspace: &Path,
) -> Result<()> {
    let PackageOptions {
        release,
        output,
        zip,
        features,
    } = parse_package_options(args, "package-demo")?;
    if zip {
        bail!(
            "cargo xtask package-demo does not support --zip; use game-dev package for project zips"
        );
    }
    if !release {
        bail!("package-demo currently requires --release");
    }
    let output = output.ok_or_else(|| anyhow!("package-demo requires --out <directory>"))?;
    package_workspace_demo(workspace, &output, &features)
}

fn release_check_command(args: impl Iterator<Item = String>, workspace: &Path) -> Result<()> {
    let options = parse_release_check_options(args)?;
    run_release_check(workspace, &options)
}

struct PackageOptions {
    release: bool,
    output: Option<PathBuf>,
    zip: bool,
    features: Vec<String>,
}

struct ReleaseCheckOptions {
    skip_smoke: bool,
    skip_generated: bool,
    features: Vec<String>,
}

fn parse_release_check_options(
    mut args: impl Iterator<Item = String>,
) -> Result<ReleaseCheckOptions> {
    let mut skip_smoke = false;
    let mut skip_generated = false;
    let mut features = Vec::new();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--skip-smoke" => skip_smoke = true,
            "--skip-generated" => skip_generated = true,
            "--features" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("--features needs a comma-separated feature list"))?;
                features.push(value);
            }
            other => bail!(
                "unknown release-check argument '{other}'; expected --skip-smoke, --skip-generated, or --features <list>"
            ),
        }
    }
    Ok(ReleaseCheckOptions {
        skip_smoke,
        skip_generated,
        features,
    })
}

fn run_release_check(workspace: &Path, options: &ReleaseCheckOptions) -> Result<()> {
    let workspace_features = workspace_feature_names(&options.features);

    let mut fmt = Command::new("cargo");
    fmt.args(["fmt", "--all", "--", "--check"])
        .current_dir(workspace);
    run_command(&mut fmt, "cargo fmt --all -- --check")?;

    let mut test = Command::new("cargo");
    test.args(["test", "--workspace", "--locked"])
        .current_dir(workspace);
    add_features(&mut test, &workspace_features);
    run_command(&mut test, "cargo test --workspace --locked")?;

    let mut headless = Command::new("cargo");
    headless
        .args([
            "test",
            "-p",
            "game-runtime",
            "--test",
            "headless_runner",
            "--no-default-features",
            "--locked",
        ])
        .current_dir(workspace);
    run_command(
        &mut headless,
        "cargo test -p game-runtime --test headless_runner --no-default-features --locked",
    )?;

    let mut clippy = Command::new("cargo");
    clippy
        .args(["clippy", "--workspace", "--all-targets", "--locked"])
        .current_dir(workspace);
    add_features(&mut clippy, &workspace_features);
    clippy.args(["--", "-D", "warnings"]);
    run_command(
        &mut clippy,
        "cargo clippy --workspace --all-targets --locked -- -D warnings",
    )?;

    let mut build = Command::new("cargo");
    build
        .args(["build", "-p", "game", "--release", "--locked"])
        .current_dir(workspace);
    add_features(&mut build, &options.features);
    run_command(&mut build, "cargo build -p game --release --locked")?;

    let mut doctor = cargo_run_game_cli(workspace, &options.features);
    doctor.args(["doctor", "--explain"]);
    run_command(&mut doctor, "cargo run -p game-cli -- doctor --explain")?;

    let mut asset_check = cargo_run_game_cli(workspace, &options.features);
    asset_check.arg("asset-check");
    run_command(&mut asset_check, "cargo run -p game-cli -- asset-check")?;

    let mut validate_data = cargo_run_game_cli(workspace, &options.features);
    validate_data.args(["validate-data", "assets/game.ron"]);
    run_command(
        &mut validate_data,
        "cargo run -p game-cli -- validate-data assets/game.ron",
    )?;

    if options.skip_generated {
        println!("==> skipping generated-project checks");
    } else {
        run_generated_release_checks(workspace, options)?;
    }

    if options.skip_smoke {
        println!("==> skipping graphical smoke checks");
    } else {
        run_smoke_release_checks(workspace, &options.features)?;
    }

    println!("release check passed");
    Ok(())
}

fn cargo_run_game_cli(workspace: &Path, features: &[String]) -> Command {
    let mut command = Command::new("cargo");
    command
        .args(["run", "-p", "game-cli"])
        .current_dir(workspace);
    add_features(&mut command, features);
    command.arg("--");
    command
}

fn workspace_feature_names(features: &[String]) -> Vec<String> {
    features
        .iter()
        .map(|feature| {
            if feature.contains('/') {
                feature.clone()
            } else {
                format!("game/{feature}")
            }
        })
        .collect()
}

fn add_features(command: &mut Command, features: &[String]) {
    for feature in features {
        command.arg("--features").arg(feature);
    }
}

fn run_command(command: &mut Command, label: &str) -> Result<()> {
    println!("==> {label}");
    let status = command
        .status()
        .with_context(|| format!("could not run `{label}`"))?;
    if !status.success() {
        bail!("`{label}` failed with {status}");
    }
    Ok(())
}

fn run_generated_release_checks(workspace: &Path, options: &ReleaseCheckOptions) -> Result<()> {
    let root = env::temp_dir().join("game-release-check/generated");
    if root.exists() {
        fs::remove_dir_all(&root)
            .with_context(|| format!("failed to remove '{}'", root.display()))?;
    }
    fs::create_dir_all(&root).with_context(|| format!("failed to create '{}'", root.display()))?;

    let dependency = format!(
        r#"{{ path = "{}" }}"#,
        workspace.join("crates/game-starter").display()
    );
    let simple = root.join("simple");
    let data = root.join("data");
    new_project(&simple, DemoTemplate::Simple, &dependency)?;
    new_project(&data, DemoTemplate::DataDriven, &dependency)?;

    run_generated_project_release_checks(workspace, &simple, "simple", &options.features)?;
    run_generated_project_release_checks(workspace, &data, "data-driven", &options.features)
}

fn run_generated_project_release_checks(
    workspace: &Path,
    project: &Path,
    label: &str,
    features: &[String],
) -> Result<()> {
    let mut check = Command::new("cargo");
    check.arg("check").current_dir(project);
    add_features(&mut check, features);
    run_command(
        &mut check,
        &format!("cargo check ({label} generated project)"),
    )?;

    let mut game_dev_check = Command::new("cargo");
    game_dev_check
        .args(["run", "--manifest-path"])
        .arg(workspace.join("Cargo.toml"))
        .args(["-p", "game-cli"])
        .current_dir(project);
    add_features(&mut game_dev_check, features);
    game_dev_check.args(["--", "check"]);
    add_features(&mut game_dev_check, features);
    run_command(
        &mut game_dev_check,
        &format!("game-dev check ({label} generated project)"),
    )?;

    package_project_at(
        project,
        &PathBuf::from(format!("dist/{label}-release-check")),
        true,
        features,
    )
    .with_context(|| format!("failed to package {label} generated project"))
}

fn run_smoke_release_checks(workspace: &Path, features: &[String]) -> Result<()> {
    let mut default_game = Command::new("cargo");
    default_game
        .args(["run", "-p", "game", "--locked"])
        .env("GAME_SMOKE_FRAMES", "120")
        .current_dir(workspace);
    add_features(&mut default_game, features);
    run_command(
        &mut default_game,
        "GAME_SMOKE_FRAMES=120 cargo run -p game --locked",
    )?;

    let mut simple_game = Command::new("cargo");
    simple_game
        .args(["run", "-p", "game", "--locked"])
        .env("GAME_DEMO", "simple")
        .env("GAME_SMOKE_FRAMES", "120")
        .current_dir(workspace);
    add_features(&mut simple_game, features);
    run_command(
        &mut simple_game,
        "GAME_DEMO=simple GAME_SMOKE_FRAMES=120 cargo run -p game --locked",
    )?;

    let mut testbed_game = Command::new("cargo");
    testbed_game
        .args(["run", "-p", "game", "--locked"])
        .env("GAME_DEMO", "testbed")
        .env("GAME_SMOKE_FRAMES", "120")
        .current_dir(workspace);
    add_features(&mut testbed_game, features);
    run_command(
        &mut testbed_game,
        "GAME_DEMO=testbed GAME_SMOKE_FRAMES=120 cargo run -p game --locked",
    )?;

    let mut release_game = Command::new("cargo");
    release_game
        .args(["run", "-p", "game", "--release", "--locked"])
        .env("GAME_ASSET_DIR", "assets")
        .env("GAME_SMOKE_FRAMES", "120")
        .current_dir(workspace);
    add_features(&mut release_game, features);
    run_command(
        &mut release_game,
        "GAME_ASSET_DIR=assets GAME_SMOKE_FRAMES=120 cargo run -p game --release --locked",
    )?;

    let mut tiled = Command::new("cargo");
    tiled
        .args(["run", "-p", "tiled-demo", "--locked"])
        .env("GAME_ASSET_DIR", "examples/tiled-demo/assets")
        .env("GAME_SMOKE_FRAMES", "60")
        .current_dir(workspace);
    add_features(&mut tiled, features);
    run_command(
        &mut tiled,
        "GAME_ASSET_DIR=examples/tiled-demo/assets GAME_SMOKE_FRAMES=60 cargo run -p tiled-demo --locked",
    )?;

    let mut data_driven_tiled = Command::new("cargo");
    data_driven_tiled
        .args(["run", "-p", "data-driven-tiled-demo", "--locked"])
        .env("GAME_ASSET_DIR", "examples/data-driven-tiled-demo/assets")
        .env("GAME_SMOKE_FRAMES", "60")
        .current_dir(workspace);
    add_features(&mut data_driven_tiled, features);
    run_command(
        &mut data_driven_tiled,
        "GAME_ASSET_DIR=examples/data-driven-tiled-demo/assets GAME_SMOKE_FRAMES=60 cargo run -p data-driven-tiled-demo --locked",
    )
}

fn parse_package_options(
    mut args: impl Iterator<Item = String>,
    command: &str,
) -> Result<PackageOptions> {
    let mut release = false;
    let mut output = None;
    let mut zip = false;
    let mut features = Vec::new();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--release" => release = true,
            "--zip" => zip = true,
            "--features" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("--features needs a comma-separated feature list"))?;
                features.push(value);
            }
            "--out" => {
                let path = args
                    .next()
                    .ok_or_else(|| anyhow!("--out needs a destination directory"))?;
                output = Some(PathBuf::from(path));
            }
            other => bail!("unknown {command} argument '{other}'"),
        }
    }
    Ok(PackageOptions {
        release,
        output,
        zip,
        features,
    })
}

fn new_project(destination: &Path, template: DemoTemplate, dependency: &str) -> Result<()> {
    if destination.exists() {
        bail!("destination '{}' already exists", destination.display());
    }

    let crate_name = crate_name_from_destination(destination)?;
    let title = title_from_crate_name(&crate_name);
    let mut values = HashMap::new();
    values.insert("crate_name", crate_name.as_str());
    values.insert("game_starter_dependency", dependency);
    values.insert("title", title.as_str());

    write_embedded_template(template.files(), destination, &values)?;

    println!("created demo at {}", destination.display());
    if template.is_data_driven() {
        println!("setup lives in assets/game.ron; src/main.rs is ready for optional custom rules");
    } else {
        println!("setup lives in src/main.rs with beginner Rust builder chains");
    }
    println!("next steps:");
    println!("    cd {}", destination.display());
    println!("    game-dev doctor");
    println!("    game-dev check");
    println!("    game-dev run");
    Ok(())
}

fn write_embedded_template(
    files: &[TemplateFile],
    destination: &Path,
    values: &HashMap<&str, &str>,
) -> Result<()> {
    fs::create_dir_all(destination)
        .with_context(|| format!("failed to create '{}'", destination.display()))?;
    for file in files {
        let mut contents = file.contents.to_string();
        for (key, value) in values {
            contents = contents.replace(&format!("{{{{{key}}}}}"), value);
        }
        let path = destination.join(file.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create '{}'", parent.display()))?;
        }
        fs::write(&path, contents)
            .with_context(|| format!("failed to write '{}'", path.display()))?;
    }
    Ok(())
}

fn run_project() -> Result<()> {
    let status = Command::new("cargo")
        .arg("run")
        .status()
        .context("could not run `cargo run`")?;
    if !status.success() {
        bail!("cargo run failed.\n\n{}", beginner_failure_advice());
    }
    Ok(())
}

fn beginner_failure_advice() -> &'static str {
    "If this looks like a setup issue:\n    game-dev doctor --explain\n\nIf this looks like an asset/data issue:\n    game-dev asset-check\n    game-dev validate-data assets/game.ron\n\nSee:\n    docs/tutorials/common-errors.md"
}

fn check_project(options: &CheckOptions) -> Result<()> {
    let project = env::current_dir().context("failed to resolve current project directory")?;
    check_project_at(&project, options)
}

fn check_project_at(project: &Path, options: &CheckOptions) -> Result<()> {
    let asset_root = configured_asset_root();
    let assets = absolutize_from(project, &asset_root);

    println!("checking project setup...");
    doctor(DoctorOptions { explain: false });

    println!("\nchecking assets...");
    validate_assets_dir(&assets, false)?;

    let data_file = assets.join("game.ron");
    if data_file.is_file() {
        println!("checking data file...");
        game_kit::data::validate_beginner_game_file(&data_file)?;
    }

    println!("running cargo check...");
    let mut command = Command::new("cargo");
    command.arg("check").current_dir(project);
    for feature in &options.features {
        command.arg("--features").arg(feature);
    }
    let status = command
        .status()
        .context("could not run `cargo check`; is Rust installed and available on PATH?")?;
    if !status.success() {
        bail!("cargo check failed.\n\n{}", beginner_failure_advice());
    }

    println!("project check passed");
    Ok(())
}

fn package_current_project(requested_output: &Path, zip: bool, features: &[String]) -> Result<()> {
    let project = env::current_dir().context("failed to resolve current project directory")?;
    package_project_at(&project, requested_output, zip, features)
}

fn package_project_at(
    project: &Path,
    requested_output: &Path,
    zip: bool,
    features: &[String],
) -> Result<()> {
    let output = absolutize_from(project, requested_output);
    ensure_empty_or_missing(&output)?;

    let package_info = package_info_from_manifest(&project.join("Cargo.toml"))?;
    let assets = absolutize_from(project, &package_info.asset_dir);
    if !assets.is_dir() {
        bail!("assets directory '{}' does not exist", assets.display());
    }

    let mut build = Command::new("cargo");
    build.args(["build", "--release"]).current_dir(project);
    for feature in features {
        build.arg("--features").arg(feature);
    }
    let status = build
        .status()
        .context("could not run release build for generated project")?;
    if !status.success() {
        bail!(
            "release build failed; no package was created.\n\n{}",
            beginner_failure_advice()
        );
    }

    validate_assets_dir(&assets, false)?;

    let executable_name = executable_name(&package_info.package_name);
    let target = env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| project.join("target"));
    let executable = target.join("release").join(&executable_name);
    if !executable.is_file() {
        bail!(
            "release build completed but '{}' was not produced",
            executable.display()
        );
    }

    fs::create_dir_all(&output)
        .with_context(|| format!("failed to create package output '{}'", output.display()))?;
    fs::copy(&executable, output.join(&executable_name)).with_context(|| {
        format!(
            "failed to copy packaged executable '{}' to '{}'",
            executable.display(),
            output.display()
        )
    })?;
    copy_runtime_libraries(&target.join("release"), &output)?;
    copy_directory(&assets, &output.join("assets"))?;
    ensure_builtin_font(&output.join("assets"))?;
    validate_assets_dir(&output.join("assets"), true)?;
    write_launchers(&output, &executable_name)?;
    write_project_package_readme(&output, &executable_name)?;
    if zip {
        zip_package(&output)?;
    }

    println!("packaged project at {}", output.display());
    Ok(())
}

fn package_workspace_demo(
    workspace: &Path,
    requested_output: &Path,
    features: &[String],
) -> Result<()> {
    let output = absolutize_from(workspace, requested_output);
    ensure_empty_or_missing(&output)?;

    let assets = workspace.join("assets");
    validate_assets_dir(&assets, true)?;

    let mut build = Command::new("cargo");
    build.args(["build", "-p", "game", "--release", "--locked"]);
    for feature in features {
        build.arg("--features").arg(feature);
    }
    let status = build
        .current_dir(workspace)
        .status()
        .context("could not run cargo build for package-demo")?;
    if !status.success() {
        bail!("release build failed; shaders are not confirmed and no package was created");
    }

    let target = env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace.join("target"));
    let executable_name = executable_name("game");
    let executable = target.join("release").join(&executable_name);
    if !executable.is_file() {
        bail!(
            "release build completed but '{}' was not produced",
            executable.display()
        );
    }

    fs::create_dir_all(&output)
        .with_context(|| format!("failed to create package output '{}'", output.display()))?;
    fs::copy(&executable, output.join(&executable_name)).with_context(|| {
        format!(
            "failed to copy packaged executable '{}'",
            executable.display()
        )
    })?;
    copy_runtime_libraries(&target.join("release"), &output)?;
    copy_directory(&assets, &output.join("assets"))?;
    write_launchers(&output, &executable_name)?;
    write_workspace_package_readme(&output, &executable_name)?;

    println!("packaged release demo at {}", output.display());
    println!("send the entire directory, including assets/, to a player");
    Ok(())
}

fn ensure_empty_or_missing(output: &Path) -> Result<()> {
    if output.exists()
        && fs::read_dir(output)
            .with_context(|| format!("failed to read package destination '{}'", output.display()))?
            .next()
            .is_some()
    {
        bail!(
            "package destination '{}' already exists and is not empty; choose a new --out directory",
            output.display()
        );
    }
    Ok(())
}

fn validate_assets_dir(assets: &Path, require_builtin_font: bool) -> Result<()> {
    if !assets.is_dir() {
        bail!("assets directory '{}' does not exist", assets.display());
    }
    if require_builtin_font {
        let font = assets.join("fonts/DejaVuSans.ttf");
        if !font.is_file() {
            bail!("required release font '{}' does not exist", font.display());
        }
    }

    let mut checked = 0usize;
    for entry in WalkDir::new(assets) {
        let entry =
            entry.with_context(|| format!("could not walk assets '{}'", assets.display()))?;
        if !entry.file_type().is_file() {
            continue;
        }
        checked += 1;
        validate_asset_file(entry.path())?;
    }
    if checked == 0 {
        bail!("assets directory '{}' is empty", assets.display());
    }
    Ok(())
}

fn validate_asset_file(path: &Path) -> Result<()> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some(extension) if extension.eq_ignore_ascii_case("png") => {
            let image = ImageReader::open(path)
                .with_context(|| format!("could not open PNG '{}'", path.display()))?
                .with_guessed_format()
                .with_context(|| format!("could not identify PNG '{}'", path.display()))?
                .decode()
                .with_context(|| format!("could not decode PNG '{}'", path.display()))?;
            let width = image.width();
            let height = image.height();
            if width == 0 || height == 0 {
                bail!("PNG '{}' has zero width or height", path.display());
            }
            if width > 8192 || height > 8192 {
                bail!(
                    "PNG '{}' is {}x{}, which is unusually large for a beginner asset; keep textures at 8192px or smaller on each side",
                    path.display(),
                    width,
                    height
                );
            }
        }
        Some(extension) if extension.eq_ignore_ascii_case("ttf") => {
            let bytes = fs::read(path)
                .with_context(|| format!("could not read font '{}'", path.display()))?;
            Font::from_bytes(bytes, FontSettings::default())
                .map_err(|error| anyhow!("could not parse font '{}': {error}", path.display()))?;
        }
        Some(extension)
            if matches!(
                extension.to_ascii_lowercase().as_str(),
                "wav" | "ogg" | "mp3"
            ) =>
        {
            game_audio::validate_file_sound(path)
                .with_context(|| format!("could not decode sound '{}'", path.display()))?;
        }
        Some(extension) if extension.eq_ignore_ascii_case("txt") => {
            validate_text_map(path)?;
        }
        Some(extension) if extension.eq_ignore_ascii_case("tmx") => {
            game_map::load_tiled_map_file(path)
                .with_context(|| format!("could not validate TMX map '{}'", path.display()))?;
        }
        Some(extension) if extension.eq_ignore_ascii_case("ldtk") => {
            let text = fs::read_to_string(path)
                .with_context(|| format!("could not read LDtk project '{}'", path.display()))?;
            serde_json::from_str::<serde_json::Value>(&text)
                .with_context(|| format!("could not parse LDtk project '{}'", path.display()))?;
        }
        Some(extension)
            if extension.eq_ignore_ascii_case("ron")
                && path.file_name().and_then(OsStr::to_str) == Some("game.ron") =>
        {
            game_kit::data::validate_beginner_game_file(path).with_context(|| {
                format!("could not validate beginner data file '{}'", path.display())
            })?;
        }
        Some(extension)
            if extension.eq_ignore_ascii_case("ron") && is_animation_metadata_file(path) =>
        {
            game_kit::assets::validate_animation_sheet_file(path).with_context(|| {
                format!("could not validate animation metadata '{}'", path.display())
            })?;
        }
        _ => {}
    }
    Ok(())
}

fn is_animation_metadata_file(path: &Path) -> bool {
    path.parent()
        .and_then(Path::file_name)
        .and_then(OsStr::to_str)
        == Some("animations")
}

fn validate_text_map(path: &Path) -> Result<()> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("could not read text map '{}'", path.display()))?;
    let rows = text
        .lines()
        .map(|line| line.trim_end_matches('\r'))
        .collect::<Vec<_>>();
    let Some(first) = rows.first() else {
        bail!("text map '{}' has no rows", path.display());
    };
    let width = first.chars().count();
    if width == 0 {
        bail!("text map '{}' has an empty first row", path.display());
    }
    for (index, row) in rows.iter().enumerate() {
        if row.chars().count() != width {
            bail!(
                "text map '{}' row {} has width {}, expected {width}",
                path.display(),
                index + 1,
                row.chars().count()
            );
        }
        if row.chars().any(char::is_whitespace) {
            bail!(
                "text map '{}' row {} contains whitespace; use visible tile symbols only",
                path.display(),
                index + 1
            );
        }
    }
    Ok(())
}

fn copy_directory(source: &Path, destination: &Path) -> Result<()> {
    for entry in WalkDir::new(source) {
        let entry = entry.with_context(|| format!("could not walk '{}'", source.display()))?;
        let relative = entry
            .path()
            .strip_prefix(source)
            .expect("walk entry is under its source directory");
        let target = destination.join(relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)
                .with_context(|| format!("failed to create '{}'", target.display()))?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create '{}'", parent.display()))?;
            }
            fs::copy(entry.path(), &target).with_context(|| {
                format!(
                    "failed to copy asset '{}' to '{}'",
                    entry.path().display(),
                    target.display()
                )
            })?;
        }
    }
    Ok(())
}

fn copy_runtime_libraries(build_dir: &Path, output: &Path) -> Result<()> {
    for name in [
        "libSDL3.so.0",
        "libSDL3.0.dylib",
        "libSDL3.dylib",
        "SDL3.dll",
    ] {
        let source = build_dir.join(name);
        if source.is_file() {
            fs::copy(&source, output.join(name)).with_context(|| {
                format!(
                    "failed to copy runtime library '{}' to '{}'",
                    source.display(),
                    output.display()
                )
            })?;
        }
    }
    Ok(())
}

fn ensure_builtin_font(assets: &Path) -> Result<()> {
    let target = assets.join("fonts/DejaVuSans.ttf");
    if target.is_file() {
        return Ok(());
    }
    let source = source_assets_dir().join("fonts/DejaVuSans.ttf");
    if !source.is_file() {
        bail!(
            "release packages need assets/fonts/DejaVuSans.ttf, but '{}' was not found; add that font to your project assets",
            source.display()
        );
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create '{}'", parent.display()))?;
    }
    fs::copy(&source, &target).with_context(|| {
        format!(
            "failed to copy bundled font '{}' to '{}'",
            source.display(),
            target.display()
        )
    })?;
    Ok(())
}

fn write_launchers(output: &Path, executable_name: &str) -> Result<()> {
    let shell = output.join("run.sh");
    fs::write(
        &shell,
        format!(
            "#!/usr/bin/env sh\ncd \"$(dirname \"$0\")\"\npackage_dir=$(pwd)\nif [ -n \"${{LD_LIBRARY_PATH:-}}\" ]; then\n  export LD_LIBRARY_PATH=\"$package_dir:$LD_LIBRARY_PATH\"\nelse\n  export LD_LIBRARY_PATH=\"$package_dir\"\nfi\nif [ -n \"${{DYLD_LIBRARY_PATH:-}}\" ]; then\n  export DYLD_LIBRARY_PATH=\"$package_dir:$DYLD_LIBRARY_PATH\"\nelse\n  export DYLD_LIBRARY_PATH=\"$package_dir\"\nfi\nexec ./{executable_name} \"$@\"\n"
        ),
    )
    .with_context(|| format!("failed to write '{}'", shell.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&shell, fs::Permissions::from_mode(0o755))
            .with_context(|| format!("failed to mark '{}' executable", shell.display()))?;
    }

    let powershell = output.join("run.ps1");
    fs::write(
        &powershell,
        format!(
            "Set-Location -LiteralPath $PSScriptRoot\r\n& .\\{executable_name} @args\r\nexit $LASTEXITCODE\r\n"
        ),
    )
    .with_context(|| format!("failed to write '{}'", powershell.display()))?;

    let batch = output.join("run.bat");
    fs::write(
        &batch,
        "@echo off\r\ncd /d \"%~dp0\"\r\npowershell -ExecutionPolicy Bypass -File .\\run.ps1 %*\r\n",
    )
    .with_context(|| format!("failed to write '{}'", batch.display()))?;
    Ok(())
}

fn write_project_package_readme(output: &Path, executable_name: &str) -> Result<()> {
    let readme = output.join("README.txt");
    fs::write(
        &readme,
        format!(
            "Playable game package\n\nKeep this directory together: `{executable_name}` needs the adjacent `assets` folder. If runtime library files such as SDL3 are included, keep them beside the executable too.\n\nLinux/macOS: run ./run.sh from a terminal.\nWindows: right-click run.ps1 and choose Run with PowerShell, or double-click run.bat.\n\nRuntime requirements\n\nThis build requires a Vulkan-capable GPU and driver. If it fails to start, install or update your Vulkan runtime/driver.\n\nLinux: install the Vulkan loader/tools package and your GPU vendor driver. Mesa/lavapipe can run smoke tests but is not ideal for players.\nWindows: update your graphics driver; the Vulkan Runtime is usually included with current NVIDIA, AMD, and Intel drivers.\nmacOS: run through MoltenVK/Vulkan SDK support; this command does not create a .app bundle.\n"
        ),
    )
    .with_context(|| format!("failed to write '{}'", readme.display()))
}

fn write_workspace_package_readme(output: &Path, executable_name: &str) -> Result<()> {
    let readme = output.join("README.txt");
    fs::write(
        &readme,
        format!(
            "Playable game package\n\nKeep this directory together: `{executable_name}` needs the adjacent `assets` folder. If runtime library files such as SDL3 are included, keep them beside the executable too.\n\nLinux: run ./run.sh from a terminal.\nWindows: right-click run.ps1 and choose Run with PowerShell, or double-click run.bat.\nmacOS: open Terminal in this folder and run ./run.sh; an app bundle is not created by this command.\n\nRuntime requirements\n\nThis build requires a Vulkan-capable GPU and driver. If it fails to start, install or update your Vulkan runtime/driver.\n\nLinux: install the Vulkan loader/tools package and your GPU vendor driver. Mesa/lavapipe can run smoke tests but is not ideal for players.\nWindows: update your graphics driver; the Vulkan Runtime is usually included with current NVIDIA, AMD, and Intel drivers.\nmacOS: run through MoltenVK/Vulkan SDK support; this command does not create a .app bundle.\n\nThe bundled binary defaults to the Arena demo. Set GAME_DEMO=simple or GAME_DEMO=testbed before launching to select those bundled demos.\n"
        ),
    )
    .with_context(|| format!("failed to write '{}'", readme.display()))
}

fn zip_package(output: &Path) -> Result<()> {
    let zip_path = output.with_extension("zip");
    if zip_path.exists() {
        bail!(
            "zip destination '{}' already exists; remove it or choose another --out path",
            zip_path.display()
        );
    }
    let status = Command::new("zip")
        .args(["-r"])
        .arg(&zip_path)
        .arg(".")
        .current_dir(output)
        .status()
        .context("could not run `zip`; install zip or omit --zip")?;
    if !status.success() {
        bail!("zip command failed while packaging '{}'", output.display());
    }
    println!("wrote {}", zip_path.display());
    Ok(())
}

fn doctor(options: DoctorOptions) {
    println!("game environment doctor\n");

    report_check(
        check_cargo(),
        "Cargo",
        "Cargo builds and runs your generated project.",
        "Install Rust from https://rustup.rs, then open a new terminal.",
        "Cargo is the Rust build tool. Without it, `game-dev run` and `game-dev package` cannot call `cargo run` or `cargo build`.",
        options,
    );

    report_check(
        check_rustc_version(),
        "rustc 1.87 or newer",
        "The framework uses Rust 2024 and dependencies that require Rust 1.87+.",
        "Run `rustup update stable`, or install Rust from https://rustup.rs.",
        "The project declares rust-version = 1.87 so Cargo can give consistent errors across machines.",
        options,
    );

    let shader = executable_on_path("glslc") || executable_on_path("shaderc");
    report_check(
        shader,
        "shader compiler (glslc or shaderc)",
        "Shaders must compile before the Vulkan renderer can start.",
        &shader_install_hint(),
        "Debug and release builds both need a GLSL compiler. `glslc` comes from the Vulkan SDK or distro shaderc packages.",
        options,
    );

    let vulkan_summary = command_output("vulkaninfo", &["--summary"]);
    let vulkan = vulkan_summary
        .as_ref()
        .is_some_and(|output| output.status.success());
    report_check(
        vulkan,
        "Vulkan loader and a usable driver",
        "The windowed renderer needs Vulkan before it can open the game.",
        &vulkan_install_hint(),
        "If `vulkaninfo --summary` fails, the renderer cannot choose a physical device. Install the loader, tools, and the GPU vendor driver.",
        options,
    );

    let vulkan_13 = vulkan_summary
        .as_ref()
        .and_then(|output| String::from_utf8(output.stdout.clone()).ok())
        .is_some_and(|text| has_vulkan_13_device(&text));
    report_check(
        vulkan_13,
        "Vulkan 1.3+ physical device",
        "The renderer expects a Vulkan 1.3 capable GPU or software driver.",
        &vulkan_install_hint(),
        "Integrated GPUs are fine if their driver reports Vulkan 1.3 or newer. On CI, lavapipe provides a software device.",
        options,
    );

    let sdl3 =
        command_succeeds("pkg-config", &["--exists", "sdl3"]) || env::var_os("SDL3_DIR").is_some();
    report_check(
        sdl3,
        "SDL3 development files",
        "SDL3 creates the window, reads input, and opens the audio device.",
        &sdl3_install_hint(),
        "On Linux, `pkg-config --exists sdl3` should succeed. On Windows, set SDL3_DIR when using vcpkg.",
        options,
    );

    let audio = audio_prerequisites_available();
    report_check(
        audio,
        "audio backend prerequisites",
        "The mixer needs the platform audio development files that SDL3 links against.",
        &audio_install_hint(),
        "Linux source builds commonly need ALSA development files. Windows and macOS usually get this through their normal SDK/Homebrew setup.",
        options,
    );

    let assets = env::current_dir()
        .map(|dir| dir.join("assets"))
        .is_ok_and(|path| path.is_dir());
    report_check(
        assets,
        "assets/ folder",
        "Generated games load maps, textures, sounds, and data from assets/.",
        "Run this command from your generated project folder, or create the project with `game-dev new my-game`.",
        "The runtime searches for an assets folder next to your project or packaged binary. Running commands from another directory is a common first-run mistake.",
        options,
    );

    let font = env::current_dir()
        .map(|dir| dir.join("assets/fonts/DejaVuSans.ttf"))
        .is_ok_and(|path| path.is_file());
    report_optional_check(
        font,
        "assets/fonts/DejaVuSans.ttf",
        "Release packages need the built-in UI font beside assets/.",
        "For generated projects, `game-dev package --release --out dist/my-game` adds the bundled font automatically. To run a hand-made release layout, copy assets/fonts/DejaVuSans.ttf into your project.",
        "Debug builds can use the framework checkout fallback, but release packages should be self-contained.",
        options,
    );

    let validation = vulkan
        && vulkan_summary.as_ref().is_some_and(|output| {
            String::from_utf8_lossy(&output.stdout).contains("VK_LAYER_KHRONOS_validation")
        });
    report_optional_check(
        validation,
        "Vulkan validation layers",
        "Validation layers turn many renderer mistakes into readable debug messages.",
        &validation_install_hint(),
        "If your machine cannot provide validation layers, use GAME_DISABLE_VALIDATION=1 only as a local fallback. Keep validation enabled when debugging renderer issues.",
        options,
    );

    if shader && vulkan && vulkan_13 && sdl3 && audio && assets {
        println!("\nCore prerequisites look available. Try: game-dev run");
    } else {
        println!("\nFix the failed checks above, then run this command again.");
    }
}

fn report_check(
    ok: bool,
    name: &str,
    why: &str,
    fix: &str,
    explanation: &str,
    options: DoctorOptions,
) {
    if ok {
        println!("[ok] {name}");
    } else {
        println!("[fail] {name}");
        println!("  why: {why}");
        println!("  fix: {fix}");
        if options.explain {
            println!("  details: {explanation}");
        }
    }
}

fn report_optional_check(
    ok: bool,
    name: &str,
    why: &str,
    fix: &str,
    explanation: &str,
    options: DoctorOptions,
) {
    if ok {
        println!("[ok] {name}");
    } else {
        println!("[warn] {name}");
        println!("  why: {why}");
        println!("  fix: {fix}");
        if options.explain {
            println!("  details: {explanation}");
        }
    }
}

fn command_succeeds(command: &str, args: &[&str]) -> bool {
    command_output(command, args).is_some_and(|output| output.status.success())
}

fn command_output(command: &str, args: &[&str]) -> Option<std::process::Output> {
    Command::new(command).args(args).output().ok()
}

fn check_cargo() -> bool {
    command_succeeds("cargo", &["--version"])
}

fn check_rustc_version() -> bool {
    command_output("rustc", &["--version"])
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|text| parse_rustc_minor(&text))
        .is_some_and(|(major, minor)| major > 1 || (major == 1 && minor >= 87))
}

fn parse_rustc_minor(text: &str) -> Option<(u32, u32)> {
    let version = text.split_whitespace().nth(1)?;
    let mut parts = version.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    Some((major, minor))
}

fn has_vulkan_13_device(summary: &str) -> bool {
    summary.lines().any(|line| {
        let Some(version) = line.split("apiVersion").nth(1) else {
            return false;
        };
        let Some(version) = version.split('=').nth(1) else {
            return false;
        };
        let mut parts = version.trim().split('.');
        let major = parts.next().and_then(|part| part.parse::<u32>().ok());
        let minor = parts.next().and_then(|part| part.parse::<u32>().ok());
        matches!((major, minor), (Some(major), Some(minor)) if major > 1 || (major == 1 && minor >= 3))
    })
}

fn audio_prerequisites_available() -> bool {
    if cfg!(target_os = "linux") {
        command_succeeds("pkg-config", &["--exists", "alsa"])
            || command_succeeds("pkg-config", &["--exists", "libpipewire-0.3"])
    } else {
        true
    }
}

fn shader_install_hint() -> String {
    match platform_hint() {
        PlatformHint::Linux(command) => {
            format!("{command}; package names are usually glslc or shaderc")
        }
        PlatformHint::Macos => "brew install shaderc".to_string(),
        PlatformHint::Windows => "winget install KhronosGroup.VulkanSDK".to_string(),
        PlatformHint::Other => {
            "install the Vulkan SDK or your platform's shaderc/glslc package".to_string()
        }
    }
}

fn vulkan_install_hint() -> String {
    match platform_hint() {
        PlatformHint::Linux(command) => {
            format!("{command}; also install your GPU vendor Vulkan driver if needed")
        }
        PlatformHint::Macos => "brew install molten-vk vulkan-tools".to_string(),
        PlatformHint::Windows => "winget install KhronosGroup.VulkanSDK".to_string(),
        PlatformHint::Other => "install Vulkan loader, tools, and a Vulkan 1.3 driver".to_string(),
    }
}

fn validation_install_hint() -> String {
    match platform_hint() {
        PlatformHint::Linux(command) => format!("{command}; include vulkan-validationlayers"),
        PlatformHint::Macos => "brew install vulkan-tools".to_string(),
        PlatformHint::Windows => "winget install KhronosGroup.VulkanSDK".to_string(),
        PlatformHint::Other => "install Vulkan validation layers for your platform".to_string(),
    }
}

fn sdl3_install_hint() -> String {
    match platform_hint() {
        PlatformHint::Linux(command) => format!("{command}; include SDL3 development files"),
        PlatformHint::Macos => "brew install sdl3".to_string(),
        PlatformHint::Windows => {
            "vcpkg install sdl3:x64-windows; set SDL3_DIR to the installed prefix".to_string()
        }
        PlatformHint::Other => "install SDL3 development files for your platform".to_string(),
    }
}

fn audio_install_hint() -> String {
    match platform_hint() {
        PlatformHint::Linux(command) => format!("{command}; include ALSA development files"),
        PlatformHint::Macos => {
            "Homebrew SDL3 setup normally covers audio prerequisites".to_string()
        }
        PlatformHint::Windows => {
            "The Windows SDK and SDL3/vcpkg setup normally cover audio prerequisites".to_string()
        }
        PlatformHint::Other => "install your platform audio development files".to_string(),
    }
}

enum PlatformHint {
    Linux(String),
    Macos,
    Windows,
    Other,
}

fn platform_hint() -> PlatformHint {
    if cfg!(target_os = "windows") {
        return PlatformHint::Windows;
    }
    if cfg!(target_os = "macos") {
        return PlatformHint::Macos;
    }
    if cfg!(target_os = "linux") {
        return PlatformHint::Linux(linux_install_command());
    }
    PlatformHint::Other
}

fn linux_install_command() -> String {
    let id_like = fs::read_to_string("/etc/os-release").unwrap_or_default();
    let lower = id_like.to_ascii_lowercase();
    if lower.contains("id=ubuntu") || lower.contains("id=debian") || lower.contains("debian") {
        "sudo apt install build-essential pkg-config libsdl3-dev vulkan-tools vulkan-validationlayers glslc libasound2-dev".to_string()
    } else if lower.contains("id=fedora") || lower.contains("fedora") {
        "sudo dnf install gcc pkgconf-pkg-config SDL3-devel vulkan-tools vulkan-validation-layers shaderc alsa-lib-devel".to_string()
    } else if lower.contains("id=arch") || lower.contains("arch") {
        "sudo pacman -S --needed base-devel pkgconf sdl3 vulkan-tools vulkan-validation-layers shaderc alsa-lib".to_string()
    } else if lower.contains("id=gentoo") || lower.contains("gentoo") {
        "sudo emerge --ask media-libs/libsdl3 media-libs/shaderc dev-util/vulkan-tools media-libs/alsa-lib".to_string()
    } else {
        "install SDL3 development files, vulkan-tools, validation layers, shaderc/glslc, and ALSA development files".to_string()
    }
}

fn executable_on_path(name: &str) -> bool {
    let Some(paths) = env::var_os("PATH") else {
        return false;
    };
    env::split_paths(&paths).any(|dir| {
        let plain = dir.join(name);
        if plain.is_file() {
            return true;
        }
        #[cfg(windows)]
        {
            return dir.join(format!("{name}.exe")).is_file();
        }
        #[cfg(not(windows))]
        false
    })
}

struct PackageManifestInfo {
    package_name: String,
    asset_dir: PathBuf,
}

fn package_info_from_manifest(manifest: &Path) -> Result<PackageManifestInfo> {
    let source = fs::read_to_string(manifest)
        .with_context(|| format!("failed to read manifest '{}'", manifest.display()))?;
    let mut section = "";
    let mut package_name = None;
    let mut asset_dir = None;
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            section = trimmed.trim_matches(&['[', ']'][..]);
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim().trim_matches('"');
        match (section, key) {
            ("package", "name") if !value.is_empty() => package_name = Some(value.to_string()),
            ("package.metadata.game", "asset_dir") if !value.is_empty() => {
                asset_dir = Some(PathBuf::from(value));
            }
            _ => {}
        }
    }
    let package_name = package_name
        .ok_or_else(|| anyhow!("could not find [package] name in '{}'", manifest.display()))?;
    Ok(PackageManifestInfo {
        package_name,
        asset_dir: asset_dir.unwrap_or_else(|| PathBuf::from("assets")),
    })
}

fn crate_name_from_destination(destination: &Path) -> Result<String> {
    let file_name = destination
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| {
            anyhow!(
                "destination '{}' has no final path segment",
                destination.display()
            )
        })?;
    let mut name = String::new();
    let mut last_was_dash = false;
    for ch in file_name.chars() {
        let ch = ch.to_ascii_lowercase();
        if ch.is_ascii_alphanumeric() {
            name.push(ch);
            last_was_dash = false;
        } else if !last_was_dash && !name.is_empty() {
            name.push('-');
            last_was_dash = true;
        }
    }
    while name.ends_with('-') {
        name.pop();
    }
    if name.is_empty() {
        bail!("could not derive a crate name from '{}'", file_name);
    }
    Ok(name)
}

fn title_from_crate_name(crate_name: &str) -> String {
    crate_name
        .split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn xtask_demo_destination(workspace: &Path, name_or_path: &str) -> Result<PathBuf> {
    let raw = Path::new(name_or_path);
    if raw.is_absolute() || raw.components().count() > 1 {
        return Ok(raw.to_path_buf());
    }

    let parent = workspace
        .parent()
        .ok_or_else(|| anyhow!("workspace '{}' has no parent", workspace.display()))?;
    Ok(parent.join(raw))
}

fn game_path_from_destination(workspace: &Path, destination: &Path) -> Result<String> {
    if destination.parent() == workspace.parent() {
        let root_name = workspace
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or_else(|| {
                anyhow!(
                    "workspace '{}' has no final path segment",
                    workspace.display()
                )
            })?;
        Ok(format!("../{root_name}"))
    } else {
        Ok(workspace.display().to_string())
    }
}

fn workspace_root() -> Result<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| anyhow!("game-cli manifest has no workspace parent"))
}

fn source_assets_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets")
}

fn absolutize_from_current(path: &Path) -> Result<PathBuf> {
    Ok(absolutize_from(&env::current_dir()?, path))
}

fn absolutize_from(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

fn executable_name(package_name: &str) -> String {
    if cfg!(windows) {
        format!("{package_name}.exe")
    } else {
        package_name.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        crate_name_from_destination, normalize_validate_data_path, package_info_from_manifest,
        validate_asset_file, validate_text_map,
    };

    #[test]
    fn crate_name_is_sanitized_from_destination() {
        assert_eq!(
            crate_name_from_destination(std::path::Path::new("My First Game")).unwrap(),
            "my-first-game"
        );
    }

    #[test]
    fn package_name_parser_reads_package_section() {
        let path =
            std::env::temp_dir().join(format!("game-cli-manifest-{}.toml", std::process::id()));
        std::fs::write(&path, "[package]\nname = \"demo_game\"\n").unwrap();
        assert_eq!(
            package_info_from_manifest(&path).unwrap().package_name,
            "demo_game"
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn package_metadata_parser_reads_asset_dir_with_default() {
        let path = std::env::temp_dir().join(format!(
            "game-cli-metadata-manifest-{}.toml",
            std::process::id()
        ));
        std::fs::write(
            &path,
            "[package]\nname = \"demo_game\"\n\n[package.metadata.game]\nasset_dir = \"game-assets\"\n",
        )
        .unwrap();
        let info = package_info_from_manifest(&path).unwrap();
        assert_eq!(info.package_name, "demo_game");
        assert_eq!(info.asset_dir, std::path::PathBuf::from("game-assets"));

        std::fs::write(&path, "[package]\nname = \"fallback_game\"\n").unwrap();
        let info = package_info_from_manifest(&path).unwrap();
        assert_eq!(info.package_name, "fallback_game");
        assert_eq!(info.asset_dir, std::path::PathBuf::from("assets"));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn text_map_validation_names_the_ragged_row() {
        let path = std::env::temp_dir().join(format!(
            "game-cli-map-validation-{}.txt",
            std::process::id()
        ));
        std::fs::write(&path, "####\n##\n").unwrap();
        let error = validate_text_map(&path).unwrap_err().to_string();
        assert!(error.contains("row 2 has width 2, expected 4"));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn validate_data_accepts_asset_relative_or_assets_prefixed_paths() {
        let asset_root = std::path::Path::new("assets");
        assert_eq!(
            normalize_validate_data_path("game.ron", asset_root),
            std::path::PathBuf::from("game.ron")
        );
        assert_eq!(
            normalize_validate_data_path("assets/game.ron", asset_root),
            std::path::PathBuf::from("game.ron")
        );

        let absolute = std::env::temp_dir().join("game.ron");
        assert_eq!(
            normalize_validate_data_path(&absolute, asset_root),
            absolute
        );
    }

    #[test]
    fn asset_check_validates_animation_metadata_texture_references() {
        let root = std::env::temp_dir().join(format!(
            "game-cli-animation-validation-{}",
            std::process::id()
        ));
        let animations = root.join("assets/animations");
        std::fs::create_dir_all(&animations).unwrap();
        let path = animations.join("player.ron");
        std::fs::write(
            &path,
            r#"(
    texture: "textures/player_sheet.png",
    columns: 4,
    rows: 1,
    clips: {"idle": (frames: [0])},
)"#,
        )
        .unwrap();

        let error = format!("{:?}", validate_asset_file(&path).unwrap_err());

        assert!(error.contains("could not validate animation metadata"));
        assert!(error.contains("references missing texture 'textures/player_sheet.png'"));

        std::fs::remove_dir_all(root).unwrap();
    }
}
