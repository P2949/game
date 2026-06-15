fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = game_runtime::RuntimeConfig::default();

    // Phase 12: the binary selects which content plugin to run. The runtime,
    // renderer, audio, and platform crates are identical for both demos.
    match std::env::var("GAME_DEMO").as_deref() {
        Ok("testbed") => game_runtime::run(config.title("Testbed"), testbed_content::plugin()),
        _ => game_runtime::run(config.title("Arena"), arena_content::plugin()),
    }
}
