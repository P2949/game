//! Beginner prefab builders.

use anyhow::Result;
use game_ai::{AiController, ChaseTarget, PathFollow, Patrol};
use game_combat::{Faction, Health, MeleeAttack};
use game_core::backend::TextureHandle;
use game_core::builder::PropertyBag;
use game_core::input::Axis2dId;
use game_core::world::{Sprite, Transform, Velocity};
use game_physics::Collider;
use glam::{Vec2, Vec4, vec2};

use crate::app::GameApp;
use crate::assets::{SoundRef, TextureRef};
use crate::beginner::actors::{
    CollectSound, Collectible, DeathAnimationPolicy, DespawnOnCollect, DespawnOnHit, Door,
    DoorAction, DoorTarget, Enemy, ExitDoor, Lifetime, Name, Pickup, Player, PlayerMovement,
    PlayerProjectile, Projectile, ProjectileDamage, ScoreValue, SpawnPlacement, Spawner, Speed,
};
use crate::beginner::animation::{
    Animation, AnimationClip, AnimationSet, SpriteSheet, attack_frames, die_frames, idle_frames,
    walk_frames,
};
use crate::bundle::vec2s;

const PLAYER_SIZE: f32 = 20.0;
const ENEMY_SIZE: f32 = 22.0;
const PICKUP_SIZE: f32 = 16.0;
const DOOR_SIZE: f32 = 24.0;
const PROJECTILE_SIZE: f32 = 10.0;
const PLAYER_HEALTH: i32 = 100;
const ENEMY_HEALTH: i32 = 40;
const PLAYER_SPEED: f32 = 130.0;
const ENEMY_SPEED: f32 = 80.0;
const PROJECTILE_SPEED: f32 = 300.0;
const DEFAULT_LAYER: i16 = 10;
const ENEMY_CHASE_RANGE: f32 = 180.0;
const ENEMY_REPATH_SECONDS: f32 = 0.25;
const ENEMY_MELEE_COOLDOWN: f32 = 0.75;
const PROJECTILE_DIRECTION_X: &str = "beginner/projectile_direction_x";
const PROJECTILE_DIRECTION_Y: &str = "beginner/projectile_direction_y";

