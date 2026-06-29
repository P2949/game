//! Beginner event helpers.

use std::collections::HashSet;

use game_combat::Health;
use game_core::world::{EntityId, NamedValues, Sprite, Tags, Transform, Velocity};
use glam::Vec2;

use crate::beginner::actors::{FacingDirection, PrefabName};
use crate::beginner::collections::ScoreOps;
use crate::beginner::spawn::SpawnAuthor;
use crate::context::GameCtx;

pub const DEFAULT_PICKUP_COLLECT_RANGE: f32 = 24.0;

/// Tracks the currently active overlaps for one or more authored area events.
/// The string keeps independently registered event pairs from interfering with
/// one another when an actor overlaps several areas at once.
#[derive(Default)]
pub(crate) struct OverlapTracker {
    pub(crate) active: HashSet<(EntityId, EntityId, String)>,
}

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

    /// Reads a named numeric value configured with `.data("key", value)`.
    pub fn data(&self, key: &str) -> Option<f32> {
        self.game
            .component::<NamedValues>(self.token.id)
            .and_then(|values| values.get_f32(key))
    }

    /// Replaces a named numeric value, creating the actor data bag when needed.
    pub fn set_data(&mut self, key: impl Into<String>, value: f32) {
        let key = key.into();
        if let Some(values) = self.game.component_mut::<NamedValues>(self.token.id) {
            values.set_f32(key, value);
        } else {
            let mut values = NamedValues::default();
            values.set_f32(key, value);
            self.game.insert_component(self.token.id, values);
        }
    }

    /// Adds to a named numeric value, creating it from zero when needed.
    pub fn add_data(&mut self, key: impl Into<String>, amount: f32) {
        let key = key.into();
        let value = self.data(&key).unwrap_or_default() + amount;
        self.set_data(key, value);
    }

    /// Returns whether this actor carries a particular author-defined tag.
    pub fn has_tag(&self, tag: &str) -> bool {
        self.game
            .component::<Tags>(self.token.id)
            .is_some_and(|tags| tags.has(tag))
    }

    /// Returns whether this actor came from the named beginner prefab.
    pub fn is_prefab(&self, name: &str) -> bool {
        self.game
            .component::<PrefabName>(self.token.id)
            .is_some_and(|prefab| prefab.matches(name))
    }

    /// Adds an author-defined tag to this actor.
    pub fn set_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        if let Some(tags) = self.game.component_mut::<Tags>(self.token.id) {
            tags.0.insert(tag);
        } else {
            let mut tags = Tags::default();
            tags.0.insert(tag);
            self.game.insert_component(self.token.id, tags);
        }
    }

    /// Removes an author-defined tag from this actor, if it has one.
    pub fn remove_tag(&mut self, tag: &str) {
        if let Some(tags) = self.game.component_mut::<Tags>(self.token.id) {
            tags.0.remove(tag);
        }
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

    /// Spawns a prefab at this actor's current position.
    pub fn spawn(&mut self, prefab: impl Into<String>) {
        if let Some(position) = self.position() {
            self.game.spawn(prefab).at_world(position);
        }
    }

    /// Alias for [`Self::spawn`] when the spawned prefab is a pickup/drop.
    pub fn drop(&mut self, prefab: impl Into<String>) {
        self.spawn(prefab);
    }

    /// Moves this actor to a world position.
    pub fn move_to(&mut self, position: Vec2) {
        if let Some(transform) = self.game.component_mut::<Transform>(self.token.id) {
            transform.pos = position;
        }
    }

    /// Pushes this actor directly away from a point by setting its velocity.
    pub fn push_away_from(&mut self, point: Vec2, speed: f32) {
        let Some(position) = self.position() else {
            return;
        };
        let velocity = (position - point).normalize_or_zero() * speed.max(0.0);
        if let Some(current) = self.game.component_mut::<Velocity>(self.token.id) {
            current.0 = velocity;
        } else {
            self.game
                .insert_component(self.token.id, Velocity::new(velocity));
        }
    }

    /// Plays a registered sound by key.
    pub fn play_sound(&mut self, key: &str) {
        self.game.play_sound_named(key);
    }

    /// Changes this actor's sprite texture by registered texture key.
    pub fn change_sprite(&mut self, key: &str) -> bool {
        let Some(texture) = self.game.named_texture(key) else {
            self.game.report_missing_named_texture(key);
            return false;
        };
        let Some(sprite) = self.game.component_mut::<Sprite>(self.token.id) else {
            return false;
        };
        sprite.texture = texture;
        true
    }

    /// Updates this actor's stored facing direction to point toward the player.
    pub fn face_towards_player(&mut self) {
        let Some(position) = self.position() else {
            return;
        };
        let Some(player_position) = self.game.player_position() else {
            return;
        };
        let Some(direction) = FacingDirection::from_motion(player_position - position) else {
            return;
        };
        if let Some(facing) = self.game.component_mut::<FacingDirection>(self.token.id) {
            *facing = direction;
        } else {
            self.game.insert_component(self.token.id, direction);
        }
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

    pub fn change_scene(&mut self, scene: &str) {
        self.game.change_scene_or_log(scene);
    }

    pub fn change_map(&mut self, map: &str) {
        self.game.change_map_or_log(map);
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

    pub fn change_scene(&mut self, scene: &str) {
        self.game.change_scene_or_log(scene);
    }

    pub fn change_map(&mut self, map: &str) {
        self.game.change_map_or_log(map);
    }

    pub fn spawn(&mut self, prefab: impl Into<String>) -> SpawnAuthor<'_, 'a, 'w> {
        self.game.spawn(prefab)
    }
}

/// Information for a map transition observed by beginner content.
pub struct MapChangedEvent<'g, 'a, 'w> {
    game: &'g mut GameCtx<'a, 'w>,
    previous: Option<String>,
    current: Option<String>,
}

