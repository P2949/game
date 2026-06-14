pub mod actor;
pub mod ai;
pub mod combat;
pub mod level;
pub mod spawn;

use anyhow::Result;

use crate::engine::app::{Ctx, Game, StartCtx};
use crate::engine::assets::Assets;
use crate::engine::input::Action;

pub type World = crate::engine::world::World<actor::Actor>;

pub struct ArenaGame {
    assets: Assets,
    paused: bool,
}

impl ArenaGame {
    pub fn new() -> Self {
        Self {
            assets: Assets::load(),
            paused: false,
        }
    }

    fn reset_world(&self, world: &mut World, map: &crate::engine::tilemap::TileMap) {
        world.clear();
        spawn::spawn_markers(world, map, &self.assets);
    }

    fn update_camera(&self, ctx: &mut Ctx<actor::Actor>, dt: f32) {
        let zoom_axis = ctx.input.zoom_axis();
        if zoom_axis != 0.0 {
            let zoom_step = 1.0 + 2.0 * dt;
            let mut zoom = ctx.camera.zoom();
            if zoom_axis > 0.0 {
                zoom *= zoom_step;
            } else {
                zoom /= zoom_step;
            }
            ctx.camera.set_zoom(zoom.clamp(0.25, 6.0));
        }

        if let Some(pos) = ai::player_pos(ctx.world) {
            ctx.camera.set_center(pos);
        }
    }
}

impl Default for ArenaGame {
    fn default() -> Self {
        Self::new()
    }
}

impl Game for ArenaGame {
    type Actor = actor::Actor;

    fn start(&mut self, ctx: &mut StartCtx<Self::Actor>) -> Result<()> {
        let map = level::arena();
        spawn::spawn_markers(ctx.world, &map, &self.assets);
        ctx.set_map(map, level::theme(&self.assets));
        Ok(())
    }

    fn update(&mut self, ctx: &mut Ctx<Self::Actor>, dt: f32) {
        if ctx.input.pressed(Action::Pause) {
            self.paused = !self.paused;
        }

        if ctx.input.pressed(Action::Reset) {
            self.reset_world(ctx.world, ctx.map);
            self.paused = false;
        }

        if ctx.input.pressed(Action::DebugDie) {
            combat::kill_player(ctx.world);
        }

        if combat::player_is_dead(ctx.world) {
            ai::stop_all(ctx.world);
            if ctx.input.pressed(Action::Attack) || ctx.input.pressed(Action::Reset) {
                self.reset_world(ctx.world, ctx.map);
                self.paused = false;
            } else {
                ctx.gfx.text(
                    "You died",
                    glam::vec2(24.0, 24.0),
                    glam::vec4(1.0, 0.35, 0.25, 1.0),
                );
                self.update_camera(ctx, dt);
                return;
            }
        }

        if self.paused {
            ai::stop_all(ctx.world);
            ctx.gfx.text(
                "Paused",
                glam::vec2(24.0, 24.0),
                glam::vec4(1.0, 0.95, 0.75, 1.0),
            );
            self.update_camera(ctx, dt);
            return;
        }

        ai::drive_player(ctx.world, ctx.input);
        ai::chase_player(ctx.world, ctx.nav, dt);
        crate::engine::physics::step(ctx.world, ctx.map, dt);
        combat::tick(ctx.world, ctx.input, ctx.audio, dt);
        self.update_camera(ctx, dt);
    }
}
