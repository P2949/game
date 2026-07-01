use super::shared::*;

pub struct PickupPrefabAuthor<'a, 'app> {
    app: &'a mut GameApp<'app>,
    name: String,
    spec: ObjectPrefabSpec,
    score: i32,
    heal: i32,
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
            heal: 0,
            sound: None,
            despawn_on_collect: false,
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

    /// Restores the player's health when this pickup is collected.
    pub fn heal_player(mut self, amount: i32) -> Self {
        self.heal = amount.max(0);
        self
    }

    pub fn play_sound(mut self, sound: impl IntoSoundRef) -> Self {
        self.sound = Some(sound.into_sound_ref());
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
        let heal = self.heal;

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
                            HealValue(heal),
                            CollectSound(sound),
                            DespawnOnCollect,
                            spec.sprite(sprite_source),
                            Tags(spec.tags.clone()),
                            NamedValues::from(spec.named_values.clone()),
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
                            HealValue(heal),
                            CollectSound(sound),
                            spec.sprite(sprite_source),
                            Tags(spec.tags.clone()),
                            NamedValues::from(spec.named_values.clone()),
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
                            HealValue(heal),
                            DespawnOnCollect,
                            spec.sprite(sprite_source),
                            Tags(spec.tags.clone()),
                            NamedValues::from(spec.named_values.clone()),
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
                            HealValue(heal),
                            spec.sprite(sprite_source),
                            Tags(spec.tags.clone()),
                            NamedValues::from(spec.named_values.clone()),
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
