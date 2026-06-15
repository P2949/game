use crate::builder::GameBuilder;

pub trait GamePlugin {
    fn build(&self, app: &mut GameBuilder) -> anyhow::Result<()>;
}
