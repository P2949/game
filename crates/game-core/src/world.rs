use std::any::{Any, TypeId};
use std::collections::HashMap;

use glam::{Vec2, Vec4};

use crate::backend::TextureHandle;

pub trait Component: 'static {}

impl<T: 'static> Component for T {}

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub pos: Vec2,
}

impl Transform {
    pub fn at(pos: Vec2) -> Self {
        Self { pos }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Velocity(pub Vec2);

impl Velocity {
    pub fn new(value: Vec2) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Sprite {
    pub texture: TextureHandle,
    pub size: Vec2,
    pub layer: i16,
    pub color: Vec4,
}

impl Sprite {
    pub fn new(texture: TextureHandle, size: Vec2) -> Self {
        Self {
            texture,
            size,
            layer: 0,
            color: Vec4::ONE,
        }
    }

    pub fn layer(mut self, layer: i16) -> Self {
        self.layer = layer;
        self
    }

    pub fn tint(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }
}

pub struct Entity {
    components: Vec<Box<dyn PendingComponent>>,
}

impl Entity {
    pub fn empty() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    pub fn new(pos: Vec2) -> Self {
        Self::empty()
            .with(Transform::at(pos))
            .with(Velocity::default())
    }

    pub fn with<T: Component>(mut self, component: T) -> Self {
        self.components.push(Box::new(Pending(component)));
        self
    }

    pub fn with_sprite(self, sprite: Sprite) -> Self {
        self.with(sprite)
    }

    pub fn with_collider<T: Component>(self, collider: T) -> Self {
        self.with(collider)
    }

    fn insert_into(self, id: EntityId, world: &mut World) {
        for component in self.components {
            component.insert(id, world);
        }
    }
}

impl Default for Entity {
    fn default() -> Self {
        Self::empty()
    }
}

trait PendingComponent {
    fn insert(self: Box<Self>, id: EntityId, world: &mut World);
}

struct Pending<T: Component>(T);

impl<T: Component> PendingComponent for Pending<T> {
    fn insert(self: Box<Self>, id: EntityId, world: &mut World) {
        world.insert(id, self.0);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct EntityId {
    index: u32,
    generation: u32,
}

struct Slot {
    generation: u32,
    alive: bool,
}

pub struct ComponentStore<T: Component> {
    pub entries: HashMap<EntityId, T>,
}

impl<T: Component> Default for ComponentStore<T> {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
}

trait ErasedComponentStore {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn contains_entity(&self, id: EntityId) -> bool;
    fn remove_entity(&mut self, id: EntityId);
    fn clear(&mut self);
}

impl<T: Component> ErasedComponentStore for ComponentStore<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn contains_entity(&self, id: EntityId) -> bool {
        self.entries.contains_key(&id)
    }

    fn remove_entity(&mut self, id: EntityId) {
        self.entries.remove(&id);
    }

    fn clear(&mut self) {
        self.entries.clear();
    }
}

#[derive(Default)]
struct ComponentStores {
    stores: HashMap<TypeId, Box<dyn ErasedComponentStore>>,
}

impl ComponentStores {
    fn store<T: Component>(&self) -> Option<&ComponentStore<T>> {
        self.stores
            .get(&TypeId::of::<T>())
            .and_then(|store| store.as_any().downcast_ref())
    }

    fn store_mut<T: Component>(&mut self) -> &mut ComponentStore<T> {
        self.stores
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(ComponentStore::<T>::default()))
            .as_any_mut()
            .downcast_mut()
            .expect("component store TypeId must match stored component type")
    }

    fn remove_entity(&mut self, id: EntityId) {
        for store in self.stores.values_mut() {
            store.remove_entity(id);
        }
    }

    fn clear(&mut self) {
        for store in self.stores.values_mut() {
            store.clear();
        }
    }
}

#[derive(Default)]
pub struct World {
    slots: Vec<Slot>,
    free: Vec<u32>,
    components: ComponentStores,
    resources: HashMap<TypeId, Box<dyn Any>>,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, entity: Entity) -> EntityId {
        let id = self.spawn_empty();
        entity.insert_into(id, self);
        id
    }

    pub fn spawn_empty(&mut self) -> EntityId {
        if let Some(index) = self.free.pop() {
            let slot = &mut self.slots[index as usize];
            slot.alive = true;
            EntityId {
                index,
                generation: slot.generation,
            }
        } else {
            let index = self.slots.len() as u32;
            self.slots.push(Slot {
                generation: 0,
                alive: true,
            });
            EntityId {
                index,
                generation: 0,
            }
        }
    }

    pub fn despawn(&mut self, id: EntityId) {
        if let Some(slot) = self.slots.get_mut(id.index as usize) {
            if slot.generation == id.generation && slot.alive {
                slot.alive = false;
                slot.generation = slot.generation.wrapping_add(1);
                self.components.remove_entity(id);
                self.free.push(id.index);
            }
        }
    }

    pub fn clear(&mut self) {
        self.slots.clear();
        self.free.clear();
        self.components.clear();
    }

    pub fn insert_resource<T: 'static>(&mut self, resource: T) -> Option<T> {
        self.resources
            .insert(TypeId::of::<T>(), Box::new(resource))
            .and_then(|previous| previous.downcast::<T>().ok().map(|resource| *resource))
    }

    pub fn get_resource<T: 'static>(&self) -> Option<&T> {
        self.resources
            .get(&TypeId::of::<T>())
            .and_then(|resource| resource.downcast_ref())
    }

