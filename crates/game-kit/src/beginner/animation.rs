//! Beginner spritesheet and animation helpers.

use std::collections::HashMap;

use game_core::backend::TextureHandle;
use game_core::world::{EntityId, Sprite};
use glam::Vec2;

use crate::beginner::actors::Player;
use crate::context::GameCtx;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpriteSheet {
    pub texture: TextureHandle,
    pub columns: u32,
    pub rows: u32,
}

impl SpriteSheet {
    pub fn new(texture: TextureHandle, columns: u32, rows: u32) -> Self {
        Self {
            texture,
            columns: columns.max(1),
            rows: rows.max(1),
        }
    }

    pub fn frame_uv(&self, frame: usize) -> (Vec2, Vec2) {
        let total = (self.columns * self.rows) as usize;
        let frame = if total == 0 { 0 } else { frame % total };
        let col = frame as u32 % self.columns;
        let row = frame as u32 / self.columns;
        let frame_size = Vec2::new(1.0 / self.columns as f32, 1.0 / self.rows as f32);
        let uv_min = Vec2::new(col as f32 * frame_size.x, row as f32 * frame_size.y);
        (uv_min, uv_min + frame_size)
    }

    pub fn sprite(&self, frame: usize, size: Vec2) -> Sprite {
        let (uv_min, uv_max) = self.frame_uv(frame);
        Sprite::new(self.texture, size).region(uv_min, uv_max)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Animation {
    pub current: String,
    pub timer: f32,
    pub frame: usize,
}

impl Animation {
    pub fn play(name: impl Into<String>) -> Self {
        Self {
            current: name.into(),
            timer: 0.0,
            frame: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnimationClip {
    pub frames: Vec<usize>,
    pub fps: f32,
    pub looping: bool,
}

impl AnimationClip {
    pub fn frames(frames: impl IntoIterator<Item = usize>) -> Self {
        Self {
            frames: frames.into_iter().collect(),
            fps: 8.0,
            looping: true,
        }
    }

    pub fn fps(mut self, fps: f32) -> Self {
        self.fps = fps.max(0.001);
        self
    }

    pub fn looping(mut self) -> Self {
        self.looping = true;
        self
    }

    pub fn once(mut self) -> Self {
        self.looping = false;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnimationSet {
    pub sheet: SpriteSheet,
    animations: HashMap<String, AnimationClip>,
}

impl AnimationSet {
    pub fn new(sheet: SpriteSheet) -> Self {
        Self {
            sheet,
            animations: HashMap::new(),
        }
    }

    pub fn animation(mut self, name: impl Into<String>, clip: AnimationClip) -> Self {
        self.animations.insert(name.into(), clip);
        self
    }

    pub fn get(&self, name: &str) -> Option<&AnimationClip> {
        self.animations.get(name)
    }
}

impl<'a, 'w> GameCtx<'a, 'w> {
    pub fn play_animation(&mut self, id: EntityId, name: impl Into<String>) -> bool {
        let name = name.into();
        let has_clip = self
            .component::<AnimationSet>(id)
            .is_some_and(|set| set.get(&name).is_some());
        if !has_clip {
            return false;
        }

        let Some(animation) = self.component_mut::<Animation>(id) else {
            return false;
        };
        if animation.current != name {
            animation.current = name;
            animation.timer = 0.0;
            animation.frame = 0;
        }
        true
    }

    pub fn player(&mut self) -> PlayerActor<'_, 'a, 'w> {
        PlayerActor { game: self }
    }

    pub fn update_animations(&mut self, dt: f32) {
        let ids = self.entities_with::<Animation>();
        for id in ids {
            let Some((sheet, clip)) = self.component::<Animation>(id).and_then(|animation| {
                let set = self.component::<AnimationSet>(id)?;
                let clip = set.get(&animation.current)?;
                Some((set.sheet, clip.clone()))
            }) else {
                continue;
            };
            if clip.frames.is_empty() {
                continue;
            }

            let frame_index = {
                let Some(animation) = self.component_mut::<Animation>(id) else {
                    continue;
                };
                animation.timer += dt;
                let frame_seconds = 1.0 / clip.fps.max(0.001);
                while animation.timer >= frame_seconds {
                    animation.timer -= frame_seconds;
                    if animation.frame + 1 < clip.frames.len() {
                        animation.frame += 1;
                    } else if clip.looping {
                        animation.frame = 0;
                    }
                }
                clip.frames[animation.frame.min(clip.frames.len() - 1)]
            };

            if let Some(sprite) = self.component_mut::<Sprite>(id) {
                let (uv_min, uv_max) = sheet.frame_uv(frame_index);
                sprite.texture = sheet.texture;
                sprite.uv_min = uv_min;
                sprite.uv_max = uv_max;
            }
        }
    }
}

pub struct PlayerActor<'g, 'a, 'w> {
    game: &'g mut GameCtx<'a, 'w>,
}

impl<'g, 'a, 'w> PlayerActor<'g, 'a, 'w> {
    pub fn play_animation(&mut self, name: impl Into<String>) -> bool {
        let Some(id) = self.game.first_entity_with::<Player>() else {
            return false;
        };
        self.game.play_animation(id, name)
    }
}

#[cfg(test)]
mod tests {
    use game_core::backend::TextureHandle;
    use glam::vec2;

    use super::SpriteSheet;

    #[test]
    fn spritesheet_computes_frame_uvs() {
        let sheet = SpriteSheet::new(TextureHandle(7), 4, 2);

        assert_eq!(sheet.frame_uv(0), (vec2(0.0, 0.0), vec2(0.25, 0.5)));
        assert_eq!(sheet.frame_uv(5), (vec2(0.25, 0.5), vec2(0.5, 1.0)));
    }
}
