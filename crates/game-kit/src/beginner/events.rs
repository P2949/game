//! Beginner event helpers.

use game_combat::Health;
use game_core::world::EntityId;
use glam::Vec2;

use crate::beginner::collections::ScoreOps;
use crate::beginner::spawn::SpawnAuthor;
use crate::context::GameCtx;

pub const DEFAULT_PICKUP_COLLECT_RANGE: f32 = 24.0;

/// An internal identity for an actor involved in a beginner event. Its entity id
/// never reaches beginner content; [`EventActor`] is the safe, object-shaped
/// handle exposed by event callbacks.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ActorToken {
    id: EntityId,
}

impl ActorToken {
    pub(crate) fn new(id: EntityId) -> Self {
        Self { id }
    }
}

/// A mutable actor handle supplied by a custom event rule.
pub struct EventActor<'g, 'a, 'w> {
    game: &'g mut GameCtx<'a, 'w>,
    token: ActorToken,
}

impl<'g, 'a, 'w> EventActor<'g, 'a, 'w> {
    pub(crate) fn new(game: &'g mut GameCtx<'a, 'w>, token: ActorToken) -> Self {
        Self { game, token }
    }

    /// The actor's current world position, when it still exists.
    pub fn position(&self) -> Option<Vec2> {
        self.game.position(self.token.id)
    }

    /// Applies damage and reports whether the actor had health to damage.
    pub fn damage(&mut self, amount: i32) -> bool {
        self.game.damage_entity(self.token.id, amount)
    }

    /// Restores health and reports whether the actor had health to heal.
    pub fn heal(&mut self, amount: i32) -> bool {
        let Some(health) = self.game.component_mut::<Health>(self.token.id) else {
            return false;
        };
        health.current = (health.current + amount.max(0)).min(health.max);
        true
    }

    /// Reduces the actor's health to zero when it has health.
    pub fn kill(&mut self) {
        let current = self
            .game
            .component::<Health>(self.token.id)
            .map(|health| health.current)
            .unwrap_or_default();
        if current > 0 {
            self.game.damage_entity(self.token.id, current);
        }
    }

    /// Queues removal of this actor after the current simulation step.
    pub fn despawn(self) {
        self.game.commands().despawn(self.token.id);
    }

    /// Starts a configured animation and reports whether it was present.
    pub fn play_animation(&mut self, name: impl Into<String>) -> bool {
        self.game.play_animation(self.token.id, name)
    }
}

/// Information and safe actor handles for a player collecting a pickup.
pub struct CollectEvent<'g, 'a, 'w> {
    game: &'g mut GameCtx<'a, 'w>,
    collector: ActorToken,
    pickup: ActorToken,
}

impl<'g, 'a, 'w> CollectEvent<'g, 'a, 'w> {
    pub(crate) fn new(
        game: &'g mut GameCtx<'a, 'w>,
        collector: ActorToken,
        pickup: ActorToken,
    ) -> Self {
        Self {
            game,
            collector,
            pickup,
        }
    }

    pub fn collector(&mut self) -> EventActor<'_, 'a, 'w> {
        EventActor::new(self.game, self.collector)
    }

    pub fn pickup(&mut self) -> EventActor<'_, 'a, 'w> {
        EventActor::new(self.game, self.pickup)
    }

    /// Alias for games that call their pickup prefab a coin.
    pub fn coin(&mut self) -> EventActor<'_, 'a, 'w> {
        self.pickup()
    }

    pub fn collector_position(&self) -> Option<Vec2> {
        self.game.position(self.collector.id)
    }

    pub fn pickup_position(&self) -> Option<Vec2> {
        self.game.position(self.pickup.id)
    }

    pub fn despawn_pickup(&mut self) {
        self.game.commands().despawn(self.pickup.id);
    }

    pub fn score(&mut self) -> ScoreOps<'_, 'a, 'w> {
        self.game.score()
    }

    pub fn play_sound(&mut self, key: &str) {
        self.game.play_sound_named(key);
    }

    pub fn spawn(&mut self, prefab: impl Into<String>) -> SpawnAuthor<'_, 'a, 'w> {
        self.game.spawn(prefab)
    }
}

/// Information and a safe actor handle for an enemy that has just died.
pub struct EnemyDeathEvent<'g, 'a, 'w> {
    game: &'g mut GameCtx<'a, 'w>,
    enemy: ActorToken,
}

