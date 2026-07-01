pub(super) use std::collections::{HashMap, HashSet};

pub(super) use anyhow::Result;
pub(super) use game_ai::{AiController, ChaseTarget, PathFollow, Patrol};
pub(super) use game_combat::{Faction, Health, MeleeAttack};
pub(super) use game_core::backend::TextureHandle;
pub(super) use game_core::builder::PropertyBag;
pub(super) use game_core::input::Axis2dId;
pub(super) use game_core::world::{NamedValues, Sprite, Tags, Transform, Velocity};
pub(super) use game_physics::{Collider, Trigger};
pub(super) use glam::{Vec2, Vec4, vec2};

pub(super) use crate::app::GameApp;
pub(super) use crate::assets::{IntoSoundRef, IntoTextureRef, SoundRef, TextureRef};
pub(super) use crate::beginner::actors::{
    Area, AreaName, Checkpoint, CollectSound, Collectible, DeathAnimationPolicy, DespawnOnCollect,
    DespawnOnHit, Door, DoorAction, DoorTarget, DropsPrefab, Enemy, ExitDoor, FacingDirection,
    HealValue, Lifetime, Name, Pickup, Player, PlayerMovement, PlayerProjectile, Projectile,
    ProjectileDamage, ScoreValue, SpawnPlacement, Spawner, Speed, TriggerArea,
};
pub(super) use crate::beginner::animation::{
    Animation, AnimationClip, AnimationSet, AnimationSheet, SpriteSheet, attack_frames, die_frames,
    idle_frames, walk_frames,
};
pub(super) use crate::beginner::tuning::{TunedF32, TunedI32};
pub(super) use crate::bundle::vec2s;
pub(super) use crate::prefab::{IntoContentName, IntoMovementAxis};

pub(super) const PLAYER_SIZE: f32 = 20.0;
pub(super) const ENEMY_SIZE: f32 = 22.0;
pub(super) const PICKUP_SIZE: f32 = 16.0;
pub(super) const DOOR_SIZE: f32 = 24.0;
pub(super) const PROJECTILE_SIZE: f32 = 10.0;
pub(super) const AREA_SIZE: f32 = 32.0;
pub(super) const PLAYER_HEALTH: i32 = 100;
pub(super) const ENEMY_HEALTH: i32 = 40;
pub(super) const PLAYER_SPEED: f32 = 130.0;
pub(super) const ENEMY_SPEED: f32 = 80.0;
pub(super) const PROJECTILE_SPEED: f32 = 300.0;
pub(super) const DEFAULT_LAYER: i16 = 10;
pub(super) const ENEMY_CHASE_RANGE: f32 = 180.0;
pub(super) const ENEMY_REPATH_SECONDS: f32 = 0.25;
pub(super) const ENEMY_MELEE_COOLDOWN: f32 = 0.75;
pub(super) const PROJECTILE_DIRECTION_X: &str = "beginner/projectile_direction_x";
pub(super) const PROJECTILE_DIRECTION_Y: &str = "beginner/projectile_direction_y";

pub(super) fn projectile_direction(properties: &PropertyBag) -> Vec2 {
    let x = properties
        .get(PROJECTILE_DIRECTION_X)
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or_default();
    let y = properties
        .get(PROJECTILE_DIRECTION_Y)
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or_default();
    vec2(x, y).normalize_or_zero()
}

#[derive(Clone)]
pub(super) struct MeleeSpec {
    pub(super) range: f32,
    pub(super) damage: TunedI32,
    pub(super) cooldown: f32,
}

#[derive(Clone, Copy)]
pub(super) struct ChaseSpec {
    pub(super) range: f32,
    pub(super) stop_distance: Option<f32>,
    pub(super) repath_seconds: f32,
}

#[derive(Clone, Copy)]
pub(super) enum PatrolSpec {
    Between(Vec2, Vec2),
    Horizontal { half_distance: f32 },
}

#[derive(Clone, Copy)]
pub(super) enum SpriteSource {
    Texture(TextureHandle),
    Sheet(SpriteSheet),
}

#[derive(Clone)]
pub(super) enum SpriteSourceRef {
    Texture(TextureRef),
    Sheet(SpriteSheet),
}

#[derive(Clone)]
pub(super) struct ActorPrefabSpec {
    pub(super) display_name: Option<String>,
    pub(super) sprite: Option<SpriteSourceRef>,
    pub(super) size: Vec2,
    pub(super) tint: Vec4,
    pub(super) layer: i16,
    pub(super) health: TunedI32,
    pub(super) speed: TunedF32,
    pub(super) melee: Option<MeleeSpec>,
    pub(super) collider: Option<Vec2>,
    pub(super) animations: Vec<(String, AnimationClip)>,
    pub(super) play_animation: Option<String>,
    pub(super) tags: HashSet<String>,
    pub(super) named_values: HashMap<String, f32>,
}

impl ActorPrefabSpec {
    pub(super) fn player() -> Self {
        Self {
            display_name: None,
            sprite: None,
            size: vec2s(PLAYER_SIZE),
            tint: Vec4::ONE,
            layer: DEFAULT_LAYER,
            health: PLAYER_HEALTH.into(),
            speed: PLAYER_SPEED.into(),
            melee: None,
            collider: None,
            animations: Vec::new(),
            play_animation: None,
            tags: HashSet::new(),
            named_values: HashMap::new(),
        }
    }

