//! Game-shaped actor, collection, camera, and score helpers.

use game_combat::Health;
use game_core::builder::PropertyBag;
use game_core::world::{EntityId, Velocity};
use glam::Vec2;

use crate::assets::SoundRef;
use crate::beginner::actors::{
    CollectSound, DespawnOnCollect, Enemy, HealValue, Pickup, Player, ScoreValue,
};
use crate::context::GameCtx;

const PROJECTILE_DIRECTION_X: &str = "beginner/projectile_direction_x";
const PROJECTILE_DIRECTION_Y: &str = "beginner/projectile_direction_y";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Score {
    pub value: i32,
}

pub struct PlayerActor<'g, 'a, 'w> {
    game: &'g mut GameCtx<'a, 'w>,
}

impl<'g, 'a, 'w> PlayerActor<'g, 'a, 'w> {
    pub(crate) fn new(game: &'g mut GameCtx<'a, 'w>) -> Self {
        Self { game }
    }

    fn id(&self) -> Option<EntityId> {
        self.game.player_id()
    }

    pub fn exists(&self) -> bool {
        self.id().is_some()
    }

    pub fn is_dead(&self) -> bool {
        self.id().is_some_and(|id| self.game.is_dead(id))
    }

    pub fn position(&self) -> Option<Vec2> {
        self.id().and_then(|id| self.game.position(id))
    }

    pub fn health(&self) -> Option<i32> {
        self.id()
            .and_then(|id| self.game.component::<Health>(id))
            .map(|health| health.current)
    }

    pub fn heal(&mut self, amount: i32) -> bool {
        let Some(id) = self.id() else {
            return false;
        };
        let Some(health) = self.game.component_mut::<Health>(id) else {
            return false;
        };
        health.current = (health.current + amount.max(0)).min(health.max);
        true
    }

    pub fn damage(&mut self, amount: i32) -> bool {
        let Some(id) = self.id() else {
            return false;
        };
        self.game.damage_entity(id, amount)
    }

    pub fn kill(&mut self) {
        let Some(id) = self.id() else {
            return;
        };
        if let Some(health) = self.game.component_mut::<Health>(id) {
            health.damage(health.current);
        }
        if let Some(velocity) = self.game.component_mut::<Velocity>(id) {
            velocity.0 = Vec2::ZERO;
        }
    }

    pub fn stop(&mut self) {
        let Some(id) = self.id() else {
            return;
        };
        if let Some(velocity) = self.game.component_mut::<Velocity>(id) {
            velocity.0 = Vec2::ZERO;
        }
    }

    pub fn play_animation(&mut self, name: impl Into<String>) -> bool {
        let Some(id) = self.id() else {
            return false;
        };
        self.game.play_animation(id, name)
    }

    /// Begins a player-owned projectile shot. Direction helpers fire
    /// immediately, keeping the usual beginner form compact:
    /// `game.player().shoot("bolt").towards_mouse();`.
    pub fn shoot(self, prefab: impl Into<String>) -> ShootAuthor<'g, 'a, 'w> {
        let origin = self.id().and_then(|id| self.game.position(id));
        ShootAuthor {
            game: self.game,
            prefab: prefab.into(),
            origin,
        }
    }
}

/// Configures a projectile shot before one of its direction methods fires it.
pub struct ShootAuthor<'g, 'a, 'w> {
    game: &'g mut GameCtx<'a, 'w>,
    prefab: String,
    origin: Option<Vec2>,
}

impl<'g, 'a, 'w> ShootAuthor<'g, 'a, 'w> {
    pub fn towards_mouse(self) -> FiredShot<'g, 'a, 'w> {
        let direction = self
            .origin
            .map(|origin| self.game.mouse_world_position() - origin)
            .unwrap_or(Vec2::X);
        self.direction(direction)
    }

    pub fn right(self) -> FiredShot<'g, 'a, 'w> {
        self.direction(Vec2::X)
    }

    pub fn left(self) -> FiredShot<'g, 'a, 'w> {
        self.direction(-Vec2::X)
    }

    pub fn up(self) -> FiredShot<'g, 'a, 'w> {
        self.direction(-Vec2::Y)
    }

    pub fn down(self) -> FiredShot<'g, 'a, 'w> {
        self.direction(Vec2::Y)
    }

    pub fn direction(self, direction: Vec2) -> FiredShot<'g, 'a, 'w> {
        let direction = direction.normalize_or_zero();
        if let Some(origin) = self.origin {
            let mut properties = PropertyBag::default();
            properties.insert(PROJECTILE_DIRECTION_X, direction.x.to_string());
            properties.insert(PROJECTILE_DIRECTION_Y, direction.y.to_string());
            properties.insert("beginner/projectile_owner", "player");
            self.game
                .spawn_prefab_with_properties_or_log(&self.prefab, origin, properties);
        }
        FiredShot { game: self.game }
    }

    /// Fires straight right when no directional helper is needed.
    pub fn fire(self) -> FiredShot<'g, 'a, 'w> {
        self.right()
    }
}

