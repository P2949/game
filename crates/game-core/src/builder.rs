use std::any::TypeId;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use crate::app::{MapData, TileTheme};
use crate::assets::AssetRegistry;
use crate::input::InputRegistry;
use crate::nav::NavGrid;
use crate::schedule::Schedule;
use crate::tilemap::TileMap;
use crate::world::{Component, EntityId, World};

/// Registry-assigned identifier for a map, minted by [`MapRegistry`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MapId(pub u32);

/// Registry-assigned identifier for a prefab, minted by [`PrefabRegistry`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PrefabId(pub u32);

/// Free-form string properties attached to a spawned map object.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PropertyBag {
    values: HashMap<String, String>,
}

impl PropertyBag {
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.values.insert(key.into(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }
}

#[derive(Default)]
pub struct MapRegistry {
    maps: Vec<RegisteredMap>,
}

pub struct RegisteredMap {
    pub name: String,
    pub data: MapData,
}

impl MapRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Low-level convenience wrapper around [`Self::try_register`] that panics
    /// on duplicate map names. Content should use `game-kit::MapAuthor`, which
    /// returns `Result`.
    pub fn register(
        &mut self,
        name: impl Into<String>,
        tilemap: TileMap,
        theme: TileTheme,
    ) -> MapId {
        self.try_register(name, tilemap, theme)
            .expect("map names must be unique")
    }

    pub fn try_register(
        &mut self,
        name: impl Into<String>,
        tilemap: TileMap,
        theme: TileTheme,
    ) -> anyhow::Result<MapId> {
        let name = name.into();
        if self.maps.iter().any(|map| map.name == name) {
            anyhow::bail!("duplicate map name '{name}'");
        }

        let id = MapId(self.maps.len() as u32);
        let nav = NavGrid::from_tilemap(&tilemap);
        self.maps.push(RegisteredMap {
            name,
            data: MapData {
                tilemap,
                nav,
                theme,
            },
        });
        Ok(id)
    }

    pub fn get(&self, id: MapId) -> Option<&RegisteredMap> {
        self.maps.get(id.0 as usize)
    }

    pub fn id(&self, name: &str) -> Option<MapId> {
        self.maps
            .iter()
            .position(|map| map.name == name)
            .map(|index| MapId(index as u32))
    }

    pub fn len(&self) -> usize {
        self.maps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.maps.is_empty()
    }
}

type PrefabSpawnFn = dyn Fn(&mut World, glam::Vec2, &PropertyBag) -> anyhow::Result<EntityId>;

pub struct Prefab {
    spawn: Box<PrefabSpawnFn>,
}

impl Prefab {
    pub fn new(
        spawn: impl Fn(&mut World, glam::Vec2, &PropertyBag) -> anyhow::Result<EntityId> + 'static,
    ) -> Self {
        Self {
            spawn: Box::new(spawn),
        }
    }
}

#[derive(Default)]
pub struct PrefabRegistry {
    prefabs: HashMap<PrefabId, Prefab>,
    names: HashMap<String, PrefabId>,
}

impl PrefabRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Low-level convenience wrapper around [`Self::try_register`] that panics
    /// on duplicate prefab names. Content should use `game-kit::PrefabAuthor`,
    /// which returns `Result`.
    pub fn register(
        &mut self,
        name: impl Into<String>,
        spawn: impl Fn(&mut World, glam::Vec2, &PropertyBag) -> anyhow::Result<EntityId> + 'static,
    ) -> PrefabId {
        self.try_register(name, spawn)
            .expect("prefab names must be unique")
    }

    /// Registers a prefab under a unique `name`, erroring if the name is already
    /// registered. Silently aliasing a duplicate to the existing prefab would hide
    /// authoring mistakes (two different compositions under one name), so this is
    /// rejected rather than deduplicated.
    pub fn try_register(
        &mut self,
        name: impl Into<String>,
        spawn: impl Fn(&mut World, glam::Vec2, &PropertyBag) -> anyhow::Result<EntityId> + 'static,
    ) -> anyhow::Result<PrefabId> {
        let name = name.into();
        if self.names.contains_key(&name) {
            anyhow::bail!("duplicate prefab name '{name}'");
        }

        let id = PrefabId(self.prefabs.len() as u32);
        self.prefabs.insert(id, Prefab::new(spawn));
        self.names.insert(name, id);
        Ok(id)
    }

    pub fn id(&self, name: &str) -> Option<PrefabId> {
        self.names.get(name).copied()
    }

    pub fn contains(&self, id: PrefabId) -> bool {
        self.prefabs.contains_key(&id)
    }

    pub fn contains_name(&self, name: &str) -> bool {
        self.names.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.prefabs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.prefabs.is_empty()
    }

    pub fn spawn(
        &self,
        id: PrefabId,
        world: &mut World,
        position: glam::Vec2,
        properties: &PropertyBag,
    ) -> anyhow::Result<EntityId> {
        let prefab = self
            .prefabs
            .get(&id)
            .ok_or_else(|| anyhow::anyhow!("unknown prefab id {:?}", id))?;
        (prefab.spawn)(world, position, properties)
    }
}

