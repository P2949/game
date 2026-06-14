pub mod app;
pub mod assets;
pub mod audio;
pub mod camera;
pub mod gfx;
pub mod input;
pub mod nav;
pub mod physics;
pub mod tilemap;
pub mod world;

#[allow(unused_imports)]
pub mod prelude {
    pub use crate::engine::app::{Ctx, Game, StartCtx, TileTheme, run};
    pub use crate::engine::assets::Assets;
    pub use crate::engine::audio::Audio;
    pub use crate::engine::camera::Camera2D;
    pub use crate::engine::gfx::{Gfx, SpriteHandle};
    pub use crate::engine::input::{Action, Input};
    pub use crate::engine::nav::NavGrid;
    pub use crate::engine::tilemap::{Tile, TileMap};
    pub use crate::engine::world::{Collider, Entity, EntityId, Sprite, Transform, World};
}

#[allow(unused_imports)]
pub use app::{Ctx, Game, StartCtx, TileTheme, run};
#[allow(unused_imports)]
pub use assets::Assets;
#[allow(unused_imports)]
pub use audio::Audio;
#[allow(unused_imports)]
pub use camera::Camera2D;
#[allow(unused_imports)]
pub use gfx::{Gfx, SpriteHandle};
#[allow(unused_imports)]
pub use input::{Action, Input};
#[allow(unused_imports)]
pub use nav::NavGrid;
#[allow(unused_imports)]
pub use tilemap::{Tile, TileMap};
#[allow(unused_imports)]
pub use world::{Collider, Entity, EntityId, Sprite, Transform, World};