/// A shot already queued by a [`ShootAuthor`] direction helper. Sound helpers
/// remain chainable without exposing the deferred command queue.
pub struct FiredShot<'g, 'a, 'w> {
    game: &'g mut GameCtx<'a, 'w>,
}

impl<'g, 'a, 'w> FiredShot<'g, 'a, 'w> {
    pub fn play_sound(self, sound: impl Into<SoundRef>) {
        match sound.into() {
            SoundRef::Handle(handle) => self.game.play_sound(handle, 1.0),
            SoundRef::Key(key) => self.game.play_sound_named(&key),
        }
    }

    pub fn play_sound_named(self, key: &str) {
        self.game.play_sound_named(key);
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct EnemyFilter {
    alive_only: bool,
    dead_only: bool,
    near_player: Option<f32>,
}

pub struct EnemyCollection<'g, 'a, 'w> {
    game: &'g mut GameCtx<'a, 'w>,
    filter: EnemyFilter,
}

impl<'g, 'a, 'w> EnemyCollection<'g, 'a, 'w> {
    pub(crate) fn new(game: &'g mut GameCtx<'a, 'w>) -> Self {
        Self {
            game,
            filter: EnemyFilter::default(),
        }
    }

    pub fn alive(mut self) -> Self {
        self.filter.alive_only = true;
        self.filter.dead_only = false;
        self
    }

    pub fn dead(mut self) -> Self {
        self.filter.dead_only = true;
        self.filter.alive_only = false;
        self
    }

    pub fn near_player(mut self, range: f32) -> Self {
        self.filter.near_player = Some(range.max(0.0));
        self
    }

    pub fn count(&self) -> usize {
        self.ids().len()
    }

    pub fn damage(&mut self, amount: i32) -> usize {
        self.ids()
            .into_iter()
            .filter(|id| self.game.damage_entity(*id, amount))
            .count()
    }

    pub fn despawn(&mut self) -> usize {
        let ids = self.ids();
        let mut commands = self.game.commands();
        for id in &ids {
            commands.despawn(*id);
        }
        ids.len()
    }

    pub fn play_animation(&mut self, name: impl Into<String>) -> usize {
        let name = name.into();
        self.ids()
            .into_iter()
            .filter(|id| self.game.play_animation(*id, name.clone()))
            .count()
    }

    fn ids(&self) -> Vec<EntityId> {
        let player_pos = self
            .filter
            .near_player
            .and_then(|_| self.game.player_position());

        self.game
            .entities_with::<Enemy>()
            .into_iter()
            .filter(|id| {
                if self.filter.alive_only && self.game.is_dead(*id) {
                    return false;
                }
                if self.filter.dead_only && !self.game.is_dead(*id) {
                    return false;
                }
                if let Some(range) = self.filter.near_player {
                    let Some(player_pos) = player_pos else {
                        return false;
                    };
                    let Some(enemy_pos) = self.game.position(*id) else {
                        return false;
                    };
                    return enemy_pos.distance(player_pos) <= range;
                }
                true
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct PickupFilter {
    near_player: Option<f32>,
}

pub struct PickupCollection<'g, 'a, 'w> {
    game: &'g mut GameCtx<'a, 'w>,
    filter: PickupFilter,
}

impl<'g, 'a, 'w> PickupCollection<'g, 'a, 'w> {
    pub(crate) fn new(game: &'g mut GameCtx<'a, 'w>) -> Self {
        Self {
            game,
            filter: PickupFilter::default(),
        }
    }

    pub fn near_player(mut self, range: f32) -> Self {
        self.filter.near_player = Some(range.max(0.0));
        self
    }

    pub fn count(&self) -> usize {
        self.ids().len()
    }

    pub fn collect(&mut self) -> usize {
        let ids = self.ids();
        self.collect_ids(ids)
    }

    pub(crate) fn collect_ids(&mut self, ids: Vec<EntityId>) -> usize {
        let mut score_delta = 0;
        let mut heal_delta = 0;
        let mut sounds = Vec::new();
        let mut despawn = Vec::new();

        for id in &ids {
            if let Some(score) = self.game.component::<ScoreValue>(*id) {
                score_delta += score.0;
            }
            if let Some(heal) = self.game.component::<HealValue>(*id) {
                heal_delta += heal.0;
            }
            if let Some(sound) = self.game.component::<CollectSound>(*id) {
                sounds.push(sound.0);
            }
            if self.game.has::<DespawnOnCollect>(*id) {
                despawn.push(*id);
            }
        }

        if score_delta != 0 {
            self.game.score().add(score_delta);
        }
        if heal_delta != 0 {
            self.game.player().heal(heal_delta);
        }

        let mut commands = self.game.commands();
        for sound in sounds {
            commands.play_sound(sound);
        }
        for id in despawn {
            commands.despawn(id);
        }

        ids.len()
    }

    pub(crate) fn ids(&self) -> Vec<EntityId> {
        let player_pos = self
            .filter
            .near_player
            .and_then(|_| self.game.player_position());

        self.game
            .entities_with::<Pickup>()
            .into_iter()
            .filter(|id| {
                if let Some(range) = self.filter.near_player {
                    let Some(player_pos) = player_pos else {
                        return false;
                    };
                    let Some(pickup_pos) = self.game.position(*id) else {
                        return false;
                    };
                    return pickup_pos.distance(player_pos) <= range;
                }
                true
            })
            .collect()
    }
}

pub struct CameraOps<'g, 'a, 'w> {
    game: &'g mut GameCtx<'a, 'w>,
}

impl<'g, 'a, 'w> CameraOps<'g, 'a, 'w> {
    pub(crate) fn new(game: &'g mut GameCtx<'a, 'w>) -> Self {
        Self { game }
    }

    pub fn follow_player(&mut self) {
        self.game.camera_follow_first::<Player>();
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.game.camera_mut().set_zoom(zoom);
    }

    pub fn shake(&mut self, seconds: f32) {
        self.game.shake_camera(seconds);
    }
}

pub struct ScoreOps<'g, 'a, 'w> {
    game: &'g mut GameCtx<'a, 'w>,
}

impl<'g, 'a, 'w> ScoreOps<'g, 'a, 'w> {
    pub(crate) fn new(game: &'g mut GameCtx<'a, 'w>) -> Self {
        Self { game }
    }

