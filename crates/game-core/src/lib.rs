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
pub mod schedule;
pub mod tilemap;
pub mod world;

#[allow(unused_imports)]
pub mod prelude {
    pub use crate::app::{
        Ctx, Game, MapData, RenderFrame, StartCtx, TileTheme, extract_entity_sprites,
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
    pub use crate::gfx::{Gfx, SpriteDraw, TextDraw};
    pub use crate::input::{
        ActionBinding, ActionId, Axis2dBinding, Axis2dId, Input, InputRegistry, Key,
    };
    pub use crate::nav::NavGrid;
    pub use crate::plugin::GamePlugin;
    pub use crate::schedule::{Schedule, ScheduleValidator, StartupSystem, System};
    pub use crate::tilemap::{Tile, TileMap};
    pub use crate::world::{
        Component, ComponentStore, Entity, EntityId, Sprite, Transform, Velocity, World,
    };
}

// TEMP: compatibility modules for the Phase 2 physical split. Most source files
// still use their old single-crate paths; later phases remove these re-exports.
pub mod engine {
    pub use crate::{
        app, assets, audio, backend, builder, camera, commands, gfx, input, nav, plugin, schedule,
        tilemap, world,
    };

    #[allow(unused_imports)]
    pub mod prelude {
        pub use crate::prelude::*;
    }
}

#[allow(unused_imports)]
pub use app::{
    Ctx, Game, MapData, RenderFrame, StartCtx, TileTheme, extract_entity_sprites,
    extract_tilemap_sprites,
};
#[allow(unused_imports)]
pub use assets::{AssetRegistry, AssetValidator};
#[allow(unused_imports)]
pub use audio::{Audio, AudioCommands};
#[allow(unused_imports)]
pub use backend::{
    AudioBackend, AudioCommand, FontHandle, FontLoadRequest, PlatformBackend, PlatformEvents,
    RenderBackend, RenderOutcome, SoundHandle, SoundLoadRequest, TextureHandle, TextureLoadRequest,
};
#[allow(unused_imports)]
pub use builder::{
    GameBuilder, MapId, MapRegistry, Prefab, PrefabId, PrefabRegistry, PrefabValidator,
    PropertyBag, RegisteredMap,
};
#[allow(unused_imports)]
pub use camera::Camera2D;
#[allow(unused_imports)]
pub use commands::{Command, CommandQueue};
#[allow(unused_imports)]
pub use gfx::{Gfx, SpriteDraw, TextDraw};
#[allow(unused_imports)]
pub use input::{ActionBinding, ActionId, Axis2dBinding, Axis2dId, Input, InputRegistry, Key};
#[allow(unused_imports)]
pub use nav::NavGrid;
#[allow(unused_imports)]
pub use plugin::GamePlugin;
#[allow(unused_imports)]
pub use schedule::{Schedule, ScheduleValidator, StartupSystem, System};
#[allow(unused_imports)]
pub use tilemap::{Tile, TileMap};
#[allow(unused_imports)]
pub use world::{Component, ComponentStore, Entity, EntityId, Sprite, Transform, Velocity, World};