impl<'g, 'a, 'w> MapChangedEvent<'g, 'a, 'w> {
    pub(crate) fn new(
        game: &'g mut GameCtx<'a, 'w>,
        previous: Option<String>,
        current: Option<String>,
    ) -> Self {
        Self {
            game,
            previous,
            current,
        }
    }

    /// The map now active after the change.
    pub fn map_name(&self) -> Option<&str> {
        self.current.as_deref()
    }

    /// The map that was active before the change.
    pub fn previous_map_name(&self) -> Option<&str> {
        self.previous.as_deref()
    }

    /// The player position after the change, if a player exists.
    pub fn position(&self) -> Option<Vec2> {
        self.game.player_position()
    }

    pub fn player(&mut self) -> Option<EventActor<'_, 'a, 'w>> {
        self.game
            .player_id()
            .map(ActorToken::new)
            .map(|token| EventActor::new(self.game, token))
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

    pub fn change_scene(&mut self, scene: &str) {
        self.game.change_scene_or_log(scene);
    }

    pub fn change_map(&mut self, map: &str) {
        self.game.change_map_or_log(map);
    }
}

/// Information for an eligible door opening while the player is near it.
pub struct DoorEvent<'g, 'a, 'w> {
    game: &'g mut GameCtx<'a, 'w>,
    player: ActorToken,
    door: ActorToken,
}

impl<'g, 'a, 'w> DoorEvent<'g, 'a, 'w> {
    pub(crate) fn new(game: &'g mut GameCtx<'a, 'w>, player: ActorToken, door: ActorToken) -> Self {
        Self { game, player, door }
    }

    pub fn player(&mut self) -> EventActor<'_, 'a, 'w> {
        EventActor::new(self.game, self.player)
    }

    pub fn door(&mut self) -> EventActor<'_, 'a, 'w> {
        EventActor::new(self.game, self.door)
    }

    pub fn position(&self) -> Option<Vec2> {
        self.game.position(self.door.id)
    }

    pub fn map_name(&self) -> Option<String> {
        self.game.current_map_name()
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

    pub fn change_scene(&mut self, scene: &str) {
        self.game.change_scene_or_log(scene);
    }

    pub fn change_map(&mut self, map: &str) {
        self.game.change_map_or_log(map);
    }
}

