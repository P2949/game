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

/// Requests that file-backed runtime assets be reloaded after the current
/// gameplay step. The concrete runtime owns decoding and backend replacement.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AssetReloadRequest;

/// Last asset reload result, kept in core so runtime diagnostics can be shown
/// by a content-facing debug overlay without a runtime -> game-kit dependency.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetReloadStatus {
    pub message: String,
}

impl AssetReloadStatus {
    pub fn queued() -> Self {
        Self {
            message: "queued".to_owned(),
        }
    }

    pub fn succeeded(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn failed(message: impl Into<String>) -> Self {
        Self {
            message: format!("failed: {}", message.into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandErrorKind {
    SpawnPrefab,
    ChangeMap,
    ReloadMap,
    ReloadAssets,
    RestartMap,
    RestartStartMap,
    AuthoringSpawn,
    MapTransition,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandError {
    pub kind: CommandErrorKind,
    pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommandErrors {
    errors: Vec<CommandError>,
}

impl CommandErrors {
    pub fn push(&mut self, kind: CommandErrorKind, message: impl Into<String>) {
        self.errors.push(CommandError {
            kind,
            message: message.into(),
        });
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &CommandError> {
        self.errors.iter()
    }

    pub fn clear(&mut self) {
        self.errors.clear();
    }

    pub fn last(&self) -> Option<&CommandError> {
        self.errors.last()
    }
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
    ReloadAssets,
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

    pub fn reload_assets(&mut self) {
        self.push(Command::ReloadAssets);
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
    use crate::commands::{Command, CommandErrorKind, CommandErrors, CommandQueue};

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
        commands.reload_assets();
        commands.restart_map();
        commands.restart_start_map();

        assert_eq!(
            commands.drain().collect::<Vec<_>>(),
            vec![
                Command::ChangeMap(MapId(2)),
                Command::Quit,
                Command::ReloadMap(MapId(2)),
                Command::ReloadAssets,
                Command::RestartMap,
                Command::RestartStartMap,
            ]
        );
    }

    #[test]
    fn command_errors_keep_order_and_last_error() {
        let mut errors = CommandErrors::default();
        assert!(errors.is_empty());

        errors.push(CommandErrorKind::ChangeMap, "unknown map id MapId(7)");
        errors.push(
            CommandErrorKind::SpawnPrefab,
            "unknown prefab id PrefabId(3)",
        );

        assert_eq!(errors.len(), 2);
        assert_eq!(
            errors.iter().map(|error| &error.kind).collect::<Vec<_>>(),
            vec![&CommandErrorKind::ChangeMap, &CommandErrorKind::SpawnPrefab]
        );
        assert_eq!(
            errors.last().map(|error| error.message.as_str()),
            Some("unknown prefab id PrefabId(3)")
        );

        errors.clear();
        assert!(errors.is_empty());
    }
}