    pub(super) fn enemy() -> Self {
        Self {
            display_name: None,
            sprite: None,
            size: vec2s(ENEMY_SIZE),
            tint: Vec4::ONE,
            layer: DEFAULT_LAYER,
            health: ENEMY_HEALTH.into(),
            speed: ENEMY_SPEED.into(),
            melee: None,
            collider: None,
            animations: Vec::new(),
            play_animation: None,
            tags: HashSet::new(),
            named_values: HashMap::new(),
        }
    }

    pub(super) fn sprite_source(
        &self,
        app: &GameApp<'_>,
        kind: &str,
        prefab_name: &str,
    ) -> Result<SpriteSource> {
        let label = actor_kind_label(kind);
        let source = self.sprite.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "{label} prefab '{prefab_name}' has no sprite.\n\nAdd:\n    .sprite(\"{kind}\")\n\nor:\n    .spritesheet(assets.{kind}_sheet)\n\nExample:\n    game.{kind}_prefab(\"{prefab_name}\")\n        .sprite(\"{kind}\")\n        .build()?;"
            )
        })?;
        match source {
            SpriteSourceRef::Texture(texture) => Ok(SpriteSource::Texture(
                app.resolve_texture_ref(texture.clone())?,
            )),
            SpriteSourceRef::Sheet(sheet) => Ok(SpriteSource::Sheet(*sheet)),
        }
    }

    pub(super) fn sprite(&self, source: SpriteSource, frame: usize) -> Sprite {
        let sprite = match source {
            SpriteSource::Texture(texture) => Sprite::new(texture, self.size),
            SpriteSource::Sheet(sheet) => sheet.sprite(frame, self.size),
        };
        sprite.layer(self.layer).tint(self.tint)
    }

    pub(super) fn animation_components(
        &self,
        source: SpriteSource,
        kind: &str,
        prefab_name: &str,
    ) -> Result<Option<(Animation, AnimationSet)>> {
        if self.animations.is_empty() {
            if let Some(name) = &self.play_animation {
                anyhow::bail!(
                    "{kind} prefab '{prefab_name}' asks to play animation '{name}', but no animations were registered.\n\nAdd:\n    .animation(\"{name}\", AnimationClip::frames(0..4))"
                );
            }
            return Ok(None);
        }

        let SpriteSource::Sheet(sheet) = source else {
            anyhow::bail!(
                "{kind} prefab '{prefab_name}' has animations but uses a static sprite texture.\n\nUse:\n    .spritesheet(assets.{kind}_sheet)"
            );
        };

        let initial = self
            .play_animation
            .clone()
            .unwrap_or_else(|| self.animations[0].0.clone());
        let mut set = AnimationSet::new(sheet);
        for (name, clip) in &self.animations {
            set = set.animation(name.clone(), clip.clone());
        }
        if set.get(&initial).is_none() {
            anyhow::bail!(
                "{kind} prefab '{prefab_name}' asks to play animation '{initial}', but that clip was not registered."
            );
        }

        Ok(Some((Animation::play(initial), set)))
    }

    pub(super) fn display_name(&self, fallback: &str) -> String {
        self.display_name
            .clone()
            .unwrap_or_else(|| fallback.to_owned())
    }

    pub(super) fn collider(&self) -> Vec2 {
        self.collider.unwrap_or(self.size)
    }
}

pub(super) fn actor_kind_label(kind: &str) -> &'static str {
    match kind {
        "player" => "Player",
        "enemy" => "Enemy",
        "pickup" => "Pickup",
        "door" => "Door",
        "projectile" => "Projectile",
        _ => "Actor",
    }
}

#[derive(Clone)]
pub(super) struct ObjectPrefabSpec {
    pub(super) display_name: Option<String>,
    pub(super) sprite: Option<SpriteSourceRef>,
    pub(super) size: Vec2,
    pub(super) tint: Vec4,
    pub(super) layer: i16,
    pub(super) collider: Option<Vec2>,
    pub(super) tags: HashSet<String>,
    pub(super) named_values: HashMap<String, f32>,
}

impl ObjectPrefabSpec {
    pub(super) fn new(size: f32) -> Self {
        Self {
            display_name: None,
            sprite: None,
            size: vec2s(size),
            tint: Vec4::ONE,
            layer: DEFAULT_LAYER,
            collider: None,
            tags: HashSet::new(),
            named_values: HashMap::new(),
        }
    }

    pub(super) fn sprite_source(
        &self,
        app: &GameApp<'_>,
        kind: &str,
        prefab_name: &str,
    ) -> Result<SpriteSource> {
        let label = actor_kind_label(kind);
        let source = self.sprite.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "{label} prefab '{prefab_name}' has no sprite.\n\nAdd:\n    .sprite(\"{kind}\")"
            )
        })?;
        match source {
            SpriteSourceRef::Texture(texture) => Ok(SpriteSource::Texture(
                app.resolve_texture_ref(texture.clone())?,
            )),
            SpriteSourceRef::Sheet(sheet) => Ok(SpriteSource::Sheet(*sheet)),
        }
    }

    pub(super) fn sprite(&self, source: SpriteSource) -> Sprite {
        self.sprite_at(source, 0)
    }

    pub(super) fn sprite_at(&self, source: SpriteSource, frame: usize) -> Sprite {
        let sprite = match source {
            SpriteSource::Texture(texture) => Sprite::new(texture, self.size),
            SpriteSource::Sheet(sheet) => sheet.sprite(frame, self.size),
        };
        sprite.layer(self.layer).tint(self.tint)
    }

    pub(super) fn display_name(&self, fallback: &str) -> String {
        self.display_name
            .clone()
            .unwrap_or_else(|| fallback.to_owned())
    }

    pub(super) fn collider(&self) -> Vec2 {
        self.collider.unwrap_or(self.size)
    }
}