struct ComponentRequirement {
    prefab_name: String,
    component_type: TypeId,
    component_name: &'static str,
}

pub struct PrefabValidator<'a> {
    registry: &'a PrefabRegistry,
    requirements: Vec<ComponentRequirement>,
}

impl<'a> PrefabValidator<'a> {
    pub fn new(registry: &'a PrefabRegistry) -> Self {
        Self {
            registry,
            requirements: Vec::new(),
        }
    }

    pub fn require_component<T: Component>(&mut self, prefab_name: impl Into<String>) -> &mut Self {
        self.requirements.push(ComponentRequirement {
            prefab_name: prefab_name.into(),
            component_type: TypeId::of::<T>(),
            component_name: std::any::type_name::<T>(),
        });
        self
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        // Group every requirement by prefab so each prefab is spawned exactly once
        // and all of its required components are checked against that single
        // entity. Spawning per-requirement would both waste work and mask a prefab
        // whose composition varies between spawns. `BTreeMap` keeps the spawn and
        // error order deterministic.
        let mut by_prefab: BTreeMap<&str, Vec<&ComponentRequirement>> = BTreeMap::new();
        for requirement in &self.requirements {
            by_prefab
                .entry(requirement.prefab_name.as_str())
                .or_default()
                .push(requirement);
        }

        for (prefab_name, requirements) in by_prefab {
            let prefab_id = self
                .registry
                .id(prefab_name)
                .ok_or_else(|| anyhow::anyhow!("unknown prefab '{}'", prefab_name))?;
            let mut world = World::new();
            let entity = self.registry.spawn(
                prefab_id,
                &mut world,
                glam::Vec2::ZERO,
                &PropertyBag::default(),
            )?;
            for requirement in requirements {
                if !world.has_component_type(entity, requirement.component_type) {
                    let component = short_type_name(requirement.component_name);
                    anyhow::bail!(
                        "Prefab '{}' is missing {}.\n\nThis usually means the prefab did not add one of the components required for this kind of object.\n\nIf using the beginner API, this is probably a bug in game-kit.\nIf using the advanced tuple API, include the missing component inside the prefab spawn bundle, for example:\n\n    {}",
                        prefab_name,
                        component,
                        example_component_insert(component)
                    );
                }
            }
        }
        Ok(())
    }
}

fn short_type_name(name: &str) -> &str {
    name.rsplit("::").next().unwrap_or(name)
}

fn example_component_insert(component: &str) -> &'static str {
    match component {
        "Transform" => "Transform::at(at)",
        "Velocity" => "Velocity::default()",
        "Sprite" => "Sprite::new(assets.player, vec2s(20.0))",
        "Collider" => "Collider::box_of(vec2s(20.0))",
        "Health" => "Health::new(100)",
        "Faction" => "Faction::player()",
        _ => "<the missing component>",
    }
}

/// Low-level engine builder used by `game-kit` and the runtime.
///
/// Content crates should prefer `game_kit::GameApp`, which wraps this type with
/// asset, input, prefab, map, and system authoring APIs.
#[derive(Default)]
pub struct GameBuilder {
    assets: AssetRegistry,
    input: InputRegistry,
    maps: MapRegistry,
    prefabs: Rc<PrefabRegistry>,
    schedule: Schedule,
    start_map: Option<MapId>,
}

