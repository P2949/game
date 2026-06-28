//! Declarative custom beginner rules.

use game_core::world::{EntityId, NamedValues, Tags};
use glam::Vec2;

use crate::app::GameApp;
use crate::beginner::state::SimpleGameState;
use crate::context::GameCtx;

/// Starts a named custom rule. The current beginner custom-rule surface is
/// intentionally small: compose concrete patterns first, and keep arbitrary
/// system code in advanced content.
pub struct CustomRuleAuthor<'a, 'app> {
    app: &'a mut GameApp<'app>,
    name: String,
}

impl<'a, 'app> CustomRuleAuthor<'a, 'app> {
    pub(crate) fn new(app: &'a mut GameApp<'app>, name: String) -> Self {
        Self { app, name }
    }

    /// Selects actors carrying a tag added by `.tag("...")` on a prefab.
    pub fn for_each_tag(self, tag: impl Into<String>) -> TaggedCustomRuleAuthor<'a, 'app> {
        TaggedCustomRuleAuthor {
            app: self.app,
            name: self.name,
            tag: tag.into(),
        }
    }
}

/// Custom-rule author after actor selection.
pub struct TaggedCustomRuleAuthor<'a, 'app> {
    app: &'a mut GameApp<'app>,
    name: String,
    tag: String,
}

impl<'a, 'app> TaggedCustomRuleAuthor<'a, 'app> {
    /// Counts down the selected actors' named numeric data each active tick.
    pub fn countdown(self, key: impl Into<String>) -> CountdownRuleAuthor<'a, 'app> {
        CountdownRuleAuthor {
            app: self.app,
            name: self.name,
            tag: self.tag,
            key: key.into(),
            effects: Vec::new(),
        }
    }
}

/// Builder for "when this tag's data key reaches zero" rules.
pub struct CountdownRuleAuthor<'a, 'app> {
    app: &'a mut GameApp<'app>,
    name: String,
    tag: String,
    key: String,
    effects: Vec<CountdownEffect>,
}

#[derive(Clone, Debug, PartialEq)]
enum CountdownEffect {
    DamageTag {
        tag: String,
        amount: i32,
        radius: f32,
    },
    DamagePlayer {
        amount: i32,
        radius: f32,
    },
    DespawnSelf,
    PlaySound(String),
    SpawnPrefab(String),
}

impl<'a, 'app> CountdownRuleAuthor<'a, 'app> {
    /// Readability marker for chained effects.
    pub fn when_zero(self) -> Self {
        self
    }

    /// Damages tagged actors within `radius` of the expired actor.
    pub fn damage_tag(mut self, tag: impl Into<String>, amount: i32, radius: f32) -> Self {
        self.effects.push(CountdownEffect::DamageTag {
            tag: tag.into(),
            amount,
            radius: radius.max(0.0),
        });
        self
    }

    /// Damages the player within `radius` of the expired actor.
    pub fn damage_player(mut self, amount: i32, radius: f32) -> Self {
        self.effects.push(CountdownEffect::DamagePlayer {
            amount,
            radius: radius.max(0.0),
        });
        self
    }

    /// Queues removal of the expired actor.
    pub fn despawn_self(mut self) -> Self {
        self.effects.push(CountdownEffect::DespawnSelf);
        self
    }

    /// Plays a named sound when the countdown reaches zero.
    pub fn play_sound(mut self, key: impl Into<String>) -> Self {
        self.effects.push(CountdownEffect::PlaySound(key.into()));
        self
    }

    /// Spawns a prefab at the expired actor's position.
    pub fn spawn_prefab(mut self, prefab: impl Into<String>) -> Self {
        self.effects
            .push(CountdownEffect::SpawnPrefab(prefab.into()));
        self
    }

    pub fn build(self) {
        let tag = self.tag;
        let key = self.key;
        let effects = self.effects;
        let _name = self.name;
        self.app
            .every_active_tick::<SimpleGameState>(move |game, dt| {
                countdown_rule_system(game, dt, &tag, &key, &effects);
            });
    }
}

