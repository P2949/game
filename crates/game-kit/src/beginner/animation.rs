//! Beginner spritesheet and animation helpers.

use std::collections::HashMap;

use game_core::backend::TextureHandle;
use game_core::world::{EntityId, Sprite};
use glam::Vec2;

use crate::context::GameCtx;

/// A completed non-looping animation, retained briefly so callbacks can observe
/// it without holding raw entity ids.
#[derive(Clone, Debug)]
pub(crate) struct AnimationFinishedRecord {
    pub(crate) sequence: u64,
    pub(crate) entity: EntityId,
    pub(crate) name: String,
}

/// Runtime queue used by [`crate::GameApp::on_animation_finished`].
#[derive(Default)]
pub(crate) struct AnimationFinishedEvents {
    next_sequence: u64,
    records: Vec<AnimationFinishedRecord>,
}

impl AnimationFinishedEvents {
    fn push(&mut self, entity: EntityId, name: String) {
        self.next_sequence += 1;
        self.records.push(AnimationFinishedRecord {
            sequence: self.next_sequence,
            entity,
            name,
        });
        // Callback systems run once per frame. This cap avoids an unbounded
        // resource if a game never registers one while preserving recent events.
        if self.records.len() > 256 {
            self.records.drain(..self.records.len() - 256);
        }
    }

    pub(crate) fn after(&self, sequence: u64) -> impl Iterator<Item = &AnimationFinishedRecord> {
        self.records
            .iter()
            .filter(move |record| record.sequence > sequence)
    }
}

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

/// A spritesheet plus named clips loaded from a small animation metadata file.
/// Pass it directly to a prefab's `.animation_sheet(...)` method to avoid
/// writing frame ranges in Rust.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationSheet {
    sheet: SpriteSheet,
    clips: Vec<(String, AnimationClip)>,
}

impl AnimationSheet {
    pub(crate) fn new(sheet: SpriteSheet, clips: Vec<(String, AnimationClip)>) -> Self {
        Self { sheet, clips }
    }

    /// The sheet portion, useful when a recipe wants to configure clips by hand.
    pub fn spritesheet(&self) -> SpriteSheet {
        self.sheet
    }

    pub(crate) fn into_parts(self) -> (SpriteSheet, Vec<(String, AnimationClip)>) {
        (self.sheet, self.clips)
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
        let mut finished = Vec::new();
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
                let frame_seconds = 1.0 / clip.fps.max(0.001);
                let was_finished = !clip.looping
                    && animation.frame + 1 >= clip.frames.len()
                    && animation.timer >= frame_seconds;
                animation.timer += dt;
                while animation.timer >= frame_seconds {
                    if animation.frame + 1 < clip.frames.len() {
                        animation.timer -= frame_seconds;
                        animation.frame += 1;
                    } else if clip.looping {
                        animation.timer -= frame_seconds;
                        animation.frame = 0;
                    } else {
                        if !was_finished {
                            finished.push((id, animation.current.clone()));
                        }
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

        if !finished.is_empty() {
            let events = self.resource_or_insert_with(AnimationFinishedEvents::default);
            for (entity, name) in finished {
                events.push(entity, name);
            }
        }
    }

    /// True when `name` is a completed one-shot clip on `id`.
    pub fn animation_finished(&self, id: EntityId, name: &str) -> bool {
        let Some(animation) = self.component::<Animation>(id) else {
            return false;
        };
        if animation.current != name {
            return false;
        }
        let Some(set) = self.component::<AnimationSet>(id) else {
            return false;
        };
        let Some(clip) = set.get(name) else {
            return false;
        };
        !clip.looping
            && !clip.frames.is_empty()
            && animation.frame + 1 >= clip.frames.len()
            && animation.timer >= 1.0 / clip.fps.max(0.001)
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