    pub fn get_resource_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.resources
            .get_mut(&TypeId::of::<T>())
            .and_then(|resource| resource.downcast_mut())
    }

    pub fn resource_or_insert_with<T: 'static>(&mut self, create: impl FnOnce() -> T) -> &mut T {
        self.resources
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(create()))
            .downcast_mut()
            .expect("resource TypeId must match stored resource type")
    }

    pub fn remove_resource<T: 'static>(&mut self) -> Option<T> {
        self.resources
            .remove(&TypeId::of::<T>())
            .and_then(|resource| resource.downcast::<T>().ok().map(|resource| *resource))
    }

    pub fn clear_resources(&mut self) {
        self.resources.clear();
    }

    pub fn insert<T: Component>(&mut self, id: EntityId, component: T) -> Option<T> {
        if !self.is_alive(id) {
            return None;
        }
        self.components
            .store_mut::<T>()
            .entries
            .insert(id, component)
    }

    pub fn get<T: Component>(&self, id: EntityId) -> Option<&T> {
        self.is_alive(id)
            .then(|| self.components.store::<T>()?.entries.get(&id))
            .flatten()
    }

    pub fn get_mut<T: Component>(&mut self, id: EntityId) -> Option<&mut T> {
        if !self.is_alive(id) {
            return None;
        }
        self.components.store_mut::<T>().entries.get_mut(&id)
    }

    pub fn remove<T: Component>(&mut self, id: EntityId) -> Option<T> {
        if !self.is_alive(id) {
            return None;
        }
        self.components.store_mut::<T>().entries.remove(&id)
    }

    pub fn has<T: Component>(&self, id: EntityId) -> bool {
        self.get::<T>(id).is_some()
    }

    pub fn has_component_type(&self, id: EntityId, type_id: TypeId) -> bool {
        self.is_alive(id)
            && self
                .components
                .stores
                .get(&type_id)
                .is_some_and(|store| store.contains_entity(id))
    }

    pub fn ids(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.slots.iter().enumerate().filter_map(|(index, slot)| {
            slot.alive.then_some(EntityId {
                index: index as u32,
                generation: slot.generation,
            })
        })
    }

    pub fn ids_with<T: Component>(&self) -> Vec<EntityId> {
        self.components
            .store::<T>()
            .map(|store| {
                store
                    .entries
                    .keys()
                    .copied()
                    .filter(|id| self.is_alive(*id))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn query<T: Component>(&self) -> Vec<(EntityId, &T)> {
        self.components
            .store::<T>()
            .map(|store| {
                store
                    .entries
                    .iter()
                    .filter(|(id, _)| self.is_alive(**id))
                    .map(|(id, component)| (*id, component))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn query2<A: Component, B: Component>(&self) -> Vec<(EntityId, &A, &B)> {
        self.ids_with::<A>()
            .into_iter()
            .filter_map(|id| Some((id, self.get::<A>(id)?, self.get::<B>(id)?)))
            .collect()
    }

    fn is_alive(&self, id: EntityId) -> bool {
        self.slots
            .get(id.index as usize)
            .is_some_and(|slot| slot.alive && slot.generation == id.generation)
    }
}

#[cfg(test)]
mod tests {
    use super::{Entity, Transform, Velocity, World};

    #[test]
    fn despawn_invalidates_old_ids() {
        let mut world = World::new();
        let first = world.spawn(Entity::new(glam::Vec2::ZERO).with(1_i32));
        assert_eq!(world.get::<i32>(first), Some(&1));

        world.despawn(first);
        assert!(world.get::<i32>(first).is_none());

        let second = world.spawn(Entity::new(glam::Vec2::ZERO).with(2_i32));
        assert_ne!(first, second);
        assert_eq!(world.get::<i32>(second), Some(&2));
        assert!(world.get::<i32>(first).is_none());
    }

    #[test]
    fn ids_with_collects_matching_live_entities() {
        let mut world = World::new();
        world.spawn(Entity::new(glam::Vec2::ZERO));
        let even = world.spawn(Entity::new(glam::Vec2::ZERO).with(2_i32));

        assert_eq!(world.ids_with::<i32>(), vec![even]);
    }

    #[test]
    fn built_in_components_are_inserted_by_entity_constructor() {
        let mut world = World::new();
        let id = world.spawn(Entity::new(glam::vec2(3.0, 4.0)));

        assert_eq!(
            world.get::<Transform>(id).unwrap().pos,
            glam::vec2(3.0, 4.0)
        );
        assert_eq!(world.get::<Velocity>(id).unwrap().0, glam::Vec2::ZERO);
    }

    #[test]
    fn resources_are_typed_and_survive_entity_clear() {
        let mut world = World::new();
        world.insert_resource(String::from("arena"));
        world.spawn(Entity::new(glam::Vec2::ZERO));

        world.clear();

        assert_eq!(world.ids().count(), 0);
        assert_eq!(world.get_resource::<String>().unwrap(), "arena");
        assert_eq!(
            world.insert_resource(String::from("next")),
            Some(String::from("arena"))
        );
        assert_eq!(
            world.remove_resource::<String>(),
            Some(String::from("next"))
        );
    }
}