fn countdown_rule_system(
    game: &mut GameCtx<'_, '_>,
    dt: f32,
    tag: &str,
    key: &str,
    effects: &[CountdownEffect],
) {
    let tagged = game
        .entities_with::<Tags>()
        .into_iter()
        .filter(|id| {
            game.component::<Tags>(*id)
                .is_some_and(|tags| tags.has(tag))
        })
        .collect::<Vec<_>>();
    let expired = tagged
        .into_iter()
        .filter_map(|id| tick_countdown(game, id, key, dt).map(|position| (id, position)))
        .collect::<Vec<_>>();

    for (actor, position) in expired {
        apply_countdown_effects(game, actor, position, effects);
    }
}

fn tick_countdown(game: &mut GameCtx<'_, '_>, actor: EntityId, key: &str, dt: f32) -> Option<Vec2> {
    let values = game.component_mut::<NamedValues>(actor)?;
    let remaining = values.get_f32(key).unwrap_or_default() - dt.max(0.0);
    values.set_f32(key, remaining);
    (remaining <= 0.0).then(|| game.position(actor)).flatten()
}

fn apply_countdown_effects(
    game: &mut GameCtx<'_, '_>,
    actor: EntityId,
    position: Vec2,
    effects: &[CountdownEffect],
) {
    for effect in effects {
        match effect {
            CountdownEffect::DamageTag {
                tag,
                amount,
                radius,
            } => {
                game.actors_tagged(tag)
                    .near(position, *radius)
                    .damage(*amount);
            }
            CountdownEffect::DamagePlayer { amount, radius } => {
                game.player().damage_if_near(position, *radius, *amount);
            }
            CountdownEffect::DespawnSelf => {
                game.commands().despawn(actor);
            }
            CountdownEffect::PlaySound(key) => game.play_sound_named(key),
            CountdownEffect::SpawnPrefab(prefab) => game.spawn(prefab.clone()).at_world(position),
        }
    }
}

#[cfg(test)]
mod tests {
    use game_core::backend::TextureHandle;

    use crate::app::{GameApp, GamePlugin};
    use crate::context::StartupGameCtx;
    use crate::harness::GameTestHarness;

    struct CountdownPlugin;

    impl GamePlugin for CountdownPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> anyhow::Result<()> {
            let controls = game.input(|input| input.top_down_controls())?;
            game.player_prefab("player")
                .sprite(TextureHandle(1))
                .moves_with(controls.movement, 130.0)
                .health(100)
                .build()?;
            game.enemy_prefab("bomber")
                .sprite(TextureHandle(2))
                .tag("explosive")
                .tag("enemy")
                .data("fuse", 0.01)
                .health(10)
                .build()?;
            game.enemy_prefab("slime")
                .sprite(TextureHandle(2))
                .tag("enemy")
                .health(10)
                .build()?;
            game.map("rules")
                .tiles(["#####", "#PB.#", "#E..#", "#####"])
                .simple_theme(TextureHandle(10), TextureHandle(11))
                .legend('P', "player")
                .legend('B', "bomber")
                .legend('E', "slime")
                .start();
            game.on_start(|game: &mut StartupGameCtx<'_, '_>| game.spawn_start_map());
            game.rules().top_down_controls(controls).build();
            game.custom_rule("fuse")
                .for_each_tag("explosive")
                .countdown("fuse")
                .when_zero()
                .damage_tag("enemy", 4, 80.0)
                .damage_player(4, 80.0)
                .despawn_self()
                .build();
            Ok(())
        }
    }

    #[test]
    fn countdown_rule_applies_effects_without_user_loop_code() {
        let mut game = GameTestHarness::from_plugin(CountdownPlugin).unwrap();

        game.step_seconds(0.02);

        assert!(game.player().health() < 100);
        assert!(game.enemy_count() < 2 || game.enemies().iter().any(|enemy| enemy.health() < 10));
    }
}