    pub fn add(&mut self, amount: i32) {
        self.game.resource_or_insert_with(Score::default).value += amount;
    }

    pub fn value(&mut self) -> i32 {
        self.game.resource_or_insert_with(Score::default).value
    }

    pub fn reset(&mut self) {
        self.game.insert_resource(Score::default());
    }
}

impl<'a, 'w> GameCtx<'a, 'w> {
    pub fn player(&mut self) -> PlayerActor<'_, 'a, 'w> {
        PlayerActor::new(self)
    }

    pub fn enemies(&mut self) -> EnemyCollection<'_, 'a, 'w> {
        EnemyCollection::new(self)
    }

    pub fn pickups(&mut self) -> PickupCollection<'_, 'a, 'w> {
        PickupCollection::new(self)
    }

    pub fn collect_pickups_near_player(&mut self, range: f32) -> usize {
        self.pickups().near_player(range).collect()
    }

    pub(crate) fn pickup_ids_near_player(&mut self, range: f32) -> Vec<EntityId> {
        self.pickups().near_player(range).ids()
    }

    pub(crate) fn collect_pickup(&mut self, id: EntityId) -> bool {
        self.pickups().collect_ids(vec![id]) > 0
    }

    pub fn camera2d(&mut self) -> CameraOps<'_, 'a, 'w> {
        CameraOps::new(self)
    }

    pub fn score(&mut self) -> ScoreOps<'_, 'a, 'w> {
        ScoreOps::new(self)
    }
}

#[cfg(test)]
mod tests {
    use game_core::backend::{SoundHandle, TextureHandle};

    use super::Score;
    use crate::app::{GameApp, GamePlugin};
    use crate::beginner::actors::Pickup;
    use crate::context::{GameCtx, StartupGameCtx};
    use crate::harness::GameTestHarness;

    struct CollectionPlugin;

    impl GamePlugin for CollectionPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> anyhow::Result<()> {
            let controls = game.input(|input| input.top_down_controls())?;

            game.player_prefab("player")
                .sprite(TextureHandle(1))
                .moves_with(controls.movement, 130.0)
                .health(100)
                .build()?;

            game.enemy_prefab("slime")
                .sprite(TextureHandle(2))
                .health(40)
                .build()?;

            game.pickup_prefab("coin")
                .sprite(TextureHandle(3))
                .score(3)
                .play_sound(SoundHandle(1))
                .despawn_on_collect()
                .build()?;

            game.map("collections")
                .tiles(["#####", "#PEC#", "#####"])
                .simple_theme(TextureHandle(10), TextureHandle(11))
                .legend('P', "player")
                .legend('E', "slime")
                .legend('C', "coin")
                .start();

            game.on_start(|game: &mut StartupGameCtx<'_, '_>| game.spawn_start_map());
            game.every_tick(|game: &mut GameCtx<'_, '_>, _dt| {
                game.player().damage(10);
                game.player().heal(5);
                game.enemies().alive().near_player(80.0).damage(50);
                game.enemies().dead().despawn();
                let collected = game.pickups().near_player(80.0).collect();
                if collected > 0 {
                    game.score().add(2);
                }
                game.camera2d().follow_player();
                game.camera2d().set_zoom(2.0);
            });

            Ok(())
        }
    }

    #[test]
    fn collection_wrappers_drive_common_beginner_rules() {
        let mut game = GameTestHarness::from_plugin(CollectionPlugin).unwrap();

        game.step();

        assert_eq!(game.player().health(), 95);
        assert_eq!(game.enemy_count(), 0);
        assert_eq!(game.count::<Pickup>(), 0);
        assert_eq!(game.world().get_resource::<Score>().unwrap().value, 5);
        game.assert_sound_played();
    }
}