fn projectile_direction(properties: &PropertyBag) -> Vec2 {
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

#[derive(Clone, Copy)]
struct MeleeSpec {
    range: f32,
    damage: i32,
    cooldown: f32,
}

#[derive(Clone, Copy)]
enum SpriteSource {
    Texture(TextureHandle),
    Sheet(SpriteSheet),
}

#[derive(Clone)]
enum SpriteSourceRef {
    Texture(TextureRef),
    Sheet(SpriteSheet),
}

#[derive(Clone)]
struct ActorPrefabSpec {
    display_name: Option<String>,
    sprite: Option<SpriteSourceRef>,
    size: Vec2,
    tint: Vec4,
    layer: i16,
    health: i32,
    speed: f32,
    melee: Option<MeleeSpec>,
    collider: Option<Vec2>,
    animations: Vec<(String, AnimationClip)>,
    play_animation: Option<String>,
}

impl ActorPrefabSpec {
    fn player() -> Self {
        Self {
            display_name: None,
            sprite: None,
            size: vec2s(PLAYER_SIZE),
            tint: Vec4::ONE,
            layer: DEFAULT_LAYER,
            health: PLAYER_HEALTH,
            speed: PLAYER_SPEED,
            melee: None,
            collider: None,
            animations: Vec::new(),
            play_animation: None,
        }
    }

    fn enemy() -> Self {
        Self {
            display_name: None,
            sprite: None,
            size: vec2s(ENEMY_SIZE),
            tint: Vec4::ONE,
            layer: DEFAULT_LAYER,
            health: ENEMY_HEALTH,
            speed: ENEMY_SPEED,
            melee: None,
            collider: None,
            animations: Vec::new(),
            play_animation: None,
        }
    }

    fn sprite_source(
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

    fn sprite(&self, source: SpriteSource, frame: usize) -> Sprite {
        let sprite = match source {
            SpriteSource::Texture(texture) => Sprite::new(texture, self.size),
            SpriteSource::Sheet(sheet) => sheet.sprite(frame, self.size),
        };
        sprite.layer(self.layer).tint(self.tint)
    }

    fn animation_components(
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

    fn display_name(&self, fallback: &str) -> String {
        self.display_name
            .clone()
            .unwrap_or_else(|| fallback.to_owned())
    }

    fn collider(&self) -> Vec2 {
        self.collider.unwrap_or(self.size)
    }
}

fn actor_kind_label(kind: &str) -> &'static str {
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
struct ObjectPrefabSpec {
    display_name: Option<String>,
    sprite: Option<SpriteSourceRef>,
    size: Vec2,
    tint: Vec4,
    layer: i16,
    collider: Option<Vec2>,
}

impl ObjectPrefabSpec {
    fn new(size: f32) -> Self {
        Self {
            display_name: None,
            sprite: None,
            size: vec2s(size),
            tint: Vec4::ONE,
            layer: DEFAULT_LAYER,
            collider: None,
        }
    }

    fn sprite_source(
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

    fn sprite(&self, source: SpriteSource) -> Sprite {
        let sprite = match source {
            SpriteSource::Texture(texture) => Sprite::new(texture, self.size),
            SpriteSource::Sheet(sheet) => sheet.sprite(0, self.size),
        };
        sprite.layer(self.layer).tint(self.tint)
    }

    fn display_name(&self, fallback: &str) -> String {
        self.display_name
            .clone()
            .unwrap_or_else(|| fallback.to_owned())
    }

    fn collider(&self) -> Vec2 {
        self.collider.unwrap_or(self.size)
    }
}

pub struct PlayerPrefabAuthor<'a, 'app> {
    app: &'a mut GameApp<'app>,
    name: String,
    spec: ActorPrefabSpec,
    movement_axis: Option<Axis2dId>,
}

impl<'a, 'app> PlayerPrefabAuthor<'a, 'app> {
    pub(crate) fn new(app: &'a mut GameApp<'app>, name: String) -> Self {
        Self {
            app,
            name,
            spec: ActorPrefabSpec::player(),
            movement_axis: None,
        }
    }

    pub fn named(mut self, display_name: impl Into<String>) -> Self {
        self.spec.display_name = Some(display_name.into());
        self
    }

    pub fn sprite(mut self, texture: impl Into<TextureRef>) -> Self {
        self.spec.sprite = Some(SpriteSourceRef::Texture(texture.into()));
        self
    }

    pub fn spritesheet(mut self, sheet: SpriteSheet) -> Self {
        self.spec.sprite = Some(SpriteSourceRef::Sheet(sheet));
        self
    }

    pub fn animation(mut self, name: impl Into<String>, clip: AnimationClip) -> Self {
        self.spec.animations.push((name.into(), clip));
        self
    }

    pub fn idle(self, frames: impl IntoIterator<Item = usize>) -> Self {
        self.animation("idle", idle_frames(frames))
    }

    pub fn walk(self, frames: impl IntoIterator<Item = usize>) -> Self {
        self.animation("walk", walk_frames(frames))
    }

    pub fn walk_up(self, frames: impl IntoIterator<Item = usize>) -> Self {
        self.animation("walk_up", walk_frames(frames))
    }

    pub fn walk_down(self, frames: impl IntoIterator<Item = usize>) -> Self {
        self.animation("walk_down", walk_frames(frames))
    }

    pub fn walk_left(self, frames: impl IntoIterator<Item = usize>) -> Self {
        self.animation("walk_left", walk_frames(frames))
    }

    pub fn walk_right(self, frames: impl IntoIterator<Item = usize>) -> Self {
        self.animation("walk_right", walk_frames(frames))
    }

    pub fn attack(self, frames: impl IntoIterator<Item = usize>) -> Self {
        self.animation("attack", attack_frames(frames))
    }

    pub fn die(self, frames: impl IntoIterator<Item = usize>) -> Self {
        self.animation("die", die_frames(frames))
    }

    pub fn play(mut self, name: impl Into<String>) -> Self {
        self.spec.play_animation = Some(name.into());
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.spec.size = vec2s(size);
        self
    }

    pub fn size2(mut self, size: Vec2) -> Self {
        self.spec.size = size;
        self
    }

    pub fn collider(mut self, size: Vec2) -> Self {
        self.spec.collider = Some(size);
        self
    }

    pub fn tint(mut self, tint: Vec4) -> Self {
        self.spec.tint = tint;
        self
    }

    pub fn layer(mut self, layer: i16) -> Self {
        self.spec.layer = layer;
        self
    }

    pub fn health(mut self, health: i32) -> Self {
        self.spec.health = health;
        self
    }

    pub fn speed(mut self, speed: f32) -> Self {
        self.spec.speed = speed;
        self
    }

    pub fn moves_with(mut self, axis: Axis2dId, speed: f32) -> Self {
        self.movement_axis = Some(axis);
        self.spec.speed = speed;
        self
    }

    pub fn melee(mut self, range: f32, damage: i32) -> Self {
        self.spec.melee = Some(MeleeSpec {
            range,
            damage,
            cooldown: 0.0,
        });
        self
    }

    pub fn build(self) -> Result<()> {
        let sprite_source = self.spec.sprite_source(self.app, "player", &self.name)?;
        let movement_axis = self.movement_axis.ok_or_else(|| {
            anyhow::anyhow!(
                "player prefab '{}' has no movement axis.\n\nAdd:\n    .moves_with(controls.movement, 130.0)",
                self.name
            )
        })?;
        let spec = self.spec;
        let prefab_name = self.name.clone();
        let display_name = spec.display_name(&prefab_name);
        let collider = spec.collider();
        let melee = spec.melee.unwrap_or(MeleeSpec {
            range: 30.0,
            damage: 0,
            cooldown: 0.0,
        });
        let animation_components =
            spec.animation_components(sprite_source, "player", &prefab_name)?;

        if let Some((animation, animation_set)) = animation_components {
            self.app.prefab(self.name, move |prefab| {
                let first_frame = animation_set
                    .get(&animation.current)
                    .and_then(|clip| clip.frames.first())
                    .copied()
                    .unwrap_or(0);
                prefab
                    .spawn(move |at| {
                        (
                            Name::new(display_name.clone()),
                            Transform::at(at),
                            Velocity::default(),
                            Player,
                            PlayerMovement::axis(movement_axis),
                            Speed::new(spec.speed),
                            Health::new(spec.health),
                            MeleeAttack::new(melee.range, melee.damage).cooldown(melee.cooldown),
                            Faction::player(),
                            spec.sprite(sprite_source, first_frame),
                            animation.clone(),
                            animation_set.clone(),
                            Collider::box_of(collider),
                        )
                    })?
                    .require::<Transform>()
                    .require::<Velocity>()
                    .require::<Player>()
                    .require::<PlayerMovement>()
                    .require::<Speed>()
                    .require::<Sprite>()
                    .require::<Animation>()
                    .require::<AnimationSet>()
                    .require::<Collider>()
                    .require::<Health>()
                    .require::<MeleeAttack>()
                    .require::<Faction>();
                Ok(())
            })
        } else {
            self.app.prefab(self.name, move |prefab| {
                prefab
                    .spawn(move |at| {
                        (
                            Name::new(display_name.clone()),
                            Transform::at(at),
                            Velocity::default(),
                            Player,
                            PlayerMovement::axis(movement_axis),
                            Speed::new(spec.speed),
                            Health::new(spec.health),
                            MeleeAttack::new(melee.range, melee.damage).cooldown(melee.cooldown),
                            Faction::player(),
                            spec.sprite(sprite_source, 0),
                            Collider::box_of(collider),
                        )
                    })?
                    .require::<Transform>()
                    .require::<Velocity>()
                    .require::<Player>()
                    .require::<PlayerMovement>()
                    .require::<Speed>()
                    .require::<Sprite>()
                    .require::<Collider>()
                    .require::<Health>()
                    .require::<MeleeAttack>()
                    .require::<Faction>();
                Ok(())
            })
        }
    }
}

#[derive(Clone, Copy)]
struct ChaseSpec {
    range: f32,
    stop_distance: Option<f32>,
    repath_seconds: f32,
}

#[derive(Clone, Copy)]
enum PatrolSpec {
    Between(Vec2, Vec2),
    Horizontal { half_distance: f32 },
}

pub struct EnemyPrefabAuthor<'a, 'app> {
    app: &'a mut GameApp<'app>,
    name: String,
    spec: ActorPrefabSpec,
    chase: Option<ChaseSpec>,
    patrol: Option<PatrolSpec>,
    patrol_speed: Option<f32>,
    despawn_after_death_animation: bool,
}

impl<'a, 'app> EnemyPrefabAuthor<'a, 'app> {
    pub(crate) fn new(app: &'a mut GameApp<'app>, name: String) -> Self {
        Self {
            app,
            name,
            spec: ActorPrefabSpec::enemy(),
            chase: None,
            patrol: None,
            patrol_speed: None,
            despawn_after_death_animation: false,
        }
    }

    pub fn named(mut self, display_name: impl Into<String>) -> Self {
        self.spec.display_name = Some(display_name.into());
        self
    }

    pub fn sprite(mut self, texture: impl Into<TextureRef>) -> Self {
        self.spec.sprite = Some(SpriteSourceRef::Texture(texture.into()));
        self
    }

    pub fn spritesheet(mut self, sheet: SpriteSheet) -> Self {
        self.spec.sprite = Some(SpriteSourceRef::Sheet(sheet));
        self
    }

    pub fn animation(mut self, name: impl Into<String>, clip: AnimationClip) -> Self {
        self.spec.animations.push((name.into(), clip));
        self
    }

    pub fn idle(self, frames: impl IntoIterator<Item = usize>) -> Self {
        self.animation("idle", idle_frames(frames))
    }

    pub fn walk(self, frames: impl IntoIterator<Item = usize>) -> Self {
        self.animation("walk", walk_frames(frames))
    }

    pub fn attack(self, frames: impl IntoIterator<Item = usize>) -> Self {
        self.animation("attack", attack_frames(frames))
    }

    pub fn die(self, frames: impl IntoIterator<Item = usize>) -> Self {
        self.animation("die", die_frames(frames))
    }

    /// Keeps this enemy alive until its configured `die` animation finishes
    /// when `dead_enemies_play_death_animation()` and
    /// `dead_enemies_despawn_after_animation()` are enabled.
    pub fn despawn_after_death_animation(mut self) -> Self {
        self.despawn_after_death_animation = true;
        self
    }

    pub fn play(mut self, name: impl Into<String>) -> Self {
        self.spec.play_animation = Some(name.into());
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.spec.size = vec2s(size);
        self
    }

    pub fn size2(mut self, size: Vec2) -> Self {
        self.spec.size = size;
        self
    }

    pub fn collider(mut self, size: Vec2) -> Self {
        self.spec.collider = Some(size);
        self
    }

    pub fn tint(mut self, tint: Vec4) -> Self {
        self.spec.tint = tint;
        self
    }

    pub fn layer(mut self, layer: i16) -> Self {
        self.spec.layer = layer;
        self
    }

    pub fn health(mut self, health: i32) -> Self {
        self.spec.health = health;
        self
    }

    pub fn speed(mut self, speed: f32) -> Self {
        self.spec.speed = speed;
        self
    }

    pub fn melee(mut self, range: f32, damage: i32) -> Self {
        self.spec.melee = Some(MeleeSpec {
            range,
            damage,
            cooldown: ENEMY_MELEE_COOLDOWN,
        });
        self
    }

    pub fn melee_cooldown(mut self, cooldown: f32) -> Self {
        let melee = self.spec.melee.get_or_insert(MeleeSpec {
            range: 26.0,
            damage: 0,
            cooldown: ENEMY_MELEE_COOLDOWN,
        });
        melee.cooldown = cooldown;
        self
    }

    pub fn chases_player(mut self) -> Self {
        self.chase = Some(ChaseSpec {
            range: ENEMY_CHASE_RANGE,
            stop_distance: None,
            repath_seconds: ENEMY_REPATH_SECONDS,
        });
        self
    }

    pub fn chase_range(mut self, range: f32) -> Self {
        let chase = self.chase.get_or_insert(ChaseSpec {
            range: ENEMY_CHASE_RANGE,
            stop_distance: None,
            repath_seconds: ENEMY_REPATH_SECONDS,
        });
        chase.range = range;
        self
    }

    pub fn stop_distance(mut self, distance: f32) -> Self {
        let chase = self.chase.get_or_insert(ChaseSpec {
            range: ENEMY_CHASE_RANGE,
            stop_distance: None,
            repath_seconds: ENEMY_REPATH_SECONDS,
        });
        chase.stop_distance = Some(distance);
        self
    }

    pub fn repath_seconds(mut self, seconds: f32) -> Self {
        let chase = self.chase.get_or_insert(ChaseSpec {
            range: ENEMY_CHASE_RANGE,
            stop_distance: None,
            repath_seconds: ENEMY_REPATH_SECONDS,
        });
        chase.repath_seconds = seconds;
        self
    }

    pub fn patrol_between(mut self, a: Vec2, b: Vec2) -> Self {
        self.patrol = Some(PatrolSpec::Between(a, b));
        self
    }

    pub fn patrol_horizontal(mut self, distance: f32) -> Self {
        self.patrol = Some(PatrolSpec::Horizontal {
            half_distance: distance.abs() * 0.5,
        });
        self
    }

    pub fn patrol_speed(mut self, speed: f32) -> Self {
        self.patrol_speed = Some(speed);
        self
    }

    pub fn build(self) -> Result<()> {
        let sprite_source = self.spec.sprite_source(self.app, "enemy", &self.name)?;
        let spec = self.spec;
        let prefab_name = self.name.clone();
        let display_name = spec.display_name(&prefab_name);
        let collider = spec.collider();
        let melee = spec.melee.unwrap_or(MeleeSpec {
            range: 26.0,
            damage: 0,
            cooldown: ENEMY_MELEE_COOLDOWN,
        });
        let animation_components =
            spec.animation_components(sprite_source, "enemy", &prefab_name)?;
        let patrol = self.patrol;
        let patrol_speed = self.patrol_speed.unwrap_or(spec.speed);
        let death_animation_policy = DeathAnimationPolicy {
            despawn_after_animation: self.despawn_after_death_animation,
        };

        if let Some(chase) = self.chase {
            if patrol.is_some() {
                anyhow::bail!(
                    "enemy prefab '{}' cannot both chase the player and patrol.\n\nUse either .chases_player() or .patrol_between(...).",
                    prefab_name
                );
            }
            let stop_distance = chase.stop_distance.unwrap_or(melee.range * 0.8);
            if let Some((animation, animation_set)) = animation_components {
                self.app.prefab(self.name, move |prefab| {
                    let first_frame = animation_set
                        .get(&animation.current)
                        .and_then(|clip| clip.frames.first())
                        .copied()
                        .unwrap_or(0);
                    prefab
                        .spawn(move |at| {
                            (
                                Name::new(display_name.clone()),
                                Transform::at(at),
                                Velocity::default(),
                                Enemy,
                                death_animation_policy,
                                Speed::new(spec.speed),
                                Health::new(spec.health),
                                Faction::enemy(),
                                MeleeAttack::new(melee.range, melee.damage)
                                    .cooldown(melee.cooldown),
                                AiController::chase_player(),
                                ChaseTarget::player(
                                    chase.range,
                                    stop_distance,
                                    spec.speed,
                                    chase.repath_seconds,
                                ),
                                PathFollow::default(),
                                spec.sprite(sprite_source, first_frame),
                                animation.clone(),
                                animation_set.clone(),
                                Collider::box_of(collider),
                            )
                        })?
                        .require::<Transform>()
                        .require::<Velocity>()
                        .require::<Enemy>()
                        .require::<Speed>()
                        .require::<Sprite>()
                        .require::<Animation>()
                        .require::<AnimationSet>()
                        .require::<Collider>()
                        .require::<Health>()
                        .require::<MeleeAttack>()
                        .require::<Faction>()
                        .require::<AiController>();
                    Ok(())
                })
            } else {
                self.app.prefab(self.name, move |prefab| {
                    prefab
                        .spawn(move |at| {
                            (
                                Name::new(display_name.clone()),
                                Transform::at(at),
                                Velocity::default(),
                                Enemy,
                                death_animation_policy,
                                Speed::new(spec.speed),
                                Health::new(spec.health),
                                Faction::enemy(),
                                MeleeAttack::new(melee.range, melee.damage)
                                    .cooldown(melee.cooldown),
                                AiController::chase_player(),
                                ChaseTarget::player(
                                    chase.range,
                                    stop_distance,
                                    spec.speed,
                                    chase.repath_seconds,
                                ),
                                PathFollow::default(),
                                spec.sprite(sprite_source, 0),
                                Collider::box_of(collider),
                            )
                        })?
                        .require::<Transform>()
                        .require::<Velocity>()
                        .require::<Enemy>()
                        .require::<Speed>()
                        .require::<Sprite>()
                        .require::<Collider>()
                        .require::<Health>()
                        .require::<MeleeAttack>()
                        .require::<Faction>()
                        .require::<AiController>();
                    Ok(())
                })
            }
        } else if let Some(patrol) = patrol {
            if let Some((animation, animation_set)) = animation_components {
                self.app.prefab(self.name, move |prefab| {
                    let first_frame = animation_set
                        .get(&animation.current)
                        .and_then(|clip| clip.frames.first())
                        .copied()
                        .unwrap_or(0);
                    prefab
                        .spawn(move |at| {
                            let waypoints = match patrol {
                                PatrolSpec::Between(a, b) => vec![a, b],
                                PatrolSpec::Horizontal { half_distance } => {
                                    vec![
                                        at - vec2(half_distance, 0.0),
                                        at + vec2(half_distance, 0.0),
                                    ]
                                }
                            };
                            (
                                Name::new(display_name.clone()),
                                Transform::at(at),
                                Velocity::default(),
                                Enemy,
                                death_animation_policy,
                                Speed::new(spec.speed),
                                Health::new(spec.health),
                                Faction::enemy(),
                                MeleeAttack::new(melee.range, melee.damage)
                                    .cooldown(melee.cooldown),
                                Patrol::new(waypoints, patrol_speed),
                                spec.sprite(sprite_source, first_frame),
                                animation.clone(),
                                animation_set.clone(),
                                Collider::box_of(collider),
                            )
                        })?
                        .require::<Transform>()
                        .require::<Velocity>()
                        .require::<Enemy>()
                        .require::<Speed>()
                        .require::<Sprite>()
                        .require::<Animation>()
                        .require::<AnimationSet>()
                        .require::<Collider>()
                        .require::<Health>()
                        .require::<MeleeAttack>()
                        .require::<Faction>()
                        .require::<Patrol>();
                    Ok(())
                })
            } else {
                self.app.prefab(self.name, move |prefab| {
                    prefab
                        .spawn(move |at| {
                            let waypoints = match patrol {
                                PatrolSpec::Between(a, b) => vec![a, b],
                                PatrolSpec::Horizontal { half_distance } => {
                                    vec![
                                        at - vec2(half_distance, 0.0),
                                        at + vec2(half_distance, 0.0),
                                    ]
                                }
                            };
                            (
                                Name::new(display_name.clone()),
                                Transform::at(at),
                                Velocity::default(),
                                Enemy,
                                death_animation_policy,
                                Speed::new(spec.speed),
                                Health::new(spec.health),
                                Faction::enemy(),
                                MeleeAttack::new(melee.range, melee.damage)
                                    .cooldown(melee.cooldown),
                                Patrol::new(waypoints, patrol_speed),
                                spec.sprite(sprite_source, 0),
                                Collider::box_of(collider),
                            )
                        })?
                        .require::<Transform>()
                        .require::<Velocity>()
                        .require::<Enemy>()
                        .require::<Speed>()
                        .require::<Sprite>()
                        .require::<Collider>()
                        .require::<Health>()
                        .require::<MeleeAttack>()
                        .require::<Faction>()
                        .require::<Patrol>();
                    Ok(())
                })
            }
        } else {
            if let Some((animation, animation_set)) = animation_components {
                self.app.prefab(self.name, move |prefab| {
                    let first_frame = animation_set
                        .get(&animation.current)
                        .and_then(|clip| clip.frames.first())
                        .copied()
                        .unwrap_or(0);
                    prefab
                        .spawn(move |at| {
                            (
                                Name::new(display_name.clone()),
                                Transform::at(at),
                                Velocity::default(),
                                Enemy,
                                death_animation_policy,
                                Speed::new(spec.speed),
                                Health::new(spec.health),
                                Faction::enemy(),
                                MeleeAttack::new(melee.range, melee.damage)
                                    .cooldown(melee.cooldown),
                                spec.sprite(sprite_source, first_frame),
                                animation.clone(),
                                animation_set.clone(),
                                Collider::box_of(collider),
                            )
                        })?
                        .require::<Transform>()
                        .require::<Velocity>()
                        .require::<Enemy>()
                        .require::<Speed>()
                        .require::<Sprite>()
                        .require::<Animation>()
                        .require::<AnimationSet>()
                        .require::<Collider>()
                        .require::<Health>()
                        .require::<MeleeAttack>()
                        .require::<Faction>();
                    Ok(())
                })
            } else {
                self.app.prefab(self.name, move |prefab| {
                    prefab
                        .spawn(move |at| {
                            (
                                Name::new(display_name.clone()),
                                Transform::at(at),
                                Velocity::default(),
                                Enemy,
                                death_animation_policy,
                                Speed::new(spec.speed),
                                Health::new(spec.health),
                                Faction::enemy(),
                                MeleeAttack::new(melee.range, melee.damage)
                                    .cooldown(melee.cooldown),
                                spec.sprite(sprite_source, 0),
                                Collider::box_of(collider),
                            )
                        })?
                        .require::<Transform>()
                        .require::<Velocity>()
                        .require::<Enemy>()
                        .require::<Speed>()
                        .require::<Sprite>()
                        .require::<Collider>()
                        .require::<Health>()
                        .require::<MeleeAttack>()
                        .require::<Faction>();
                    Ok(())
                })
            }
        }
    }
}

pub struct PickupPrefabAuthor<'a, 'app> {
    app: &'a mut GameApp<'app>,
    name: String,
    spec: ObjectPrefabSpec,
    score: i32,
    sound: Option<SoundRef>,
    despawn_on_collect: bool,
}

impl<'a, 'app> PickupPrefabAuthor<'a, 'app> {
    pub(crate) fn new(app: &'a mut GameApp<'app>, name: String) -> Self {
        Self {
            app,
            name,
            spec: ObjectPrefabSpec::new(PICKUP_SIZE),
            score: 0,
            sound: None,
            despawn_on_collect: false,
        }
    }

    pub fn named(mut self, display_name: impl Into<String>) -> Self {
        self.spec.display_name = Some(display_name.into());
        self
    }

    pub fn sprite(mut self, texture: impl Into<TextureRef>) -> Self {
        self.spec.sprite = Some(SpriteSourceRef::Texture(texture.into()));
        self
    }

    pub fn spritesheet(mut self, sheet: SpriteSheet) -> Self {
        self.spec.sprite = Some(SpriteSourceRef::Sheet(sheet));
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.spec.size = vec2s(size);
        self
    }

    pub fn size2(mut self, size: Vec2) -> Self {
        self.spec.size = size;
        self
    }

    pub fn collider(mut self, size: Vec2) -> Self {
        self.spec.collider = Some(size);
        self
    }

    pub fn tint(mut self, tint: Vec4) -> Self {
        self.spec.tint = tint;
        self
    }

    pub fn layer(mut self, layer: i16) -> Self {
        self.spec.layer = layer;
        self
    }

    pub fn score(mut self, value: i32) -> Self {
        self.score = value;
        self
    }

    pub fn play_sound(mut self, sound: impl Into<SoundRef>) -> Self {
        self.sound = Some(sound.into());
        self
    }

    pub fn despawn_on_collect(mut self) -> Self {
        self.despawn_on_collect = true;
        self
    }

    pub fn build(self) -> Result<()> {
        let sprite_source = self.spec.sprite_source(self.app, "pickup", &self.name)?;
        let sound = self
            .sound
            .map(|sound| self.app.resolve_sound_ref(sound))
            .transpose()?;
        let spec = self.spec;
        let prefab_name = self.name.clone();
        let display_name = spec.display_name(&prefab_name);
        let collider = spec.collider();
        let score = self.score;

        match (sound, self.despawn_on_collect) {
            (Some(sound), true) => self.app.prefab(self.name, move |prefab| {
                prefab
                    .spawn(move |at| {
                        (
                            Name::new(display_name.clone()),
                            Transform::at(at),
                            Pickup,
                            Collectible,
                            ScoreValue(score),
                            CollectSound(sound),
                            DespawnOnCollect,
                            spec.sprite(sprite_source),
                            Collider::box_of(collider),
                        )
                    })?
                    .require::<Transform>()
                    .require::<Pickup>()
                    .require::<Collectible>()
                    .require::<ScoreValue>()
                    .require::<CollectSound>()
                    .require::<DespawnOnCollect>()
                    .require::<Sprite>()
                    .require::<Collider>();
                Ok(())
            }),
            (Some(sound), false) => self.app.prefab(self.name, move |prefab| {
                prefab
                    .spawn(move |at| {
                        (
                            Name::new(display_name.clone()),
                            Transform::at(at),
                            Pickup,
                            Collectible,
                            ScoreValue(score),
                            CollectSound(sound),
                            spec.sprite(sprite_source),
                            Collider::box_of(collider),
                        )
                    })?
                    .require::<Transform>()
                    .require::<Pickup>()
                    .require::<Collectible>()
                    .require::<ScoreValue>()
                    .require::<CollectSound>()
                    .require::<Sprite>()
                    .require::<Collider>();
                Ok(())
            }),
            (None, true) => self.app.prefab(self.name, move |prefab| {
                prefab
                    .spawn(move |at| {
                        (
                            Name::new(display_name.clone()),
                            Transform::at(at),
                            Pickup,
                            Collectible,
                            ScoreValue(score),
                            DespawnOnCollect,
                            spec.sprite(sprite_source),
                            Collider::box_of(collider),
                        )
                    })?
                    .require::<Transform>()
                    .require::<Pickup>()
                    .require::<Collectible>()
                    .require::<ScoreValue>()
                    .require::<DespawnOnCollect>()
                    .require::<Sprite>()
                    .require::<Collider>();
                Ok(())
            }),
            (None, false) => self.app.prefab(self.name, move |prefab| {
                prefab
                    .spawn(move |at| {
                        (
                            Name::new(display_name.clone()),
                            Transform::at(at),
                            Pickup,
                            Collectible,
                            ScoreValue(score),
                            spec.sprite(sprite_source),
                            Collider::box_of(collider),
                        )
                    })?
                    .require::<Transform>()
                    .require::<Pickup>()
                    .require::<Collectible>()
                    .require::<ScoreValue>()
                    .require::<Sprite>()
                    .require::<Collider>();
                Ok(())
            }),
        }
    }
}

pub struct DoorPrefabAuthor<'a, 'app> {
    app: &'a mut GameApp<'app>,
    name: String,
    spec: ObjectPrefabSpec,
    action: Option<DoorAction>,
    requires_all_enemies_dead: bool,
}

impl<'a, 'app> DoorPrefabAuthor<'a, 'app> {
    pub(crate) fn new(app: &'a mut GameApp<'app>, name: String) -> Self {
        Self {
            app,
            name,
            spec: ObjectPrefabSpec::new(DOOR_SIZE),
            action: None,
            requires_all_enemies_dead: false,
        }
    }

    pub fn named(mut self, display_name: impl Into<String>) -> Self {
        self.spec.display_name = Some(display_name.into());
        self
    }

    pub fn sprite(mut self, texture: impl Into<TextureRef>) -> Self {
        self.spec.sprite = Some(SpriteSourceRef::Texture(texture.into()));
        self
    }

    pub fn spritesheet(mut self, sheet: SpriteSheet) -> Self {
        self.spec.sprite = Some(SpriteSourceRef::Sheet(sheet));
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.spec.size = vec2s(size);
        self
    }

    pub fn size2(mut self, size: Vec2) -> Self {
        self.spec.size = size;
        self
    }

    pub fn collider(mut self, size: Vec2) -> Self {
        self.spec.collider = Some(size);
        self
    }

    pub fn tint(mut self, tint: Vec4) -> Self {
        self.spec.tint = tint;
        self
    }

    pub fn layer(mut self, layer: i16) -> Self {
        self.spec.layer = layer;
        self
    }

    pub fn change_map(mut self, map: impl Into<String>) -> Self {
        self.action = Some(DoorAction::ChangeMap(map.into()));
        self
    }

    pub fn change_scene(mut self, scene: impl Into<String>) -> Self {
        self.action = Some(DoorAction::ChangeScene(scene.into()));
        self
    }

    pub fn restart_level(mut self) -> Self {
        self.action = Some(DoorAction::RestartLevel);
        self
    }

    pub fn requires_all_enemies_dead(mut self) -> Self {
        self.requires_all_enemies_dead = true;
        self
    }

    pub fn build(self) -> Result<()> {
        let sprite_source = self.spec.sprite_source(self.app, "door", &self.name)?;
        let action = self.action.ok_or_else(|| {
            anyhow::anyhow!(
                "door prefab '{}' has no action.\n\nAdd:\n    .change_map(\"level_2\")\n\nor:\n    .restart_level()",
                self.name
            )
        })?;
        let spec = self.spec;
        let prefab_name = self.name.clone();
        let display_name = spec.display_name(&prefab_name);
        let collider = spec.collider();
        let requires_all_enemies_dead = self.requires_all_enemies_dead;

        self.app.prefab(self.name, move |prefab| {
            prefab
                .spawn(move |at| {
                    (
                        Name::new(display_name.clone()),
                        Transform::at(at),
                        Door,
                        ExitDoor,
                        DoorTarget {
                            action: action.clone(),
                            requires_all_enemies_dead,
                        },
                        spec.sprite(sprite_source),
                        Collider::box_of(collider),
                    )
                })?
                .require::<Transform>()
                .require::<Door>()
                .require::<ExitDoor>()
                .require::<DoorTarget>()
                .require::<Sprite>()
                .require::<Collider>();
            Ok(())
        })
    }
}

pub struct ProjectilePrefabAuthor<'a, 'app> {
    app: &'a mut GameApp<'app>,
    name: String,
    spec: ObjectPrefabSpec,
    speed: f32,
    damage: i32,
    lifetime: f32,
    despawn_on_hit: bool,
}

impl<'a, 'app> ProjectilePrefabAuthor<'a, 'app> {
    pub(crate) fn new(app: &'a mut GameApp<'app>, name: String) -> Self {
        Self {
            app,
            name,
            spec: ObjectPrefabSpec::new(PROJECTILE_SIZE),
            speed: PROJECTILE_SPEED,
            damage: 0,
            lifetime: 1.0,
            despawn_on_hit: false,
        }
    }

    pub fn named(mut self, display_name: impl Into<String>) -> Self {
        self.spec.display_name = Some(display_name.into());
        self
    }

    pub fn sprite(mut self, texture: impl Into<TextureRef>) -> Self {
        self.spec.sprite = Some(SpriteSourceRef::Texture(texture.into()));
        self
    }

    pub fn spritesheet(mut self, sheet: SpriteSheet) -> Self {
        self.spec.sprite = Some(SpriteSourceRef::Sheet(sheet));
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.spec.size = vec2s(size);
        self
    }

    pub fn size2(mut self, size: Vec2) -> Self {
        self.spec.size = size;
        self
    }

    pub fn collider(mut self, size: Vec2) -> Self {
        self.spec.collider = Some(size);
        self
    }

    pub fn tint(mut self, tint: Vec4) -> Self {
        self.spec.tint = tint;
        self
    }

    pub fn layer(mut self, layer: i16) -> Self {
        self.spec.layer = layer;
        self
    }

    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    pub fn damage(mut self, amount: i32) -> Self {
        self.damage = amount;
        self
    }

    pub fn lifetime(mut self, seconds: f32) -> Self {
        self.lifetime = seconds.max(0.0);
        self
    }

    pub fn despawn_on_hit(mut self) -> Self {
        self.despawn_on_hit = true;
        self
    }

    pub fn build(self) -> Result<()> {
        let sprite_source = self
            .spec
            .sprite_source(self.app, "projectile", &self.name)?;
        let spec = self.spec;
        let prefab_name = self.name.clone();
        let display_name = spec.display_name(&prefab_name);
        let collider = spec.collider();
        let speed = self.speed;
        let damage = self.damage;
        let lifetime = self.lifetime;

        if self.despawn_on_hit {
            self.app.prefab(self.name, move |prefab| {
                prefab
                    .spawn_with_properties(move |at, properties| {
                        let direction = projectile_direction(properties);
                        (
                            Name::new(display_name.clone()),
                            Transform::at(at),
                            Velocity::new(direction * speed),
                            Projectile,
                            PlayerProjectile,
                            Speed::new(speed),
                            ProjectileDamage { amount: damage },
                            Lifetime {
                                seconds_left: lifetime,
                            },
                            DespawnOnHit,
                            spec.sprite(sprite_source),
                            Collider::box_of(collider),
                        )
                    })?
                    .require::<Transform>()
                    .require::<Velocity>()
                    .require::<Projectile>()
                    .require::<PlayerProjectile>()
                    .require::<Speed>()
                    .require::<ProjectileDamage>()
                    .require::<Lifetime>()
                    .require::<DespawnOnHit>()
                    .require::<Sprite>()
                    .require::<Collider>();
                Ok(())
            })
        } else {
            self.app.prefab(self.name, move |prefab| {
                prefab
                    .spawn_with_properties(move |at, properties| {
                        let direction = projectile_direction(properties);
                        (
                            Name::new(display_name.clone()),
                            Transform::at(at),
                            Velocity::new(direction * speed),
                            Projectile,
                            PlayerProjectile,
                            Speed::new(speed),
                            ProjectileDamage { amount: damage },
                            Lifetime {
                                seconds_left: lifetime,
                            },
                            spec.sprite(sprite_source),
                            Collider::box_of(collider),
                        )
                    })?
                    .require::<Transform>()
                    .require::<Velocity>()
                    .require::<Projectile>()
                    .require::<PlayerProjectile>()
                    .require::<Speed>()
                    .require::<ProjectileDamage>()
                    .require::<Lifetime>()
                    .require::<Sprite>()
                    .require::<Collider>();
                Ok(())
            })
        }
    }
}

pub struct SpawnerPrefabAuthor<'a, 'app> {
    app: &'a mut GameApp<'app>,
    name: String,
    display_name: Option<String>,
    prefab: Option<String>,
    every_seconds: f32,
    max_alive: Option<usize>,
    placement: SpawnPlacement,
}

impl<'a, 'app> SpawnerPrefabAuthor<'a, 'app> {
    pub(crate) fn new(app: &'a mut GameApp<'app>, name: String) -> Self {
        Self {
            app,
            name,
            display_name: None,
            prefab: None,
            every_seconds: 1.0,
            max_alive: None,
            placement: SpawnPlacement::AtSpawner,
        }
    }

    pub fn named(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    pub fn spawn(mut self, prefab: impl Into<String>) -> Self {
        self.prefab = Some(prefab.into());
        self
    }

    pub fn every_seconds(mut self, seconds: f32) -> Self {
        self.every_seconds = seconds.max(0.001);
        self
    }

    pub fn max_alive(mut self, max_alive: usize) -> Self {
        self.max_alive = Some(max_alive);
        self
    }

    pub fn at_spawner(mut self) -> Self {
        self.placement = SpawnPlacement::AtSpawner;
        self
    }

    pub fn near_player(mut self, radius: f32) -> Self {
        self.placement = SpawnPlacement::NearPlayer {
            radius: radius.max(0.0),
        };
        self
    }

    pub fn at_first_floor(mut self) -> Self {
        self.placement = SpawnPlacement::AtFirstFloor;
        self
    }

    pub fn build(self) -> Result<()> {
        let spawn_prefab = self.prefab.ok_or_else(|| {
            anyhow::anyhow!(
                "spawner prefab '{}' does not name a prefab to spawn.\n\nAdd:\n    .spawn(\"slime\")",
                self.name
            )
        })?;
        let prefab_name = self.name.clone();
        let display_name = self.display_name.unwrap_or_else(|| prefab_name.clone());
        let every_seconds = self.every_seconds;
        let max_alive = self.max_alive;
        let placement = self.placement;

        self.app.prefab(self.name, move |prefab| {
            prefab
                .spawn(move |at| {
                    (
                        Name::new(display_name.clone()),
                        Transform::at(at),
                        Spawner {
                            prefab: spawn_prefab.clone(),
                            every_seconds,
                            timer: 0.0,
                            max_alive,
                            placement: placement.clone(),
                        },
                    )
                })?
                .require::<Transform>()
                .require::<Spawner>();
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use game_ai::Patrol;
    use game_core::backend::{AudioCommand, SoundHandle, TextureHandle};

    use super::*;
    use crate::app::{GameApp, GamePlugin};
    use crate::context::StartupGameCtx;
    use crate::harness::GameTestHarness;

    struct ObjectPrefabPlugin;

    impl GamePlugin for ObjectPrefabPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
            game.pickup_prefab("coin")
                .sprite(TextureHandle(1))
                .score(1)
                .play_sound(SoundHandle(1))
                .despawn_on_collect()
                .build()?;

            game.door_prefab("exit")
                .sprite(TextureHandle(2))
                .change_map("level_2")
                .requires_all_enemies_dead()
                .build()?;

            game.projectile_prefab("fireball")
                .sprite(TextureHandle(3))
                .speed(320.0)
                .damage(10)
                .lifetime(2.0)
                .despawn_on_hit()
                .build()?;

            game.spawner_prefab("spawner")
                .spawn("coin")
                .every_seconds(3.0)
                .max_alive(5)
                .build()?;

            game.map("objects")
                .tiles(["#####", "#CDS#", "#F..#", "#####"])
                .simple_theme(TextureHandle(10), TextureHandle(11))
                .legend('C', "coin")
                .legend('D', "exit")
                .legend('F', "fireball")
                .legend('S', "spawner")
                .start();

            game.on_start(|game: &mut StartupGameCtx<'_, '_>| game.spawn_start_map());
            Ok(())
        }
    }

    #[test]
    fn object_prefab_builders_spawn_expected_components() {
        let game = GameTestHarness::from_plugin(ObjectPrefabPlugin).unwrap();

        assert_eq!(game.count::<Pickup>(), 1);
        assert_eq!(game.count::<Collectible>(), 1);
        assert_eq!(game.count::<ScoreValue>(), 1);
        assert_eq!(game.count::<CollectSound>(), 1);
        assert_eq!(game.count::<DespawnOnCollect>(), 1);
        assert_eq!(game.count::<Door>(), 1);
        assert_eq!(game.count::<ExitDoor>(), 1);
        assert_eq!(game.count::<DoorTarget>(), 1);
        assert_eq!(game.count::<Projectile>(), 1);
        assert_eq!(game.count::<ProjectileDamage>(), 1);
        assert_eq!(game.count::<Lifetime>(), 1);
        assert_eq!(game.count::<DespawnOnHit>(), 1);
        assert_eq!(game.count::<Spawner>(), 1);
    }

    struct PatrolPrefabPlugin;

    impl GamePlugin for PatrolPrefabPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
            game.enemy_prefab("patroller")
                .sprite(TextureHandle(1))
                .patrol_horizontal(64.0)
                .patrol_speed(40.0)
                .build()?;

            game.map("patrol")
                .tiles(["###", "#P#", "###"])
                .simple_theme(TextureHandle(10), TextureHandle(11))
                .legend('P', "patroller")
                .start();

            game.on_start(|game: &mut StartupGameCtx<'_, '_>| game.spawn_start_map());
            Ok(())
        }
    }

    #[test]
    fn enemy_prefab_can_spawn_patrol_component() {
        let game = GameTestHarness::from_plugin(PatrolPrefabPlugin).unwrap();

        assert_eq!(game.count::<Enemy>(), 1);
        assert_eq!(game.count::<Patrol>(), 1);
    }

    struct NamedAssetPrefabPlugin;

    impl GamePlugin for NamedAssetPrefabPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
            game.asset_bag()
                .texture("player", "textures/player.png")?
                .texture("slime", "textures/slime.png")?
                .texture("coin", "textures/coin.png")?
                .texture("door", "textures/door.png")?
                .texture("bolt", "textures/bolt.png")?
                .texture("floor", "textures/floor.png")?
                .texture("wall", "textures/wall.png")?
                .generated_sound("coin")?
                .build();
            let controls = game.input(|input| input.top_down_controls())?;

            game.player_prefab("player")
                .sprite("player")
                .moves_with(controls.movement, 120.0)
                .build()?;
            game.enemy_prefab("slime").sprite("slime").build()?;
            game.pickup_prefab("coin")
                .sprite("coin")
                .play_sound("coin")
                .build()?;
            game.door_prefab("door")
                .sprite("door")
                .restart_level()
                .build()?;
            game.projectile_prefab("bolt").sprite("bolt").build()?;

            game.map("named-assets")
                .tiles(["#####", "#P..#", "#####"])
                .simple_theme("floor", "wall")
                .legend('P', "player")
                .start();
            game.on_start(|game: &mut StartupGameCtx<'_, '_>| game.spawn_start_map());
            game.on_action(controls.attack, |game| game.play_sound_named("coin"));
            Ok(())
        }
    }

    #[test]
    fn beginner_prefab_and_map_builders_resolve_named_assets() {
        let mut game = GameTestHarness::from_plugin(NamedAssetPrefabPlugin).unwrap();

        assert_eq!(game.count::<Player>(), 1);
        assert_eq!(game.count::<Sprite>(), 1);

        game = game.press_action("attack");
        game.fixed_step(1.0 / 120.0);
        assert_eq!(
            game.audio_commands(),
            &[AudioCommand::Play {
                sound: SoundHandle(0),
                volume: 1.0,
                looping: false,
            }]
        );
    }

    struct MissingNamedAssetPlugin;

    impl GamePlugin for MissingNamedAssetPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
            game.asset_bag()
                .texture("player", "textures/player.png")?
                .build();
            let controls = game.input(|input| input.top_down_controls())?;
            game.player_prefab("player")
                .sprite("plaeyr")
                .moves_with(controls.movement, 120.0)
                .build()
        }
    }

    #[test]
    fn named_prefab_assets_report_friendly_missing_key_diagnostics() {
        let error = match GameTestHarness::from_plugin(MissingNamedAssetPlugin) {
            Ok(_) => panic!("missing named asset should reject the prefab"),
            Err(error) => error,
        };
        let message = error.to_string();
        assert!(message.contains("Unknown texture asset 'plaeyr'"));
        assert!(message.contains("- player"));
        assert!(message.contains("Did you mean 'player'?"));
    }
}
