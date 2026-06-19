//! Beginner spritesheet and animation helpers.

use std::collections::HashMap;

use game_core::backend::TextureHandle;
use game_core::world::{EntityId, Sprite};
use glam::Vec2;

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

pub fn frames(frames: impl IntoIterator<Item = usize>) -> AnimationClip {
    AnimationClip::frames(frames)
}

pub fn idle_frames(frames: impl IntoIterator<Item = usize>) -> AnimationClip {
    AnimationClip::frames(frames).fps(6.0).looping()
}

pub fn walk_frames(frames: impl IntoIterator<Item = usize>) -> AnimationClip {
    AnimationClip::frames(frames).fps(10.0).looping()
}

pub fn attack_frames(frames: impl IntoIterator<Item = usize>) -> AnimationClip {
    AnimationClip::frames(frames).fps(12.0).once()
}

pub fn die_frames(frames: impl IntoIterator<Item = usize>) -> AnimationClip {
    AnimationClip::frames(frames).fps(8.0).once()
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
                    if animation.frame + 1 < clip.frames.len() {
                        animation.timer -= frame_seconds;
                        animation.frame += 1;
                    } else if clip.looping {
                        animation.timer -= frame_seconds;
                        animation.frame = 0;
                    } else {
                        animation.timer = frame_seconds;
                        break;
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

#[cfg(test)]
mod tests {
    use game_core::backend::TextureHandle;
    use glam::vec2;

    use super::{SpriteSheet, attack_frames, die_frames, frames, idle_frames, walk_frames};

    #[test]
    fn spritesheet_computes_frame_uvs() {
        let sheet = SpriteSheet::new(TextureHandle(7), 4, 2);

        assert_eq!(sheet.frame_uv(0), (vec2(0.0, 0.0), vec2(0.25, 0.5)));
        assert_eq!(sheet.frame_uv(5), (vec2(0.25, 0.5), vec2(0.5, 1.0)));
    }

    #[test]
    fn clip_helpers_set_beginner_defaults() {
        assert_eq!(frames(0..2).fps, 8.0);

        let idle = idle_frames(0..2);
        assert_eq!(idle.fps, 6.0);
        assert!(idle.looping);

        let walk = walk_frames(2..4);
        assert_eq!(walk.fps, 10.0);
        assert!(walk.looping);

        let attack = attack_frames(4..6);
        assert_eq!(attack.fps, 12.0);
        assert!(!attack.looping);

        let die = die_frames(6..8);
        assert_eq!(die.fps, 8.0);
        assert!(!die.looping);
    }
}
