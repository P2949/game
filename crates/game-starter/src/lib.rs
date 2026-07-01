use anyhow::Result;
use game_kit::app::{GameApp, plugin_fn};
use game_runtime::RuntimeConfig;

pub mod prelude {
    pub use anyhow::{Context, Result};
    pub use game_kit::beginner::prelude::*;

    pub use crate::{run_game, run_game_with};
}

pub fn run_game<F>(title: &str, build: F) -> Result<()>
where
    F: for<'app> Fn(&mut GameApp<'app>) -> Result<()>,
{
    run_game_with(RuntimeConfig::default().title(title), build)
}

pub fn run_game_with<F>(config: RuntimeConfig, build: F) -> Result<()>
where
    F: for<'app> Fn(&mut GameApp<'app>) -> Result<()>,
{
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init();

    game_runtime::run(config, plugin_fn(build))
}
