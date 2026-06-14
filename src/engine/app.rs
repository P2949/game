use std::time::{Duration, Instant};

use anyhow::Result;
use glam::Vec2;

use crate::engine::audio::Audio;
use crate::engine::camera::Camera2D;
use crate::engine::gfx::Gfx;
use crate::engine::input::Input;
use crate::engine::nav::NavGrid;
use crate::engine::tilemap::{Tile, TileMap};
use crate::engine::world::{Sprite, World};
use crate::platform::input::FrameActions;
use crate::platform::time::FixedTimestep;
use crate::platform::window::Platform;
use crate::renderer::context::{RenderOutcome, VulkanContext};
use crate::renderer::{DrawCommands, SpriteDraw};

const RESIZE_IDLE_SLEEP: Duration = Duration::from_millis(16);
const LAG_WARNING_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Clone, Copy)]
pub struct TileTheme {
    pub floor: Sprite,
    pub wall: Sprite,
}

struct MapData {
    tilemap: TileMap,
    nav: NavGrid,
    theme: TileTheme,
}

pub trait Game {
    type Actor;

    fn start(&mut self, ctx: &mut StartCtx<Self::Actor>) -> Result<()>;
    fn update(&mut self, ctx: &mut Ctx<Self::Actor>, dt: f32);

    fn record_frame_time(&mut self, _ms: f32) {}
}

pub struct StartCtx<'a, U> {
    pub world: &'a mut World<U>,
    map: &'a mut Option<MapData>,
}

impl<'a, U> StartCtx<'a, U> {
    pub fn set_map(&mut self, tilemap: TileMap, theme: TileTheme) {
        let nav = NavGrid::from_tilemap(&tilemap);
        *self.map = Some(MapData {
            tilemap,
            nav,
            theme,
        });
    }
}

pub struct Ctx<'a, U> {
    pub world: &'a mut World<U>,
    pub map: &'a TileMap,
    pub nav: &'a NavGrid,
    pub input: &'a Input,
    pub camera: &'a mut Camera2D,
    pub gfx: Gfx<'a>,
    pub audio: Audio<'a>,
}

pub fn run<G: Game>(mut game: G, title: &str) -> Result<()> {
    let smoke_frames = parse_smoke_frames()?;

    let mut platform = Platform::new(title, 1280, 720)?;
    let mut renderer = VulkanContext::new(&platform.window)?;
    let audio = match crate::audio::AudioSystem::new(&platform.sdl) {
        Ok(audio) => Some(audio),
        Err(err) => {
            log::warn!("audio disabled: {err}");
            None
        }
    };
    let mut timestep = FixedTimestep::new(120.0);
    let mut camera = Camera2D::new(Vec2::ZERO, 1.0);
    let mut world: World<G::Actor> = World::new();
    let mut map_slot: Option<MapData> = None;

    {
        let mut start_ctx = StartCtx {
            world: &mut world,
            map: &mut map_slot,
        };
        game.start(&mut start_ctx)?;
    }
    let map = map_slot.expect("Game::start must call ctx.set_map(...)");

    if smoke_frames == Some(0) {
        log::info!("GAME_SMOKE_FRAMES=0 requested; initialized and exiting before rendering");
        return Ok(());
    }

    let mut pending_actions = FrameActions::default();
    let mut previous_frame = Instant::now();
    let mut last_lag_warning: Option<Instant> = None;
    let mut rendered_frames: u64 = 0;

    while !platform.should_quit {
        platform.pump_events();

        let (width, height) = platform.drawable_size();
        if width == 0 || height == 0 {
            previous_frame = Instant::now();
            timestep.reset_after_pause();
            std::thread::sleep(RESIZE_IDLE_SLEEP);
            continue;
        }

        if platform.take_stable_resize_request() {
            renderer.request_swapchain_recreate();
        }

        let now = Instant::now();
        let frame_ms = (now - previous_frame).as_secs_f32() * 1000.0;
        previous_frame = now;
        game.record_frame_time(frame_ms);

        let frame_actions = platform.input.take_frame_actions();
        if let Some(audio) = &audio {
            audio.poll_dropped_frames();
            audio.poll_dropped_voices();
        }
        pending_actions.merge(frame_actions);

        timestep.begin_frame();
        let mut steps = 0;
        while steps < FixedTimestep::MAX_STEPS_PER_FRAME {
            let Some(dt) = timestep.consume_step() else {
                break;
            };
            let actions = if steps == 0 {
                std::mem::take(&mut pending_actions)
            } else {
                FrameActions::default()
            };
            let input = Input::new(&platform.input, actions);

            {
                let mut ctx = Ctx {
                    world: &mut world,
                    map: &map.tilemap,
                    nav: &map.nav,
                    input: &input,
                    camera: &mut camera,
                    gfx: Gfx::new(&mut renderer),
                    audio: Audio::new(audio.as_ref()),
                };
                game.update(&mut ctx, dt);
            }

            steps += 1;
        }

        if timestep.step_ready() {
            let now = Instant::now();
            if last_lag_warning.is_none_or(|last| now.duration_since(last) >= LAG_WARNING_INTERVAL)
            {
                log::warn!(
                    "fixed timestep hit {} steps in one frame; discarding accumulated lag",
                    FixedTimestep::MAX_STEPS_PER_FRAME
                );
                last_lag_warning = Some(now);
            }
            timestep.discard_lag();
        }

        render_map(&mut renderer, &map);
        render_entities(&mut renderer, &world);

        if renderer.render(&platform.window, camera)? == RenderOutcome::Presented {
            rendered_frames += 1;
        }
        if let Some(limit) = smoke_frames {
            if rendered_frames >= limit {
                log::info!("GAME_SMOKE_FRAMES={limit} reached; exiting cleanly");
                platform.should_quit = true;
            }
        }
    }

    Ok(())
}

