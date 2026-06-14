use glam::{Vec2, Vec4};

use crate::engine::gfx::SpriteHandle;

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub pos: Vec2,
}

impl Transform {
    pub fn at(pos: Vec2) -> Self {
        Self { pos }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Sprite {
    pub handle: SpriteHandle,
    pub size: Vec2,
    pub layer: i16,
    pub color: Vec4,
}

impl Sprite {
    pub fn new(handle: SpriteHandle, size: Vec2) -> Self {
        Self {
            handle,
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

#[derive(Clone, Copy, Debug)]
pub struct Collider {
    pub half_extents: Vec2,
}

impl Collider {
    pub fn box_of(size: Vec2) -> Self {
        Self {
            half_extents: size * 0.5,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Entity<U> {
    pub transform: Transform,
    pub velocity: Vec2,
    pub sprite: Option<Sprite>,
    pub collider: Option<Collider>,
    pub user: U,
}

impl<U> Entity<U> {
    pub fn new(pos: Vec2, user: U) -> Self {
        Self {
            transform: Transform::at(pos),
            velocity: Vec2::ZERO,
            sprite: None,
            collider: None,
            user,
        }
    }

    pub fn with_sprite(mut self, sprite: Sprite) -> Self {
        self.sprite = Some(sprite);
        self
    }

    pub fn with_collider(mut self, collider: Collider) -> Self {
        self.collider = Some(collider);
        self
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct EntityId {
    index: u32,
    generation: u32,
}

struct Slot<U> {
    generation: u32,
    entity: Option<Entity<U>>,
}

pub struct World<U> {
    slots: Vec<Slot<U>>,
    free: Vec<u32>,
}

impl<U> World<U> {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            free: Vec::new(),
        }
    }

    pub fn spawn(&mut self, entity: Entity<U>) -> EntityId {
        if let Some(index) = self.free.pop() {
            let slot = &mut self.slots[index as usize];
            slot.entity = Some(entity);
            EntityId {
                index,
                generation: slot.generation,
            }
        } else {
            let index = self.slots.len() as u32;
            self.slots.push(Slot {
                generation: 0,
                entity: Some(entity),
            });
            EntityId {
                index,
                generation: 0,
            }
        }
    }

    pub fn despawn(&mut self, id: EntityId) {
        if let Some(slot) = self.slots.get_mut(id.index as usize) {
            if slot.generation == id.generation && slot.entity.is_some() {
                slot.entity = None;
                slot.generation = slot.generation.wrapping_add(1);
                self.free.push(id.index);
            }
        }
    }

    pub fn clear(&mut self) {
        self.slots.clear();
        self.free.clear();
    }

    #[allow(dead_code)]
    pub fn get(&self, id: EntityId) -> Option<&Entity<U>> {
        let slot = self.slots.get(id.index as usize)?;
        (slot.generation == id.generation)
            .then_some(slot.entity.as_ref())
            .flatten()
    }

    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut Entity<U>> {
        let slot = self.slots.get_mut(id.index as usize)?;
        if slot.generation != id.generation {
            return None;
        }
        slot.entity.as_mut()
    }

    pub fn iter(&self) -> impl Iterator<Item = (EntityId, &Entity<U>)> {
        self.slots.iter().enumerate().filter_map(|(index, slot)| {
            slot.entity.as_ref().map(|entity| {
                (
                    EntityId {
                        index: index as u32,
                        generation: slot.generation,
                    },
                    entity,
                )
            })
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (EntityId, &mut Entity<U>)> {
        self.slots
            .iter_mut()
            .enumerate()
            .filter_map(|(index, slot)| {
                slot.entity.as_mut().map(|entity| {
                    (
                        EntityId {
                            index: index as u32,
                            generation: slot.generation,
                        },
                        entity,
                    )
                })
            })
    }

    pub fn ids_where(&self, pred: impl Fn(&Entity<U>) -> bool) -> Vec<EntityId> {
        self.iter()
            .filter(|(_, entity)| pred(entity))
            .map(|(id, _)| id)
            .collect()
    }
}

impl<U> Default for World<U> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{Entity, World};

    #[test]
    fn despawn_invalidates_old_ids() {
        let mut world = World::new();
        let first = world.spawn(Entity::new(glam::Vec2::ZERO, 1));
        assert_eq!(world.get(first).map(|entity| entity.user), Some(1));

        world.despawn(first);
        assert!(world.get(first).is_none());

        let second = world.spawn(Entity::new(glam::Vec2::ZERO, 2));
        assert_ne!(first, second);
        assert_eq!(world.get(second).map(|entity| entity.user), Some(2));
        assert!(world.get(first).is_none());
    }

    #[test]
    fn ids_where_collects_matching_live_entities() {
        let mut world = World::new();
        world.spawn(Entity::new(glam::Vec2::ZERO, 1));
        let even = world.spawn(Entity::new(glam::Vec2::ZERO, 2));

        assert_eq!(world.ids_where(|entity| entity.user % 2 == 0), vec![even]);
    }
}
