//! Input authoring (Phase 3).
//!
//! [`InputAuthor`] names logical controls (actions and 2D axes) and binds them to
//! keys, without exposing the engine's `InputRegistry`. Reached through
//! [`GameApp::input`].

use anyhow::Result;
use game_core::input::{
    ActionBindingBuilder, ActionId, Axis2dBindingBuilder, Axis2dId, InputRegistry, Key,
};

/// Declares the logical controls a game uses.
pub struct InputAuthor<'a> {
    registry: &'a mut InputRegistry,
}

impl<'a> InputAuthor<'a> {
    pub(crate) fn new(registry: &'a mut InputRegistry) -> Self {
        Self { registry }
    }

    /// Begins declaring a single action (e.g. `"attack"`), then bind keys with
    /// [`ActionAuthor::key`]/[`ActionAuthor::keys`].
    pub fn action(&mut self, name: impl Into<String>) -> Result<ActionAuthor<'_>> {
        Ok(ActionAuthor {
            builder: self.registry.try_action(name)?,
        })
    }

    /// Begins declaring a 2D movement axis (e.g. `"move"`), then bind directions
    /// with [`Axis2dAuthor::wasd`]/[`Axis2dAuthor::arrows`]/[`Axis2dAuthor::keys`].
    pub fn axis2d(&mut self, name: impl Into<String>) -> Result<Axis2dAuthor<'_>> {
        Ok(Axis2dAuthor {
            builder: self.registry.try_axis2d(name)?,
        })
    }
}

/// Binds keys to one logical action.
pub struct ActionAuthor<'a> {
    builder: ActionBindingBuilder<'a>,
}

impl ActionAuthor<'_> {
    /// Binds a single key and finalizes the action.
    pub fn key(self, key: Key) -> ActionId {
        self.builder.bind(key).id()
    }

    /// Binds several keys to the action and finalizes it.
    pub fn keys<const N: usize>(self, keys: [Key; N]) -> ActionId {
        let mut builder = self.builder;
        for key in keys {
            builder = builder.bind(key);
        }
        builder.id()
    }
}

/// Binds directional keys to one logical 2D axis.
pub struct Axis2dAuthor<'a> {
    builder: Axis2dBindingBuilder<'a>,
}

impl<'a> Axis2dAuthor<'a> {
    /// Adds W/A/S/D directional bindings (returns `self` so arrows can be added
    /// too: `axis.wasd().arrows()`).
    pub fn wasd(self) -> Self {
        Self {
            builder: self
                .builder
                .negative_x(Key::A)
                .positive_x(Key::D)
                .negative_y(Key::W)
                .positive_y(Key::S),
        }
    }

    /// Adds arrow-key directional bindings and finalizes the axis.
    pub fn arrows(self) -> Axis2dId {
        self.builder
            .negative_x(Key::Left)
            .positive_x(Key::Right)
            .negative_y(Key::Up)
            .positive_y(Key::Down)
            .id()
    }

    /// Adds explicit directional bindings and finalizes the axis.
    pub fn keys(self, left: Key, right: Key, up: Key, down: Key) -> Axis2dId {
        self.builder
            .negative_x(left)
            .positive_x(right)
            .negative_y(up)
            .positive_y(down)
            .id()
    }
}
