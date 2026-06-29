fn main() -> anyhow::Result<()> {
    game_cli::run_xtask(std::env::args().skip(1))
}
