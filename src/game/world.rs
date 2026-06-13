#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId {
    index: u32,
    generation: u32,
}

pub struct Slot<T> {
    generation: u32,
    value: Option<T>,
}

pub struct EntityStore<T> {
    slots: Vec<Slot<T>>,
    free: Vec<u32>,
}

impl<T> EntityStore<T> {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            free: Vec::new(),
        }
    }

    pub fn spawn(&mut self, value: T) -> EntityId {
        if let Some(index) = self.free.pop() {
            let slot = &mut self.slots[index as usize];
            slot.value = Some(value);
            return EntityId {
                index,
                generation: slot.generation,
            };
        }

        let index = self.slots.len() as u32;
        self.slots.push(Slot {
            generation: 0,
            value: Some(value),
        });

        EntityId {
            index,
            generation: 0,
        }
    }

    pub fn despawn(&mut self, id: EntityId) -> Option<T> {
        let slot = self.slots.get_mut(id.index as usize)?;
        if slot.generation != id.generation {
            return None;
        }

        let value = slot.value.take()?;
        slot.generation = slot.generation.wrapping_add(1);
        self.free.push(id.index);
        Some(value)
    }

    pub fn get(&self, id: EntityId) -> Option<&T> {
        let slot = self.slots.get(id.index as usize)?;
        if slot.generation != id.generation {
            return None;
        }
        slot.value.as_ref()
    }

    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut T> {
        let slot = self.slots.get_mut(id.index as usize)?;
        if slot.generation != id.generation {
            return None;
        }
        slot.value.as_mut()
    }

    pub fn iter(&self) -> impl Iterator<Item = (EntityId, &T)> {
        self.slots.iter().enumerate().filter_map(|(index, slot)| {
            slot.value.as_ref().map(|value| {
                (
                    EntityId {
                        index: index as u32,
                        generation: slot.generation,
                    },
                    value,
                )
            })
        })
    }
}

impl<T> Default for EntityStore<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Entity {
    pub pos: glam::Vec2,
    pub prev_pos: glam::Vec2,
    pub vel: glam::Vec2,
    pub size: glam::Vec2,
    pub sprite: crate::renderer::TextureId,
    pub solid: bool,
}

impl Entity {
    pub fn interpolated_pos(&self, alpha: f32) -> glam::Vec2 {
        self.prev_pos.lerp(self.pos, alpha)
    }
}

#[cfg(test)]
mod tests {
    use super::EntityStore;

    #[test]
    fn despawned_entity_id_does_not_access_reused_slot() {
        let mut store = EntityStore::new();
        let a = store.spawn("first");
        assert_eq!(store.get(a), Some(&"first"));
        store.despawn(a);

        let b = store.spawn("second");
        assert_ne!(a, b);
        assert_eq!(store.get(a), None);
        assert_eq!(store.get(b), Some(&"second"));
    }
}
