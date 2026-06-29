fn main() -> anyhow::Result<()> {
    game_cli::run(std::env::args().skip(1))
}