fn render_map(renderer: &mut VulkanContext, map: &MapData) {
    let tile_size = map.tilemap.tile_size();
    let size = Vec2::splat(tile_size);
    for row in 0..map.tilemap.height() {
        for col in 0..map.tilemap.width() {
            let sprite = match map.tilemap.tile(col, row) {
                Tile::Floor => map.theme.floor,
                Tile::Wall => map.theme.wall,
            };
            renderer.draw_world_sprite(SpriteDraw {
                texture: sprite.handle.0,
                layer: sprite.layer,
                position: Vec2::new(col as f32 * tile_size, row as f32 * tile_size),
                size,
                uv_min: Vec2::ZERO,
                uv_max: Vec2::ONE,
                color: sprite.color,
            });
        }
    }
}

fn render_entities<U>(renderer: &mut VulkanContext, world: &World<U>) {
    for (_, entity) in world.iter() {
        if let Some(sprite) = entity.sprite {
            renderer.draw_world_sprite(SpriteDraw {
                texture: sprite.handle.0,
                layer: sprite.layer,
                position: entity.transform.pos - sprite.size * 0.5,
                size: sprite.size,
                uv_min: Vec2::ZERO,
                uv_max: Vec2::ONE,
                color: sprite.color,
            });
        }
    }
}

fn parse_smoke_frames() -> anyhow::Result<Option<u64>> {
    let Ok(raw) = std::env::var("GAME_SMOKE_FRAMES") else {
        return Ok(None);
    };

    raw.trim()
        .parse::<u64>()
        .map(Some)
        .map_err(|_| anyhow::anyhow!("GAME_SMOKE_FRAMES must be a non-negative integer"))
}

#[cfg(test)]
mod tests {
    use super::parse_smoke_frames;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    #[test]
    fn smoke_frames_unset_means_interactive_run() {
        let _guard = env_lock();
        unsafe {
            std::env::remove_var("GAME_SMOKE_FRAMES");
        }

        assert_eq!(parse_smoke_frames().unwrap(), None);
    }

    #[test]
    fn smoke_frames_accepts_zero_and_positive_counts() {
        let _guard = env_lock();
        unsafe {
            std::env::set_var("GAME_SMOKE_FRAMES", "0");
        }
        assert_eq!(parse_smoke_frames().unwrap(), Some(0));

        unsafe {
            std::env::set_var("GAME_SMOKE_FRAMES", "120");
        }
        assert_eq!(parse_smoke_frames().unwrap(), Some(120));

        unsafe {
            std::env::remove_var("GAME_SMOKE_FRAMES");
        }
    }

    #[test]
    fn smoke_frames_trims_whitespace() {
        let _guard = env_lock();
        unsafe {
            std::env::set_var("GAME_SMOKE_FRAMES", " 120 ");
        }

        assert_eq!(parse_smoke_frames().unwrap(), Some(120));

        unsafe {
            std::env::remove_var("GAME_SMOKE_FRAMES");
        }
    }

    #[test]
    fn smoke_frames_rejects_invalid_values() {
        let _guard = env_lock();
        for value in ["", "-1", "abc", "1.5"] {
            unsafe {
                std::env::set_var("GAME_SMOKE_FRAMES", value);
            }
            assert!(parse_smoke_frames().is_err(), "accepted {value:?}");
        }

        unsafe {
            std::env::remove_var("GAME_SMOKE_FRAMES");
        }
    }
}
