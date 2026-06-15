use crate::app::Game;
use crate::builder::GameBuilder;

pub trait GamePlugin {
    type Game: Game;

    fn build(&self, app: &mut GameBuilder) -> anyhow::Result<Self::Game>;
}
