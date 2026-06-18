//! Beginner prefab builders.

use anyhow::Result;
use game_ai::{AiController, ChaseTarget, PathFollow};
use game_combat::{Faction, Health, MeleeAttack};
use game_core::backend::TextureHandle;
use game_core::input::Axis2dId;
use game_core::world::{Sprite, Transform, Velocity};
use game_physics::Collider;
use glam::{Vec2, Vec4};

use crate::app::GameApp;
use crate::beginner::actors::{Enemy, Name, Player, PlayerMovement, Speed};
use crate::beginner::animation::{Animation, AnimationClip, AnimationSet, SpriteSheet};
use crate::bundle::vec2s;

const PLAYER_SIZE: f32 = 20.0;
const ENEMY_SIZE: f32 = 22.0;
const PLAYER_HEALTH: i32 = 100;
const ENEMY_HEALTH: i32 = 40;
const PLAYER_SPEED: f32 = 130.0;
const ENEMY_SPEED: f32 = 80.0;
const DEFAULT_LAYER: i16 = 10;
const ENEMY_CHASE_RANGE: f32 = 180.0;
const ENEMY_REPATH_SECONDS: f32 = 0.25;
const ENEMY_MELEE_COOLDOWN: f32 = 0.75;

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
struct ActorPrefabSpec {
    display_name: Option<String>,
    sprite: Option<SpriteSource>,
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

    fn sprite_source(&self, kind: &str, prefab_name: &str) -> Result<SpriteSource> {
        let label = actor_kind_label(kind);
        self.sprite.ok_or_else(|| {
            anyhow::anyhow!(
                "{label} prefab '{prefab_name}' has no sprite.\n\nAdd:\n    .sprite(assets.{kind})\n\nor:\n    .spritesheet(assets.{kind}_sheet)\n\nExample:\n    game.{kind}_prefab(\"{prefab_name}\")\n        .sprite(assets.{kind})\n        .build()?;"
            )
        })
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
        _ => "Actor",
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

    pub fn sprite(mut self, texture: TextureHandle) -> Self {
        self.spec.sprite = Some(SpriteSource::Texture(texture));
        self
    }

    pub fn spritesheet(mut self, sheet: SpriteSheet) -> Self {
        self.spec.sprite = Some(SpriteSource::Sheet(sheet));
        self
    }

    pub fn animation(mut self, name: impl Into<String>, clip: AnimationClip) -> Self {
        self.spec.animations.push((name.into(), clip));
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
        let sprite_source = self.spec.sprite_source("player", &self.name)?;
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

pub struct EnemyPrefabAuthor<'a, 'app> {
    app: &'a mut GameApp<'app>,
    name: String,
    spec: ActorPrefabSpec,
    chase: Option<ChaseSpec>,
}

impl<'a, 'app> EnemyPrefabAuthor<'a, 'app> {
    pub(crate) fn new(app: &'a mut GameApp<'app>, name: String) -> Self {
        Self {
            app,
            name,
            spec: ActorPrefabSpec::enemy(),
            chase: None,
        }
    }

    pub fn named(mut self, display_name: impl Into<String>) -> Self {
        self.spec.display_name = Some(display_name.into());
        self
    }

    pub fn sprite(mut self, texture: TextureHandle) -> Self {
        self.spec.sprite = Some(SpriteSource::Texture(texture));
        self
    }

    pub fn spritesheet(mut self, sheet: SpriteSheet) -> Self {
        self.spec.sprite = Some(SpriteSource::Sheet(sheet));
        self
    }

    pub fn animation(mut self, name: impl Into<String>, clip: AnimationClip) -> Self {
        self.spec.animations.push((name.into(), clip));
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

    pub fn build(self) -> Result<()> {
        let sprite_source = self.spec.sprite_source("enemy", &self.name)?;
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

        if let Some(chase) = self.chase {
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
