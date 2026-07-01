use anyhow::Result;
use game_core::builder::GameBuilder;

use super::{GameApp, GamePlugin};

/// Adapts a [`GamePlugin`] (the content-facing trait) to the engine's
/// `game_core::plugin::GamePlugin` so the runtime can run it. Build a value with
/// [`plugin`].
pub struct Plugin<P>(P);

impl<P: GamePlugin> game_core::plugin::GamePlugin for Plugin<P> {
    fn build(&self, builder: &mut GameBuilder) -> Result<()> {
        let mut app = GameApp::new(builder);
        self.0.build(&mut app)?;
        app.finish()
    }
}

/// Wraps a content plugin so it can be handed to `game_runtime::run`. Content's
/// `pub fn plugin()` returns `game_kit::app::plugin(MyPlugin)`.
pub fn plugin<P: GamePlugin>(plugin: P) -> Plugin<P> {
    Plugin(plugin)
}

pub struct FnGamePlugin<F>(F);

impl<F> GamePlugin for FnGamePlugin<F>
where
    F: for<'app> Fn(&mut GameApp<'app>) -> Result<()>,
{
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        (self.0)(game)
    }
}

pub fn plugin_fn<F>(build: F) -> Plugin<FnGamePlugin<F>>
where
    F: for<'app> Fn(&mut GameApp<'app>) -> Result<()>,
{
    plugin(FnGamePlugin(build))
}
