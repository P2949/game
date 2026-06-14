mod audio;
mod engine;
mod game;
mod platform;
mod renderer;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    engine::run(game::ArenaGame::new(), "Arena")
}
