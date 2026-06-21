use glam::Vec2;

use crate::app::MapData;
use crate::backend::SoundHandle;
use crate::builder::{MapId, PrefabId, PropertyBag};
use crate::world::EntityId;

/// Replacement data for one map after content reparses a reloadable source.
/// The command itself only carries the stable map ID; the runtime consumes this
/// resource while handling that command.
pub struct MapReload {
    pub map: MapId,
    pub data: MapData,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Despawn(EntityId),
    PlaySound(SoundHandle),
    SpawnPrefab {
        prefab: PrefabId,
        position: Vec2,
        properties: PropertyBag,
    },
    ChangeMap(MapId),
    Quit,
    ReloadMap(MapId),
    RestartMap,
    RestartStartMap,
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

    pub fn despawn(&mut self, entity: EntityId) {
        self.push(Command::Despawn(entity));
    }

    pub fn play_sound(&mut self, sound: SoundHandle) {
        self.push(Command::PlaySound(sound));
    }

    pub fn spawn_prefab(&mut self, prefab: PrefabId, position: Vec2, properties: PropertyBag) {
        self.push(Command::SpawnPrefab {
            prefab,
            position,
            properties,
        });
    }

    pub fn change_map(&mut self, map: MapId) {
        self.push(Command::ChangeMap(map));
    }

    pub fn quit(&mut self) {
        self.push(Command::Quit);
    }

    pub fn reload_map(&mut self, map: MapId) {
        self.push(Command::ReloadMap(map));
    }

    pub fn restart_map(&mut self) {
        self.push(Command::RestartMap);
    }

    pub fn restart_start_map(&mut self) {
        self.push(Command::RestartStartMap);
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
    use crate::builder::{MapId, PrefabId, PropertyBag};
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

    #[test]
    fn command_queue_records_spawn_prefab() {
        let mut commands = CommandQueue::new();
        commands.spawn_prefab(PrefabId(3), glam::vec2(1.0, 2.0), PropertyBag::default());

        assert_eq!(
            commands.drain().collect::<Vec<_>>(),
            vec![Command::SpawnPrefab {
                prefab: PrefabId(3),
                position: glam::vec2(1.0, 2.0),
                properties: PropertyBag::default(),
            }]
        );
    }

    #[test]
    fn command_queue_records_map_flow_commands() {
        let mut commands = CommandQueue::new();
        commands.change_map(MapId(2));
        commands.quit();
        commands.reload_map(MapId(2));
        commands.restart_map();
        commands.restart_start_map();

        assert_eq!(
            commands.drain().collect::<Vec<_>>(),
            vec![
                Command::ChangeMap(MapId(2)),
                Command::Quit,
                Command::ReloadMap(MapId(2)),
                Command::RestartMap,
                Command::RestartStartMap,
            ]
        );
    }
}
