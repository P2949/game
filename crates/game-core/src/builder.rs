use std::any::TypeId;
use std::collections::HashMap;

use crate::app::{MapData, TileTheme};
use crate::assets::AssetRegistry;
use crate::input::InputRegistry;
use crate::nav::NavGrid;
use crate::schedule::Schedule;
use crate::tilemap::TileMap;
use crate::world::{Component, EntityId, World};
pub use game_map::{MapId, PrefabId, PropertyBag};

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

    pub fn register(
        &mut self,
        name: impl Into<String>,
        tilemap: TileMap,
        theme: TileTheme,
    ) -> MapId {
        let id = MapId(self.maps.len() as u32);
        let nav = NavGrid::from_tilemap(&tilemap);
        self.maps.push(RegisteredMap {
            name: name.into(),
            data: MapData {
                tilemap,
                nav,
                theme,
            },
        });
        id
    }

    pub fn get(&self, id: MapId) -> Option<&RegisteredMap> {
        self.maps.get(id.0 as usize)
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

    pub fn register(
        &mut self,
        name: impl Into<String>,
        spawn: impl Fn(&mut World, glam::Vec2, &PropertyBag) -> anyhow::Result<EntityId> + 'static,
    ) -> PrefabId {
        let name = name.into();
        if let Some(id) = self.names.get(&name) {
            return *id;
        }

        let id = PrefabId(self.prefabs.len() as u32);
        self.prefabs.insert(id, Prefab::new(spawn));
        self.names.insert(name, id);
        id
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
        for requirement in &self.requirements {
            let prefab_id = self
                .registry
                .id(&requirement.prefab_name)
                .ok_or_else(|| anyhow::anyhow!("unknown prefab '{}'", requirement.prefab_name))?;
            let mut world = World::new();
            let entity = self.registry.spawn(
                prefab_id,
                &mut world,
                glam::Vec2::ZERO,
                &PropertyBag::default(),
            )?;
            if !world.has_component_type(entity, requirement.component_type) {
                anyhow::bail!(
                    "prefab '{}' did not insert required component {}",
                    requirement.prefab_name,
                    requirement.component_name
                );
            }
        }
        Ok(())
    }

    pub fn validate_map_references(&self, map: &game_map::GameMap) -> anyhow::Result<()> {
        for object in &map.objects {
            if !self.registry.contains(object.prefab) {
                anyhow::bail!(
                    "map {:?} object '{}' references unknown prefab {:?}",
                    map.id,
                    object.id,
                    object.prefab
                );
            }
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct GameBuilder {
    assets: AssetRegistry,
    input: InputRegistry,
    maps: MapRegistry,
    prefabs: PrefabRegistry,
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
        &mut self.prefabs
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
}
