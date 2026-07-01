use super::shared::*;

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

    pub fn moves_with(mut self, axis: impl IntoMovementAxis, speed: impl Into<TunedF32>) -> Self {
        self.movement_axis = Some(axis.into_movement_axis());
        self.spec.speed = speed.into();
        self
    }

    pub fn melee(mut self, range: f32, damage: impl Into<TunedI32>) -> Self {
        self.spec.melee = Some(MeleeSpec {
            range,
            damage: damage.into(),
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
        let melee = spec.melee.clone().unwrap_or(MeleeSpec {
            range: 30.0,
            damage: 0.into(),
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
                    .spawn_with_world(move |world, at| {
                        (
                            Name::new(display_name.clone()),
                            Transform::at(at),
                            Velocity::default(),
                            Player,
                            FacingDirection::default(),
                            PlayerMovement::axis(movement_axis),
                            Speed::new(spec.speed.resolve(world)),
                            Health::new(spec.health.resolve(world)),
                            MeleeAttack::new(melee.range, melee.damage.resolve(world))
                                .cooldown(melee.cooldown),
                            Faction::player(),
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
                    .spawn_with_world(move |world, at| {
                        (
                            Name::new(display_name.clone()),
                            Transform::at(at),
                            Velocity::default(),
                            Player,
                            FacingDirection::default(),
                            PlayerMovement::axis(movement_axis),
                            Speed::new(spec.speed.resolve(world)),
                            Health::new(spec.health.resolve(world)),
                            MeleeAttack::new(melee.range, melee.damage.resolve(world))
                                .cooldown(melee.cooldown),
                            Faction::player(),
                            spec.sprite(sprite_source, 0),
                            Tags(spec.tags.clone()),
                            NamedValues::from(spec.named_values.clone()),
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
