use super::shared::*;

pub struct DoorPrefabAuthor<'a, 'app> {
    app: &'a mut GameApp<'app>,
    name: String,
    spec: ObjectPrefabSpec,
    action: Option<DoorAction>,
    requires_all_enemies_dead: bool,
}

/// Builder for a collider-only trigger zone. Areas intentionally do not need a
/// sprite; add `.visible_debug("debug_trigger")` when a temporary visual is
/// useful while authoring a level.
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
                        Tags(spec.tags.clone()),
                        NamedValues::from(spec.named_values.clone()),
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
