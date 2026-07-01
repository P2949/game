use game_core::backend::TextureHandle;
use game_core::builder::GameBuilder;
use game_core::input::Key;
use game_core::world::{Sprite, Transform};
use game_map::cell;

use super::GameApp;
use crate::map::TileTheme;

fn test_theme() -> TileTheme {
    TileTheme {
        floor: Sprite::new(TextureHandle(1), glam::Vec2::splat(16.0)),
        wall: Sprite::new(TextureHandle(2), glam::Vec2::splat(16.0)),
    }
}

#[test]
fn duplicate_prefab_name_returns_error() {
    let mut builder = GameBuilder::new();
    let mut game = GameApp::new(&mut builder);

    game.prefab("duplicate", |prefab| {
        prefab.spawn(|at| (Transform::at(at),))?;
        Ok(())
    })
    .unwrap();

    let err = game
        .prefab("duplicate", |prefab| {
            prefab.spawn(|at| (Transform::at(at),))?;
            Ok(())
        })
        .unwrap_err();

    assert!(err.to_string().contains("duplicate prefab"));
}

#[test]
fn duplicate_input_action_returns_error() {
    let mut builder = GameBuilder::new();
    let mut game = GameApp::new(&mut builder);

    let err = game
        .input(|input| {
            input.action("pause")?.key(Key::P);
            input.action("pause")?.key(Key::R);
            Ok(())
        })
        .unwrap_err();

    assert!(err.to_string().contains("Duplicate input action"));
}

#[test]
fn conflicting_texture_key_returns_error() {
    let mut builder = GameBuilder::new();
    let mut game = GameApp::new(&mut builder);

    let err = game
        .assets(|assets| {
            assets.texture("hero", "textures/a.png")?;
            assets.texture("hero", "textures/b.png")?;
            Ok(())
        })
        .unwrap_err();

    assert!(err.to_string().contains("Texture asset key"));
}

#[test]
fn ron_map_rejects_in_code_authoring_calls() {
    let mut builder = GameBuilder::new();
    let mut game = GameApp::new(&mut builder);

    game.map_from_ron("")
        .tile_size(16.0)
        .tiles(["."])
        .spawn("player_start", "demo/player", cell(0, 0))
        .theme(test_theme())
        .start();

    let err = game.finish().unwrap_err();
    let message = err.to_string();

    assert!(message.contains("map '<ron>' has invalid authoring calls"));
    assert!(message.contains("tile_size() is only valid on in-code maps"));
    assert!(message.contains("tiles() is only valid on in-code maps"));
    assert!(message.contains("spawn() is only valid on in-code maps"));
}

#[test]
fn map_without_theme_points_to_simple_theme() {
    let mut builder = GameBuilder::new();
    let mut game = GameApp::new(&mut builder);

    game.map("demo").tiles(["."]).start();

    let err = game.finish().unwrap_err();
    let message = err.to_string();

    assert!(message.contains("Map 'demo' has no tile theme."));
    assert!(message.contains(".simple_theme(assets.floor, assets.wall)"));
}

#[test]
fn simple_theme_satisfies_map_theme_requirement() {
    let mut builder = GameBuilder::new();
    let mut game = GameApp::new(&mut builder);

    game.map("demo")
        .tiles(["."])
        .simple_theme(TextureHandle(1), TextureHandle(2))
        .start();

    game.finish().unwrap();
}

#[test]
fn no_start_map_returns_error() {
    let mut builder = GameBuilder::new();
    let game = GameApp::new(&mut builder);

    let err = game.finish().unwrap_err();

    let message = err.to_string();
    assert!(message.contains("No start map declared."));
    assert!(message.contains(".simple_theme(assets.floor, assets.wall)"));
}

#[test]
fn multiple_start_maps_return_error() {
    let mut builder = GameBuilder::new();
    let mut game = GameApp::new(&mut builder);

    game.map("first").tiles(["."]).theme(test_theme()).start();
    game.map("second").tiles(["."]).theme(test_theme()).start();

    let err = game.finish().unwrap_err();

    let message = err.to_string();
    assert!(message.contains("Multiple start maps declared: 'first' and 'second'"));
    assert!(message.contains("Other maps should end with .finish()"));
}