/// Information for a projectile touching a target actor.
pub struct ProjectileHitEvent<'g, 'a, 'w> {
    game: &'g mut GameCtx<'a, 'w>,
    projectile: ActorToken,
    enemy: ActorToken,
    position: Vec2,
}

impl<'g, 'a, 'w> ProjectileHitEvent<'g, 'a, 'w> {
    pub(crate) fn new(
        game: &'g mut GameCtx<'a, 'w>,
        projectile: ActorToken,
        enemy: ActorToken,
        position: Vec2,
    ) -> Self {
        Self {
            game,
            projectile,
            enemy,
            position,
        }
    }

    pub fn projectile(&mut self) -> EventActor<'_, 'a, 'w> {
        EventActor::new(self.game, self.projectile)
    }

    pub fn enemy(&mut self) -> EventActor<'_, 'a, 'w> {
        EventActor::new(self.game, self.enemy)
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn map_name(&self) -> Option<String> {
        self.game.current_map_name()
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

    pub fn change_scene(&mut self, scene: &str) {
        self.game.change_scene_or_log(scene);
    }

    pub fn change_map(&mut self, map: &str) {
        self.game.change_map_or_log(map);
    }
}

/// Information and safe actor handles for a player touching another actor.
pub struct CollisionEvent<'g, 'a, 'w> {
    game: &'g mut GameCtx<'a, 'w>,
    a: ActorToken,
    b: ActorToken,
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
    pub(crate) fn new(game: &'g mut GameCtx<'a, 'w>, a: ActorToken, b: ActorToken) -> Self {
        Self { game, a, b }
    }

    /// The first prefab supplied to an overlap callback.
    pub fn a(&mut self) -> EventActor<'_, 'a, 'w> {
        EventActor::new(self.game, self.a)
    }

    /// The second prefab supplied to an overlap callback.
    pub fn b(&mut self) -> EventActor<'_, 'a, 'w> {
        EventActor::new(self.game, self.b)
    }

    /// Alias for [`Self::a`] in actor/area callbacks.
    pub fn actor(&mut self) -> EventActor<'_, 'a, 'w> {
        self.a()
    }

    /// Alias for [`Self::b`] in actor/area callbacks.
    pub fn area(&mut self) -> EventActor<'_, 'a, 'w> {
        self.b()
    }

    /// Backwards-compatible alias for [`Self::a`].
    pub fn player(&mut self) -> EventActor<'_, 'a, 'w> {
        self.a()
    }

    /// Backwards-compatible alias for [`Self::b`].
    pub fn other(&mut self) -> EventActor<'_, 'a, 'w> {
        self.b()
    }

    pub fn enemy(&mut self) -> EventActor<'_, 'a, 'w> {
        self.b()
    }

    pub fn projectile(&mut self) -> EventActor<'_, 'a, 'w> {
        self.a()
    }

    pub fn door(&mut self) -> EventActor<'_, 'a, 'w> {
        self.b()
    }

    pub fn a_position(&self) -> Option<Vec2> {
        self.game.position(self.a.id)
    }

    pub fn b_position(&self) -> Option<Vec2> {
        self.game.position(self.b.id)
    }

    /// Backwards-compatible alias for [`Self::a_position`].
    pub fn player_position(&self) -> Option<Vec2> {
        self.a_position()
    }

    /// Backwards-compatible alias for [`Self::b_position`].
    pub fn other_position(&self) -> Option<Vec2> {
        self.b_position()
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

    pub fn change_scene(&mut self, scene: &str) {
        self.game.change_scene_or_log(scene);
    }

    pub fn change_map(&mut self, map: &str) {
        self.game.change_map_or_log(map);
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use game_core::backend::{SoundHandle, TextureHandle};
    use game_core::world::Sprite;
    use game_map::cell;

    use crate::app::{GameApp, GamePlugin};
    use crate::beginner::camera::CameraShake;
    use crate::beginner::collections::Score;
    use crate::beginner::state::SimpleGameState;
    use crate::context::StartupGameCtx;
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

            game.startup(|game: &mut StartupGameCtx<'_, '_>| {
                game.init_resource::<SimpleGameState>();
                game.spawn_start_map()
            });

            game.on_action(controls.attack, |game| {
                game.score().add(1);
            });
            game.on_action_when_playing(controls.attack, |game| {
                game.score().add(10);
            });
            game.on_action_cooldown(controls.attack, 1.0, |game| {
                game.camera2d().shake(0.25);
                game.score().add(100_000);
            });
            game.every_seconds(0.5, |game| {
                game.score().add(100);
            });
            game.after_seconds(0.25, |game| {
                game.score().add(1000);
            });
            game.on_scene_enter("menu", |game| {
                game.score().add(10_000);
            });
            game.on_scene("menu", |game, _dt| {
                game.score().add(5);
            });
            game.on_player_collect_pickup_within(40.0, |game| {
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

    struct Phase8HookPlugin;

    impl GamePlugin for Phase8HookPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> anyhow::Result<()> {
            game.asset_bag()
                .texture("player", "textures/player.png")?
                .texture("slime", "textures/slime.png")?
                .texture("bolt", "textures/bolt.png")?
                .texture("coin", "textures/coin.png")?
                .texture("door", "textures/door.png")?
                .texture("floor", "textures/floor.png")?
                .texture("wall", "textures/wall.png")?
                .generated_sound("hit")?
                .build();
            let controls = game.input(|input| input.top_down_controls())?;

            game.player_prefab("player")
                .sprite("player")
                .moves_with(controls.movement, 120.0)
                .health(5)
                .build()?;
            game.enemy_prefab("slime")
                .sprite("slime")
                .health(1)
                .build()?;
            game.projectile_prefab("bolt")
                .sprite("bolt")
                .speed(0.0)
                .lifetime(5.0)
                .build()?;
            game.pickup_prefab("coin").sprite("coin").build()?;
            game.door_prefab("exit")
                .sprite("door")
                .change_map("level_2")
                .build()?;

            game.map("level_1")
                .tile_size(16.0)
                .tiles(["#######", "#PBED.#", "#######"])
                .simple_theme("floor", "wall")
                .legend('P', "player")
                .legend('B', "bolt")
                .legend('E', "slime")
                .legend('D', "exit")
                .start();
            game.map("level_2")
                .tile_size(16.0)
                .tiles(["#####", "#P..#", "#####"])
                .simple_theme("floor", "wall")
                .legend('P', "player")
                .finish();

            game.startup(|game: &mut StartupGameCtx<'_, '_>| {
                game.init_resource::<SimpleGameState>();
                game.spawn_start_map()
            });

            game.on_projectile_hit("bolt", "slime", |event| {
                let position = event.position();
                event.score().add(2);
                event.play_sound("hit");
                event.enemy().set_tag("hit");
                event.enemy().add_data("hits", 1.0);
                event.enemy().change_sprite("coin");
                event.spawn("coin").at_world(position);
            });
            game.on_wave_cleared(|game| {
                game.score().add(4);
            });
            game.on_player_death(|game| {
                game.score().add(8);
            });
            game.on_player_respawn(|game| {
                game.score().add(16);
            });
            game.on_door_open("exit", |event| {
                event.score().add(32);
                event.change_map("level_2");
            });
            game.on_map_exit("level_1", |game| {
                game.score().add(64);
            });
            game.on_map_enter("level_2", |game| {
                game.score().add(128);
            });
            game.on_map_changed(|event| {
                event.score().add(256);
                event.play_sound("hit");
            });
            game.on_timer("bonus", 0.1, |game| {
                game.score().add(512);
            });
            game.every_seconds_while_playing(0.1, |game| {
                game.score().add(1024);
            });
            Ok(())
        }
    }

    #[test]
    fn phase8_hooks_fire_with_safe_event_objects() {
        fn score(game: &GameTestHarness) -> i32 {
            game.world()
                .get_resource::<Score>()
                .map(|score| score.value)
                .unwrap_or_default()
        }

        let mut game = GameTestHarness::from_plugin(Phase8HookPlugin).unwrap();

        game.step_seconds(0.1);
        let after_projectile_and_timers = score(&game);
        assert!(after_projectile_and_timers > 0);
        assert_eq!(game.count::<crate::beginner::actors::Pickup>(), 1);
        game.assert_sound_played();

        game.set_enemy_health(0, 0);
        game.step_seconds(0.1);
        let after_wave = score(&game);
        assert!(after_wave > after_projectile_and_timers);

        let player = game.player();
        game.set_entity_health(player, 0);
        game.step_seconds(0.1);
        let after_death = score(&game);
        assert!(after_death > after_wave);

        game.reset_to_start_map().unwrap();
        game.step_seconds(0.1);
        let after_respawn = score(&game);
        assert!(after_respawn > after_death);

        let player = game.player();
        game.move_entity_to(player, glam::vec2(72.0, 24.0));
        game.frame(0.1);
        game.assert_map("level_2");
        assert!(score(&game) > after_respawn);
    }

    struct AreaEventPlugin {
        collisions: Rc<Cell<u32>>,
        enters: Rc<Cell<u32>>,
        exits: Rc<Cell<u32>>,
    }

    impl GamePlugin for AreaEventPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> anyhow::Result<()> {
            game.asset_bag()
                .texture("player", "textures/player.png")?
                .texture("floor", "textures/floor.png")?
                .texture("wall", "textures/wall.png")?
                .build();
            let controls = game.input(|input| input.top_down_controls())?;

            game.player_prefab("player")
                .sprite("player")
                .moves_with(controls.movement, 120.0)
                .build()?;
            game.trigger_prefab("danger_zone")
                .size(glam::vec2(64.0, 64.0))
                .trigger_only()
                .build()?;
            game.map("areas")
                .tile_size(32.0)
                .tiles(["#####", "#P..#", "#####"])
                .simple_theme("floor", "wall")
                .legend('P', "player")
                .spawn("danger", "danger_zone", cell(1, 1))
                .start();
            game.on_start(|game| game.spawn_start_map());

            let collisions = Rc::clone(&self.collisions);
            game.on_collision("player", "danger_zone", move |event| {
                let _ = event.a().position();
                let _ = event.b().position();
                let _ = event.actor().position();
                let _ = event.area().position();
                let _ = event.other().position();
                collisions.set(collisions.get() + 1);
            });
            let enters = Rc::clone(&self.enters);
            game.on_enter_area("player", "danger_zone", move |event| {
                event.actor().damage(0);
                enters.set(enters.get() + 1);
            });
            let exits = Rc::clone(&self.exits);
            game.on_exit_area("player", "danger_zone", move |event| {
                let _ = event.area().position();
                exits.set(exits.get() + 1);
            });
            Ok(())
        }
    }

    #[test]
    fn general_collision_and_area_events_track_overlap_lifecycle() {
        let collisions = Rc::new(Cell::new(0));
        let enters = Rc::new(Cell::new(0));
        let exits = Rc::new(Cell::new(0));
        let mut game = GameTestHarness::from_plugin(AreaEventPlugin {
            collisions: Rc::clone(&collisions),
            enters: Rc::clone(&enters),
            exits: Rc::clone(&exits),
        })
        .unwrap();

        assert_eq!(game.count::<crate::beginner::actors::TriggerArea>(), 1);
        assert_eq!(game.count::<crate::beginner::actors::Area>(), 1);
        assert_eq!(game.count::<crate::beginner::actors::AreaName>(), 1);
        assert_eq!(game.count::<Sprite>(), 1, "areas need no sprite by default");

        game.step();
        assert_eq!(collisions.get(), 1, "collision fires while overlapping");
        assert_eq!(enters.get(), 1, "area enter fires once");
        assert_eq!(exits.get(), 0);

        game.step();
        assert_eq!(collisions.get(), 2, "collision continues each tick");
        assert_eq!(enters.get(), 1, "area enter does not repeat");

        game.move_entity_to(game.player(), glam::vec2(200.0, 200.0));
        game.step();
        assert_eq!(collisions.get(), 2);
        assert_eq!(enters.get(), 1);
        assert_eq!(exits.get(), 1, "area exit fires once when overlap ends");
    }
}
