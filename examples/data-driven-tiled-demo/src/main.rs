use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("Data Driven Tiled Demo", |game| {
        game.load_beginner_file(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/game.ron"))?;
        Ok(())
    })
}
