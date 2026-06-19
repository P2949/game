use game_kit::advanced::prelude::*;

/// Resolves a melee combat tick into engine commands: queued hit sounds and
/// despawns of dead enemies.
pub fn tick_commands(
    game: &mut GameCtx<'_, '_>,
    attack: ActionId,
    hit_sound: SoundHandle,
    dt: f32,
) {
    game.run_simple_melee_combat(attack, Some(hit_sound), dt);
}

pub fn kill_player(game: &mut GameCtx<'_, '_>) {
    game.kill_player();
}

pub fn player_is_dead(game: &GameCtx<'_, '_>) -> bool {
    game.player_is_dead()
}
