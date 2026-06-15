use crate::backend::SoundHandle;
use crate::builder::{MapId, PrefabId};
use crate::world::EntityId;

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Named(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Spawn(PrefabId, glam::Vec2),
    Despawn(EntityId),
    PlaySound(SoundHandle),
    SetMap(MapId),
    EmitEvent(Event),
}

#[derive(Default)]
pub struct CommandQueue {
    commands: Vec<Command>,
}

impl CommandQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, command: Command) {
        self.commands.push(command);
    }

    pub fn spawn(&mut self, prefab: PrefabId, position: glam::Vec2) {
        self.push(Command::Spawn(prefab, position));
    }

    pub fn despawn(&mut self, entity: EntityId) {
        self.push(Command::Despawn(entity));
    }

    pub fn play_sound(&mut self, sound: SoundHandle) {
        self.push(Command::PlaySound(sound));
    }

    pub fn set_map(&mut self, map: MapId) {
        self.push(Command::SetMap(map));
    }

    pub fn emit(&mut self, event: Event) {
        self.push(Command::EmitEvent(event));
    }

    pub fn drain(&mut self) -> impl Iterator<Item = Command> + '_ {
        self.commands.drain(..)
    }

    /// Drops every pending command without executing it. Used on world reset so
    /// commands enqueued against the pre-reset world cannot run against the new
    /// one.
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::backend::SoundHandle;
    use crate::commands::{Command, CommandQueue};

    #[test]
    fn command_queue_drains_in_order() {
        let mut commands = CommandQueue::new();
        commands.play_sound(SoundHandle(7));
        assert!(!commands.is_empty());

        assert_eq!(
            commands.drain().collect::<Vec<_>>(),
            vec![Command::PlaySound(SoundHandle(7))]
        );
        assert!(commands.is_empty());
    }
}
