use super::shared::*;

pub struct ProjectilePrefabAuthor<'a, 'app> {
    app: &'a mut GameApp<'app>,
    name: String,
    spec: ObjectPrefabSpec,
    speed: f32,
    damage: i32,
    lifetime: f32,
    despawn_on_hit: bool,
    flight: Option<AnimationClip>,
    impact: Option<AnimationClip>,
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
            flight: None,
            impact: None,
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

    /// Uses `flight` and `impact` clips from an animation metadata sheet when
    /// those names are present.
    pub fn animation_sheet(mut self, sheet: AnimationSheet) -> Self {
        let (sheet, clips) = sheet.into_parts();
        self.spec.sprite = Some(SpriteSourceRef::Sheet(sheet));
        for (name, clip) in clips {
            match name.as_str() {
                "flight" => self.flight = Some(clip),
                "impact" => self.impact = Some(clip),
                _ => {}
            }
        }
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

    /// Registers a looping `flight` clip for a spritesheet projectile.
    pub fn flight(mut self, frames: impl IntoIterator<Item = usize>) -> Self {
        self.flight = Some(walk_frames(frames));
        self
    }

    /// Registers a one-shot `impact` clip for use with
    /// `game.rules().projectile_impact_animation_before_despawn()`.
    pub fn impact(mut self, frames: impl IntoIterator<Item = usize>) -> Self {
        self.impact = Some(attack_frames(frames));
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
        let (animation, animation_set, first_frame) =
            projectile_animation_components(sprite_source, self.flight, self.impact, &prefab_name)?;

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
                            spec.sprite_at(sprite_source, first_frame),
                            animation.clone(),
                            animation_set.clone(),
                            Tags(spec.tags.clone()),
                            NamedValues::from(spec.named_values.clone()),
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
                    .require::<Animation>()
                    .require::<AnimationSet>()
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
                            spec.sprite_at(sprite_source, first_frame),
                            animation.clone(),
                            animation_set.clone(),
                            Tags(spec.tags.clone()),
                            NamedValues::from(spec.named_values.clone()),
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
                    .require::<Animation>()
                    .require::<AnimationSet>()
                    .require::<Collider>();
                Ok(())
            })
        }
    }
}

fn projectile_animation_components(
    source: SpriteSource,
    flight: Option<AnimationClip>,
    impact: Option<AnimationClip>,
    prefab_name: &str,
) -> Result<(Animation, AnimationSet, usize)> {
    let custom_animation = flight.is_some() || impact.is_some();
    let sheet = match source {
        SpriteSource::Sheet(sheet) => sheet,
        SpriteSource::Texture(texture) if !custom_animation => SpriteSheet::new(texture, 1, 1),
        SpriteSource::Texture(_) => anyhow::bail!(
            "projectile prefab '{prefab_name}' has flight/impact animations but uses a static sprite.\n\nUse:\n    .spritesheet(assets.projectile_sheet)"
        ),
    };

    let mut set = AnimationSet::new(sheet);
    let initial = if let Some(flight) = flight {
        set = set.animation("flight", flight);
        "flight"
    } else if impact.is_some() {
        "impact"
    } else {
        set = set.animation("flight", AnimationClip::frames([0]));
        "flight"
    };
    if let Some(impact) = impact {
        set = set.animation("impact", impact);
    }
    let first_frame = set
        .get(initial)
        .and_then(|clip| clip.frames.first())
        .copied()
        .unwrap_or(0);
    Ok((Animation::play(initial), set, first_frame))
}
