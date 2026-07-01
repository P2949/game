pub mod app;
pub mod assets;
pub mod audio;
pub mod backend;
pub mod builder;
pub mod camera;
pub mod commands;
pub mod gfx;
pub mod input;
pub mod nav;
pub mod plugin;
pub mod query;
pub mod schedule;
pub mod tilemap;
pub mod world;

#[allow(unused_imports)]
pub mod prelude {
    //! Low-level engine-neutral essentials.
    //!
    //! Beginner content should import `game_kit::beginner::prelude::*` or
    //! `game_starter::prelude::*`; advanced content should import
    //! `game_kit::advanced::prelude::*`. Runtime/facade internals use
    //! [`crate::internal_prelude`] or explicit module paths.

    pub use crate::backend::{
        AudioCommand, FontHandle, FontLoadRequest, RenderOutcome, SoundHandle, SoundLoadRequest,
        TextureHandle, TextureLoadRequest,
    };
    pub use crate::camera::Camera2D;
    pub use crate::gfx::{Gfx, SpriteDraw, TextDraw, UiRect};
    pub use crate::input::{ActionBinding, ActionId, Axis2dBinding, Axis2dId, Input, Key};
    pub use crate::nav::NavGrid;
    pub use crate::query::{
        DeltaTime, ParamAccess, ParamSystem, Query, QueryData, Res, ResMut, SystemParam, With,
        Without,
    };
    pub use crate::tilemap::{Tile, TileMap};
    pub use crate::world::{Component, Entity, EntityId, Sprite, Transform, Velocity, World};
}

#[allow(unused_imports)]
pub mod internal_prelude {
    //! Engine/runtime/facade internals. Content crates must use
    //! `game_kit::beginner::prelude`, `game_kit::advanced::prelude`, or
    //! `game_starter::prelude` instead of this module.
    //!
    //! This intentionally includes raw contexts, registries, validators, schedules,
    //! and command queues. It is not the content authoring API.

    pub use crate::app::{
        Ctx, MapData, RenderFrame, StartCtx, TileTheme, extract_entity_sprites,
        extract_tilemap_sprites,
    };
    pub use crate::assets::{AssetRegistry, AssetValidator};
    pub use crate::audio::{Audio, AudioCommands};
    pub use crate::backend::{
        AudioBackend, AudioCommand, FontHandle, FontLoadRequest, PlatformBackend, PlatformEvents,
        RenderBackend, RenderOutcome, SoundHandle, SoundLoadRequest, TextureHandle,
        TextureLoadRequest,
    };
    pub use crate::builder::{
        GameBuilder, MapId, MapRegistry, Prefab, PrefabId, PrefabRegistry, PrefabValidator,
        PropertyBag, RegisteredMap,
    };
    pub use crate::camera::Camera2D;
    pub use crate::commands::{Command, CommandQueue};
    pub use crate::gfx::{Gfx, SpriteDraw, TextDraw, UiRect};
    pub use crate::input::{
        ActionBinding, ActionId, Axis2dBinding, Axis2dId, Input, InputRegistry, Key,
    };
    pub use crate::nav::NavGrid;
    pub use crate::plugin::GamePlugin;
    pub use crate::query::{
        DeltaTime, ParamAccess, ParamSystem, Query, QueryData, Res, ResMut, SystemParam, With,
        Without,
    };
    pub use crate::schedule::{Schedule, ScheduleValidator, StartupSystem, System};
    pub use crate::tilemap::{Tile, TileMap};
    pub use crate::world::{
        Component, ComponentStore, Entity, EntityId, Sprite, Transform, Velocity, World,
    };
}
