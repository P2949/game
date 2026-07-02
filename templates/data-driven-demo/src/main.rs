use game_starter::prelude::*;

fn main() -> Result<()> {
    run_game("{{title}}", |game| {
        // Legacy RON setup lives in assets/game.ron. You can add ordinary Rust
        // behavior below this line later without giving up the data file.
        let _controls = game.load_beginner_file("game.ron")?;
        Ok(())
    })
}