impl GameBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn assets(&self) -> &AssetRegistry {
        &self.assets
    }

    pub fn assets_mut(&mut self) -> &mut AssetRegistry {
        &mut self.assets
    }

    pub fn input(&self) -> &InputRegistry {
        &self.input
    }

    pub fn input_mut(&mut self) -> &mut InputRegistry {
        &mut self.input
    }

    pub fn maps(&self) -> &MapRegistry {
        &self.maps
    }

    pub fn maps_mut(&mut self) -> &mut MapRegistry {
        &mut self.maps
    }

    pub fn prefabs(&self) -> &PrefabRegistry {
        &self.prefabs
    }

    pub fn prefabs_mut(&mut self) -> &mut PrefabRegistry {
        Rc::get_mut(&mut self.prefabs).expect("prefab registry is already shared")
    }

    pub fn prefabs_shared(&self) -> Rc<PrefabRegistry> {
        Rc::clone(&self.prefabs)
    }

    pub fn schedule(&self) -> &Schedule {
        &self.schedule
    }

    pub fn schedule_mut(&mut self) -> &mut Schedule {
        &mut self.schedule
    }

    pub fn set_start_map(&mut self, id: MapId) {
        self.start_map = Some(id);
    }

    pub fn start_map(&self) -> Option<MapId> {
        self.start_map
    }

    pub fn into_schedule(self) -> Schedule {
        self.schedule
    }

    /// Hands the runtime the finalized content registries it consumes as the
    /// source of truth. Maps/prefabs stay owned by runtime content instead of
    /// being merely parallel data captured by schedule closures.
    pub fn into_parts(self) -> anyhow::Result<RuntimeContent> {
        let start_map = self
            .start_map
            .ok_or_else(|| anyhow::anyhow!("runtime content has no start map"))?;
        Ok(RuntimeContent {
            assets: self.assets,
            input: self.input,
            maps: self.maps,
            prefabs: self.prefabs,
            start_map,
            schedule: self.schedule,
        })
    }
}

/// Finalized content handed to the runtime after `GamePlugin::build`.
pub struct RuntimeContent {
    pub assets: AssetRegistry,
    pub input: InputRegistry,
    pub maps: MapRegistry,
    pub prefabs: Rc<PrefabRegistry>,
    pub start_map: MapId,
    pub schedule: Schedule,
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use crate::backend::TextureHandle;
    use crate::world::{Entity, Sprite, Transform, Velocity};

    use super::{MapRegistry, PrefabRegistry, PrefabValidator};

    fn theme() -> crate::app::TileTheme {
        crate::app::TileTheme {
            floor: Sprite::new(TextureHandle(0), glam::Vec2::ONE),
            wall: Sprite::new(TextureHandle(1), glam::Vec2::ONE),
        }
    }

    #[test]
    fn try_register_rejects_duplicate_names() {
        let mut registry = PrefabRegistry::new();
        registry
            .try_register("thing", |world, pos, _| Ok(world.spawn(Entity::new(pos))))
            .unwrap();

        let err = registry
            .try_register("thing", |world, pos, _| Ok(world.spawn(Entity::new(pos))))
            .unwrap_err();

        assert!(err.to_string().contains("duplicate prefab name 'thing'"));
    }

    #[test]
    fn map_registry_rejects_duplicate_names() {
        let mut registry = MapRegistry::new();
        let map = crate::tilemap::TileMap::try_from_rows(&["."], 16.0).unwrap();
        registry.register("arena", map.clone(), theme());
        assert_eq!(registry.id("arena"), Some(super::MapId(0)));

        let err = registry.try_register("arena", map, theme()).unwrap_err();

        assert!(err.to_string().contains("duplicate map name 'arena'"));
    }

    #[test]
    fn validator_spawns_each_prefab_once_for_all_requirements() {
        let spawns = Rc::new(Cell::new(0_usize));
        let counter = Rc::clone(&spawns);
        let mut registry = PrefabRegistry::new();
        registry.register("thing", move |world, pos, _| {
            counter.set(counter.get() + 1);
            Ok(world.spawn(Entity::new(pos).with(1_i32)))
        });

        let mut validator = PrefabValidator::new(&registry);
        validator
            .require_component::<Transform>("thing")
            .require_component::<Velocity>("thing")
            .require_component::<i32>("thing");
        validator.validate().unwrap();

        assert_eq!(
            spawns.get(),
            1,
            "three requirements on one prefab must spawn it exactly once"
        );
    }

    #[test]
    fn validator_reports_missing_component() {
        let mut registry = PrefabRegistry::new();
        registry.register("bare", |world, pos, _| {
            Ok(world.spawn(Entity::empty().with(Transform::at(pos))))
        });

        let mut validator = PrefabValidator::new(&registry);
        validator
            .require_component::<Transform>("bare")
            .require_component::<i32>("bare");

        let err = validator.validate().unwrap_err();
        let message = err.to_string();
        assert!(message.contains("Prefab 'bare' is missing i32."));
        assert!(message.contains("advanced tuple API"));
        assert!(message.contains("<the missing component>"));
    }
}
