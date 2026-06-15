use glam::Vec2;

use crate::audio::Audio;
use crate::camera::Camera2D;
use crate::gfx::{Gfx, SpriteDraw, TextDraw};
use crate::input::Input;
use crate::nav::NavGrid;
use crate::tilemap::{Tile, TileMap};
use crate::world::{Sprite, Transform, World};

#[derive(Clone, Copy)]
pub struct TileTheme {
    pub floor: Sprite,
    pub wall: Sprite,
}

#[derive(Clone)]
pub struct MapData {
    pub tilemap: TileMap,
    pub nav: NavGrid,
    pub theme: TileTheme,
}

pub struct StartCtx<'a> {
    pub world: &'a mut World,
}

impl<'a> StartCtx<'a> {
    pub fn new(world: &'a mut World) -> Self {
        Self { world }
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
