use anyhow::Result;
use glam::Vec2;

use crate::engine::audio::Audio;
use crate::engine::camera::Camera2D;
use crate::engine::gfx::{Gfx, SpriteDraw, TextDraw};
use crate::engine::input::Input;
use crate::engine::nav::NavGrid;
use crate::engine::tilemap::{Tile, TileMap};
use crate::engine::world::{Sprite, Transform, World};

#[derive(Clone, Copy)]
pub struct TileTheme {
    pub floor: Sprite,
    pub wall: Sprite,
}

pub struct MapData {
    pub tilemap: TileMap,
    pub nav: NavGrid,
    pub theme: TileTheme,
}

pub trait Game {
    fn start(&mut self, ctx: &mut StartCtx) -> Result<()>;
    fn update(&mut self, ctx: &mut Ctx, dt: f32);

    fn record_frame_time(&mut self, _ms: f32) {}
}

pub struct StartCtx<'a> {
    pub world: &'a mut World,
    map: &'a mut Option<MapData>,
}

impl<'a> StartCtx<'a> {
    pub fn new(world: &'a mut World, map: &'a mut Option<MapData>) -> Self {
        Self { world, map }
    }

    pub fn set_map(&mut self, tilemap: TileMap, theme: TileTheme) {
        let nav = NavGrid::from_tilemap(&tilemap);
        *self.map = Some(MapData {
            tilemap,
            nav,
            theme,
        });
    }
}

pub struct Ctx<'a> {
    pub world: &'a mut World,
    pub map: &'a TileMap,
    pub nav: &'a NavGrid,
    pub input: &'a Input,
    pub camera: &'a mut Camera2D,
    pub gfx: Gfx<'a>,
    pub audio: Audio<'a>,
}

pub struct RenderFrame {
    pub camera: Camera2D,
    pub world_sprites: Vec<SpriteDraw>,
    pub ui_sprites: Vec<SpriteDraw>,
    pub ui_text: Vec<TextDraw>,
}

impl RenderFrame {
    pub fn new(camera: Camera2D) -> Self {
        Self {
            camera,
            world_sprites: Vec::new(),
            ui_sprites: Vec::new(),
            ui_text: Vec::new(),
        }
    }
}

pub fn extract_tilemap_sprites(map: &MapData, out: &mut RenderFrame) {
    let tile_size = map.tilemap.tile_size();
    let size = Vec2::splat(tile_size);
    for row in 0..map.tilemap.height() {
        for col in 0..map.tilemap.width() {
            let sprite = match map.tilemap.tile(col, row) {
                Tile::Floor => map.theme.floor,
                Tile::Wall => map.theme.wall,
            };
            out.world_sprites.push(SpriteDraw {
                texture: sprite.texture,
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

pub fn extract_entity_sprites(world: &World, out: &mut RenderFrame) {
    for (_, transform, sprite) in world.query2::<Transform, Sprite>() {
        out.world_sprites.push(SpriteDraw {
            texture: sprite.texture,
            layer: sprite.layer,
            position: transform.pos - sprite.size * 0.5,
            size: sprite.size,
            uv_min: Vec2::ZERO,
            uv_max: Vec2::ONE,
            color: sprite.color,
        });
    }
}
