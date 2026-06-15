use crate::builder::GameBuilder;

/// Low-level plugin trait consumed by the runtime.
///
/// Content crates normally implement `game_kit::GamePlugin`; `game_kit::plugin`
/// adapts that author-facing trait to this runtime-facing one.
pub trait GamePlugin {
    fn build(&self, app: &mut GameBuilder) -> anyhow::Result<()>;
}