impl<'g, 'a, 'w> EnemyDeathEvent<'g, 'a, 'w> {
    pub(crate) fn new(game: &'g mut GameCtx<'a, 'w>, enemy: ActorToken) -> Self {
        Self { game, enemy }
    }

    pub fn enemy(&mut self) -> EventActor<'_, 'a, 'w> {
        EventActor::new(self.game, self.enemy)
    }

    pub fn enemy_position(&self) -> Option<Vec2> {
        self.game.position(self.enemy.id)
    }

    pub fn despawn_enemy(&mut self) {
        self.game.commands().despawn(self.enemy.id);
    }

    pub fn score(&mut self) -> ScoreOps<'_, 'a, 'w> {
        self.game.score()
    }

    pub fn play_sound(&mut self, key: &str) {
        self.game.play_sound_named(key);
    }

    pub fn spawn(&mut self, prefab: impl Into<String>) -> SpawnAuthor<'_, 'a, 'w> {
        self.game.spawn(prefab)
    }
}

/// Information and safe actor handles for a player touching another actor.
pub struct CollisionEvent<'g, 'a, 'w> {
    game: &'g mut GameCtx<'a, 'w>,
    player: ActorToken,
    other: ActorToken,
}

/// Information for a one-shot animation that reached its final frame.
pub struct AnimationFinishedEvent<'g, 'a, 'w> {
    game: &'g mut GameCtx<'a, 'w>,
    actor: ActorToken,
    name: String,
}

impl<'g, 'a, 'w> AnimationFinishedEvent<'g, 'a, 'w> {
    pub(crate) fn new(
        game: &'g mut GameCtx<'a, 'w>,
        actor: ActorToken,
        name: impl Into<String>,
    ) -> Self {
        Self {
            game,
            actor,
            name: name.into(),
        }
    }

    pub fn actor(&mut self) -> EventActor<'_, 'a, 'w> {
        EventActor::new(self.game, self.actor)
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<'g, 'a, 'w> CollisionEvent<'g, 'a, 'w> {
    pub(crate) fn new(
        game: &'g mut GameCtx<'a, 'w>,
        player: ActorToken,
        other: ActorToken,
    ) -> Self {
        Self {
            game,
            player,
            other,
        }
    }

    pub fn player(&mut self) -> EventActor<'_, 'a, 'w> {
        EventActor::new(self.game, self.player)
    }

    pub fn other(&mut self) -> EventActor<'_, 'a, 'w> {
        EventActor::new(self.game, self.other)
    }

    pub fn player_position(&self) -> Option<Vec2> {
        self.game.position(self.player.id)
    }

    pub fn other_position(&self) -> Option<Vec2> {
        self.game.position(self.other.id)
    }

    pub fn score(&mut self) -> ScoreOps<'_, 'a, 'w> {
        self.game.score()
    }

    pub fn play_sound(&mut self, key: &str) {
        self.game.play_sound_named(key);
    }
}

#[cfg(test)]
mod tests {
    use game_core::backend::{SoundHandle, TextureHandle};

    use crate::app::{GameApp, GamePlugin};
    use crate::beginner::camera::CameraShake;
    use crate::beginner::collections::Score;
    use crate::beginner::state::SimpleGameState;
    use crate::context::{GameCtx, StartupGameCtx};
    use crate::harness::GameTestHarness;

    struct EventPlugin;

    impl GamePlugin for EventPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> anyhow::Result<()> {
            let controls = game.input(|input| input.top_down_controls())?;

            game.scene("menu").start_scene("menu");

            game.player_prefab("player")
                .sprite(TextureHandle(1))
                .moves_with(controls.movement, 130.0)
                .build()?;

            game.pickup_prefab("coin")
                .sprite(TextureHandle(2))
                .score(3)
                .play_sound(SoundHandle(1))
                .despawn_on_collect()
                .build()?;

            game.map("events")
                .tiles(["#####", "#PC.#", "#####"])
                .simple_theme(TextureHandle(10), TextureHandle(11))
                .legend('P', "player")
                .legend('C', "coin")
                .start();

            game.on_start(|game: &mut StartupGameCtx<'_, '_>| {
                game.init_resource::<SimpleGameState>();
                game.spawn_start_map()
            });

