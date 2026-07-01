use super::shared::*;

pub struct EnemyPrefabAuthor<'a, 'app> {
    app: &'a mut GameApp<'app>,
    name: String,
    spec: ActorPrefabSpec,
    chase: Option<ChaseSpec>,
    patrol: Option<PatrolSpec>,
    patrol_speed: Option<f32>,
    despawn_after_death_animation: bool,
    drops: DropsPrefab,
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
            drops: DropsPrefab::default(),
        }
    }

    pub fn named(mut self, display_name: impl IntoContentName) -> Self {
        self.spec.display_name = Some(display_name.into_content_name());
        self
    }

    /// Adds a named marker that custom beginner rules can select later.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.spec.tags.insert(tag.into());
        self
    }

    /// Adds a named numeric value that custom beginner rules can update later.
    pub fn data(mut self, key: impl Into<String>, value: f32) -> Self {
        self.spec.named_values.insert(key.into(), value);
        self
    }

    pub fn sprite(mut self, texture: impl IntoTextureRef) -> Self {
        self.spec.sprite = Some(SpriteSourceRef::Texture(texture.into_texture_ref()));
        self
    }

    pub fn spritesheet(mut self, sheet: SpriteSheet) -> Self {
        self.spec.sprite = Some(SpriteSourceRef::Sheet(sheet));
        self
    }

    /// Uses a metadata-authored spritesheet and all of its named clips.
    pub fn animation_sheet(mut self, sheet: AnimationSheet) -> Self {
        let (sheet, clips) = sheet.into_parts();
        self.spec.sprite = Some(SpriteSourceRef::Sheet(sheet));
        self.spec.animations.extend(clips);
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

    pub fn attack_up(self, frames: impl IntoIterator<Item = usize>) -> Self {
        self.animation("attack_up", attack_frames(frames))
    }

    pub fn attack_down(self, frames: impl IntoIterator<Item = usize>) -> Self {
        self.animation("attack_down", attack_frames(frames))
    }

    pub fn attack_left(self, frames: impl IntoIterator<Item = usize>) -> Self {
        self.animation("attack_left", attack_frames(frames))
    }

    pub fn attack_right(self, frames: impl IntoIterator<Item = usize>) -> Self {
        self.animation("attack_right", attack_frames(frames))
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

    pub fn health(mut self, health: impl Into<TunedI32>) -> Self {
        self.spec.health = health.into();
        self
    }

    pub fn speed(mut self, speed: impl Into<TunedF32>) -> Self {
        self.spec.speed = speed.into();
        self
    }

    pub fn melee(mut self, range: f32, damage: impl Into<TunedI32>) -> Self {
        self.spec.melee = Some(MeleeSpec {
            range,
            damage: damage.into(),
            cooldown: ENEMY_MELEE_COOLDOWN,
        });
        self
    }

    pub fn melee_cooldown(mut self, cooldown: f32) -> Self {
        let melee = self.spec.melee.get_or_insert(MeleeSpec {
            range: 26.0,
            damage: 0.into(),
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

    /// Spawns this prefab at the enemy's position when it is defeated.
    pub fn drops(mut self, prefab: impl Into<String>) -> Self {
        self.drops = DropsPrefab {
            prefab: prefab.into(),
            chance: 1.0,
        };
        self
    }

    /// Sets the chance for the configured drop. `0.0` disables it and `1.0`
    /// always drops; intermediate values use a stable per-enemy roll.
    pub fn drop_chance(mut self, chance: f32) -> Self {
        self.drops.chance = chance.clamp(0.0, 1.0);
        self
    }

    pub fn build(self) -> Result<()> {
        let sprite_source = self.spec.sprite_source(self.app, "enemy", &self.name)?;
        let spec = self.spec;
        let prefab_name = self.name.clone();
        let display_name = spec.display_name(&prefab_name);
        let collider = spec.collider();
        let melee = spec.melee.clone().unwrap_or(MeleeSpec {
            range: 26.0,
            damage: 0.into(),
            cooldown: ENEMY_MELEE_COOLDOWN,
        });
        let animation_components =
            spec.animation_components(sprite_source, "enemy", &prefab_name)?;
        let patrol = self.patrol;
        let patrol_speed = self.patrol_speed;
        let death_animation_policy = DeathAnimationPolicy {
            despawn_after_animation: self.despawn_after_death_animation,
        };
        let drops = self.drops;

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
                        .spawn_with_world(move |world, at| {
                            (
                                Name::new(display_name.clone()),
                                Transform::at(at),
                                Velocity::default(),
                                Enemy,
                                death_animation_policy,
                                drops.clone(),
                                Speed::new(spec.speed.resolve(world)),
                                Health::new(spec.health.resolve(world)),
                                Faction::enemy(),
                                MeleeAttack::new(melee.range, melee.damage.resolve(world))
                                    .cooldown(melee.cooldown),
                                AiController::chase_player(),
                                ChaseTarget::player(
                                    chase.range,
                                    stop_distance,
                                    spec.speed.resolve(world),
                                    chase.repath_seconds,
                                ),
                                PathFollow::default(),
                                spec.sprite(sprite_source, first_frame),
                                animation.clone(),
                                animation_set.clone(),
                                Tags(spec.tags.clone()),
                                NamedValues::from(spec.named_values.clone()),
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
                        .spawn_with_world(move |world, at| {
                            (
                                Name::new(display_name.clone()),
                                Transform::at(at),
                                Velocity::default(),
                                Enemy,
                                death_animation_policy,
                                drops.clone(),
                                Speed::new(spec.speed.resolve(world)),
                                Health::new(spec.health.resolve(world)),
                                Faction::enemy(),
                                MeleeAttack::new(melee.range, melee.damage.resolve(world))
                                    .cooldown(melee.cooldown),
                                AiController::chase_player(),
                                ChaseTarget::player(
                                    chase.range,
                                    stop_distance,
                                    spec.speed.resolve(world),
                                    chase.repath_seconds,
                                ),
                                PathFollow::default(),
                                spec.sprite(sprite_source, 0),
                                Tags(spec.tags.clone()),
                                NamedValues::from(spec.named_values.clone()),
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
                        .spawn_with_world(move |world, at| {
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
                                drops.clone(),
                                Speed::new(spec.speed.resolve(world)),
                                Health::new(spec.health.resolve(world)),
                                Faction::enemy(),
                                MeleeAttack::new(melee.range, melee.damage.resolve(world))
                                    .cooldown(melee.cooldown),
                                Patrol::new(
                                    waypoints,
                                    patrol_speed.unwrap_or_else(|| spec.speed.resolve(world)),
                                ),
                                spec.sprite(sprite_source, first_frame),
                                animation.clone(),
                                animation_set.clone(),
                                Tags(spec.tags.clone()),
                                NamedValues::from(spec.named_values.clone()),
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
                        .spawn_with_world(move |world, at| {
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
                                drops.clone(),
                                Speed::new(spec.speed.resolve(world)),
                                Health::new(spec.health.resolve(world)),
                                Faction::enemy(),
                                MeleeAttack::new(melee.range, melee.damage.resolve(world))
                                    .cooldown(melee.cooldown),
                                Patrol::new(
                                    waypoints,
                                    patrol_speed.unwrap_or_else(|| spec.speed.resolve(world)),
                                ),
                                spec.sprite(sprite_source, 0),
                                Tags(spec.tags.clone()),
                                NamedValues::from(spec.named_values.clone()),
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
                        .spawn_with_world(move |world, at| {
                            (
                                Name::new(display_name.clone()),
                                Transform::at(at),
                                Velocity::default(),
                                Enemy,
                                death_animation_policy,
                                drops.clone(),
                                Speed::new(spec.speed.resolve(world)),
                                Health::new(spec.health.resolve(world)),
                                Faction::enemy(),
                                MeleeAttack::new(melee.range, melee.damage.resolve(world))
                                    .cooldown(melee.cooldown),
                                spec.sprite(sprite_source, first_frame),
                                animation.clone(),
                                animation_set.clone(),
                                Tags(spec.tags.clone()),
                                NamedValues::from(spec.named_values.clone()),
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
                        .spawn_with_world(move |world, at| {
                            (
                                Name::new(display_name.clone()),
                                Transform::at(at),
                                Velocity::default(),
                                Enemy,
                                death_animation_policy,
                                drops.clone(),
                                Speed::new(spec.speed.resolve(world)),
                                Health::new(spec.health.resolve(world)),
                                Faction::enemy(),
                                MeleeAttack::new(melee.range, melee.damage.resolve(world))
                                    .cooldown(melee.cooldown),
                                spec.sprite(sprite_source, 0),
                                Tags(spec.tags.clone()),
                                NamedValues::from(spec.named_values.clone()),
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
