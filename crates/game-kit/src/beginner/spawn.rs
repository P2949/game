//! Beginner prefab spawning verbs.

use game_map::MapCell;
use glam::{Vec2, vec2};

use crate::context::GameCtx;

pub struct SpawnAuthor<'g, 'a, 'w> {
    game: &'g mut GameCtx<'a, 'w>,
    prefab: String,
}

impl<'g, 'a, 'w> SpawnAuthor<'g, 'a, 'w> {
    pub(crate) fn new(game: &'g mut GameCtx<'a, 'w>, prefab: String) -> Self {
        Self { game, prefab }
    }

    pub fn at(self, cell: MapCell) {
        let position = self.game.cell_center(cell);
        self.at_world(position);
    }

    pub fn at_world(self, position: Vec2) {
        self.game.spawn_prefab_or_log(&self.prefab, position);
    }

    pub fn near_player(self, radius: f32) {
        let Some(player_pos) = self.game.player_position() else {
            return;
        };
        self.at_world(player_pos + vec2(radius.max(0.0), 0.0));
    }

    pub fn at_first_floor(self) {
        let Some(position) = self.game.first_floor_center() else {
            return;
        };
        self.at_world(position);
    }
}

impl<'a, 'w> GameCtx<'a, 'w> {
    pub fn spawn(&mut self, prefab: impl Into<String>) -> SpawnAuthor<'_, 'a, 'w> {
        SpawnAuthor::new(self, prefab.into())
    }
}

#[cfg(test)]
mod tests {
    use game_core::backend::TextureHandle;
    use game_map::cell;

    use crate::app::{GameApp, GamePlugin};
    use crate::context::{GameCtx, StartupGameCtx};
    use crate::harness::GameTestHarness;

    struct SpawnPlugin;

    impl GamePlugin for SpawnPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> anyhow::Result<()> {
            let controls = game.input(|input| input.top_down_controls())?;

            game.player_prefab("player")
                .sprite(TextureHandle(1))
                .moves_with(controls.movement, 130.0)
                .build()?;

            game.enemy_prefab("slime")
                .sprite(TextureHandle(2))
                .build()?;

            game.map("spawn")
                .tiles(["#####", "#P..#", "#####"])
                .simple_theme(TextureHandle(10), TextureHandle(11))
                .legend('P', "player")
                .start();

            game.on_start(|game: &mut StartupGameCtx<'_, '_>| game.spawn_start_map());
            game.every_tick(|game: &mut GameCtx<'_, '_>, _dt| {
                game.spawn("slime").at(cell(2, 1));
                game.spawn("slime").near_player(64.0);
                game.spawn("slime").at_first_floor();
            });

            Ok(())
        }
    }

    #[test]
    fn spawn_author_queues_beginner_spawns() {
        let mut game = GameTestHarness::from_plugin(SpawnPlugin).unwrap();

        game.step();

        assert_eq!(game.enemy_count(), 3);
    }
}