            game.on_action(controls.attack, |game: &mut GameCtx<'_, '_>| {
                game.score().add(1);
            });
            game.on_action_when_playing(controls.attack, |game: &mut GameCtx<'_, '_>| {
                game.score().add(10);
            });
            game.on_action_cooldown(controls.attack, 1.0, |game: &mut GameCtx<'_, '_>| {
                game.camera2d().shake(0.25);
                game.score().add(100_000);
            });
            game.every_seconds(0.5, |game: &mut GameCtx<'_, '_>| {
                game.score().add(100);
            });
            game.after_seconds(0.25, |game: &mut GameCtx<'_, '_>| {
                game.score().add(1000);
            });
            game.on_scene_enter("menu", |game: &mut GameCtx<'_, '_>| {
                game.score().add(10_000);
            });
            game.on_scene("menu", |game: &mut GameCtx<'_, '_>, _dt| {
                game.score().add(5);
            });
            game.on_player_collect_pickup_within(40.0, |game: &mut GameCtx<'_, '_>| {
                game.score().add(20);
            });

            Ok(())
        }
    }

    #[test]
    fn event_helpers_register_beginner_systems() {
        let mut game = GameTestHarness::from_plugin(EventPlugin).unwrap();

        game.frame(0.0);
        game.tap_action("attack");
        game.tap_action("attack");
        game.step_seconds(0.5);

        assert_eq!(game.world().get_resource::<Score>().unwrap().value, 111_150);
        assert!(game.world().get_resource::<CameraShake>().is_some());
        game.assert_sound_played();
    }

    struct ObjectEventPlugin;

    impl GamePlugin for ObjectEventPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> anyhow::Result<()> {
            game.asset_bag()
                .texture("player", "textures/player.png")?
                .texture("slime", "textures/slime.png")?
                .texture("coin", "textures/coin.png")?
                .texture("gem", "textures/gem.png")?
                .texture("door", "textures/door.png")?
                .texture("floor", "textures/floor.png")?
                .texture("wall", "textures/wall.png")?
                .generated_sound("coin")?
                .build();
            let controls = game.input(|input| input.top_down_controls())?;

            game.player_prefab("player")
                .sprite("player")
                .moves_with(controls.movement, 120.0)
                .health(10)
                .build()?;
            game.enemy_prefab("slime")
                .sprite("slime")
                .health(10)
                .build()?;
            game.pickup_prefab("coin")
                .sprite("coin")
                .score(0)
                .despawn_on_collect()
                .build()?;
            game.pickup_prefab("gem").sprite("gem").build()?;
            game.door_prefab("door")
                .sprite("door")
                .restart_level()
                .build()?;

            game.map("events")
                .tile_size(16.0)
                .tiles(["######", "#PCED#", "######"])
                .simple_theme("floor", "wall")
                .legend('P', "player")
                .legend('C', "coin")
                .legend('E', "slime")
                .legend('D', "door")
                .start();
            game.on_start(|game| game.spawn_start_map());

            game.on_collect("player", "coin", |event| {
                event.coin().despawn();
                event.score().add(7);
                event.play_sound("coin");
            });
            game.on_action(controls.attack, |game| {
                game.enemies().alive().damage(100);
            });
            game.on_enemy_death_event(|event| {
                let position = event.enemy_position();
                event.enemy().despawn();
                event.score().add(4);
                if let Some(position) = position {
                    event.spawn("gem").at_world(position);
                }
            });
            game.on_player_touching_door(|event| {
                event.other().despawn();
                event.score().add(2);
            });
            Ok(())
        }
    }

    #[test]
    fn object_shaped_events_match_prefabs_and_offer_safe_actor_operations() {
        let mut game = GameTestHarness::from_plugin(ObjectEventPlugin).unwrap();

        game.step();
        assert_eq!(game.world().get_resource::<Score>().unwrap().value, 7);
        assert_eq!(game.count::<crate::beginner::actors::Pickup>(), 0);
        game.assert_sound_played();

        game.tap_action("attack");
        assert_eq!(game.world().get_resource::<Score>().unwrap().value, 11);
        assert_eq!(game.enemy_count(), 0);
        assert_eq!(game.count::<crate::beginner::actors::Pickup>(), 1);

        game.move_entity_to(game.player(), glam::vec2(72.0, 24.0));
        game.step();
        assert_eq!(game.world().get_resource::<Score>().unwrap().value, 13);
        assert_eq!(game.count::<crate::beginner::actors::Door>(), 0);
    }
}
