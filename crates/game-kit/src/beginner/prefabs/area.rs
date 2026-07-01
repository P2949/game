use super::shared::*;

pub struct AreaPrefabAuthor<'a, 'app> {
    app: &'a mut GameApp<'app>,
    name: String,
    display_name: Option<String>,
    size: Vec2,
    collider: Option<Vec2>,
    debug_sprite: Option<SpriteSourceRef>,
    tint: Vec4,
    layer: i16,
    checkpoint: bool,
    tags: HashSet<String>,
    named_values: HashMap<String, f32>,
}

impl<'a, 'app> AreaPrefabAuthor<'a, 'app> {
    pub(crate) fn new(app: &'a mut GameApp<'app>, name: String) -> Self {
        Self {
            app,
            name,
            display_name: None,
            size: vec2s(AREA_SIZE),
            collider: None,
            debug_sprite: None,
            tint: Vec4::new(1.0, 0.2, 0.2, 0.35),
            layer: DEFAULT_LAYER,
            checkpoint: false,
            tags: HashSet::new(),
            named_values: HashMap::new(),
        }
    }

    pub(crate) fn new_checkpoint(app: &'a mut GameApp<'app>, name: String) -> Self {
        let mut author = Self::new(app, name);
        author.checkpoint = true;
        author
    }

    pub fn named(mut self, name: impl IntoContentName) -> Self {
        self.display_name = Some(name.into_content_name());
        self
    }

    /// Adds a named marker that custom beginner rules can select later.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.insert(tag.into());
        self
    }

    /// Adds a named numeric value that custom beginner rules can update later.
    pub fn data(mut self, key: impl Into<String>, value: f32) -> Self {
        self.named_values.insert(key.into(), value);
        self
    }

    /// Sets the visible and collision size of this area.
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    /// Alias for [`Self::size`], matching other prefab builders.
    pub fn size2(self, size: Vec2) -> Self {
        self.size(size)
    }

    pub fn collider(mut self, size: Vec2) -> Self {
        self.collider = Some(size);
        self
    }

    /// Areas are trigger-only by default. This reads naturally in examples
    /// that want to make the non-solid behavior explicit.
    pub fn trigger_only(self) -> Self {
        self
    }

    /// Adds an optional, non-gameplay debug sprite to this area.
    pub fn visible_debug(mut self, texture: impl IntoTextureRef) -> Self {
        self.debug_sprite = Some(SpriteSourceRef::Texture(texture.into_texture_ref()));
        self
    }

    /// Displays this area with a sprite. For normal areas this is a visual aid;
    /// checkpoint areas use it as their regular marker.
    pub fn sprite(self, texture: impl IntoTextureRef) -> Self {
        self.visible_debug(texture)
    }

    pub fn tint(mut self, tint: Vec4) -> Self {
        self.tint = tint;
        self
    }

    pub fn layer(mut self, layer: i16) -> Self {
        self.layer = layer;
        self
    }

    pub fn build(self) -> Result<()> {
        let debug_sprite = self
            .debug_sprite
            .map(|source| match source {
                SpriteSourceRef::Texture(texture) => self
                    .app
                    .resolve_texture_ref(texture)
                    .map(SpriteSource::Texture),
                SpriteSourceRef::Sheet(sheet) => Ok(SpriteSource::Sheet(sheet)),
            })
            .transpose()?;
        let prefab_name = self.name.clone();
        let display_name = self.display_name.unwrap_or_else(|| prefab_name.clone());
        let area_name = prefab_name.clone();
        let size = self.size;
        let collider = self.collider.unwrap_or(size);
        let tint = self.tint;
        let layer = self.layer;
        let tags = self.tags;
        let named_values = self.named_values;
        let checkpoint = Checkpoint {
            enabled: self.checkpoint,
        };

        if let Some(source) = debug_sprite {
            self.app.prefab(self.name, move |prefab| {
                prefab
                    .spawn(move |at| {
                        let sprite = match source {
                            SpriteSource::Texture(texture) => Sprite::new(texture, size),
                            SpriteSource::Sheet(sheet) => sheet.sprite(0, size),
                        }
                        .layer(layer)
                        .tint(tint);
                        (
                            Name::new(display_name.clone()),
                            Transform::at(at),
                            Area,
                            TriggerArea,
                            checkpoint,
                            AreaName(area_name.clone()),
                            Trigger,
                            Collider::box_of(collider),
                            sprite,
                            Tags(tags.clone()),
                            NamedValues::from(named_values.clone()),
                        )
                    })?
                    .require::<Transform>()
                    .require::<Area>()
                    .require::<TriggerArea>()
                    .require::<AreaName>()
                    .require::<Trigger>()
                    .require::<Collider>()
                    .require::<Sprite>();
                Ok(())
            })
        } else {
            self.app.prefab(self.name, move |prefab| {
                prefab
                    .spawn(move |at| {
                        (
                            Name::new(display_name.clone()),
                            Transform::at(at),
                            Area,
                            TriggerArea,
                            checkpoint,
                            AreaName(area_name.clone()),
                            Trigger,
                            Collider::box_of(collider),
                            Tags(tags.clone()),
                            NamedValues::from(named_values.clone()),
                        )
                    })?
                    .require::<Transform>()
                    .require::<Area>()
                    .require::<TriggerArea>()
                    .require::<AreaName>()
                    .require::<Trigger>()
                    .require::<Collider>();
                Ok(())
            })
        }
    }
}
