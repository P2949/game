//! Declarative beginner rules builder.

use std::collections::HashMap;

use glam::Vec2;

use anyhow::Result;

use crate::app::{GameApp, GamePlugin};
use crate::beginner::actors::{
    Checkpoint, CheckpointState, DeathAnimationPolicy, DespawnOnHit, Door, DoorAction, DoorTarget,
    DropSpawned, DropsPrefab, Enemy, Lifetime, PlayerProjectile, PrefabName, Projectile,
    ProjectileDamage, ProjectileImpact, SpawnPlacement, Spawner,
};
use crate::beginner::defaults::{
    enemy_directional_animation_system, player_directional_animation_system,
};
use crate::beginner::events::DEFAULT_PICKUP_COLLECT_RANGE;
use crate::beginner::state::SimpleGameState;
use crate::context::GameCtx;
use crate::diagnostics::bad_rule_combo_error;
use crate::input::TopDownControls;

const DEFAULT_DOOR_TRIGGER_RANGE: f32 = 28.0;

pub struct RulesAuthor<'a, 'app> {
    app: &'a mut GameApp<'app>,
    top_down: Option<TopDownControls>,
    collect_pickups: bool,
    doors_change_maps: bool,
    enemies_damage_player: bool,
    dead_enemies_despawn: bool,
    camera_follows_player: bool,
    pause_and_reset: bool,
    show_score: bool,
    show_enemy_count: bool,
    show_player_health: bool,
    show_menu: bool,
    show_pause_menu: bool,
    show_game_over_panel: bool,
    show_win_panel: bool,
    projectiles_move: bool,
    projectiles_expire: bool,
    projectiles_damage_enemies: bool,
    projectiles_despawn_on_hit: bool,
    projectile_impact_before_despawn: bool,
    spawners_spawn_prefabs: bool,
    enemies_animate_by_movement: bool,
    player_directional_animation: bool,
    enemies_directional_animation: bool,
    directional_attack_animation: bool,
    dead_enemies_play_death_animation: bool,
    dead_enemies_despawn_after_animation: bool,
    enemy_drops: bool,
    win_when_all_pickups_collected: bool,
    win_when_all_enemies_dead: bool,
    player_activates_checkpoints: bool,
    respawn_at_checkpoint: bool,
}

impl<'a, 'app> RulesAuthor<'a, 'app> {
    pub(crate) fn new(app: &'a mut GameApp<'app>) -> Self {
        Self {
            app,
            top_down: None,
            collect_pickups: false,
            doors_change_maps: false,
            enemies_damage_player: false,
            dead_enemies_despawn: false,
            camera_follows_player: false,
            pause_and_reset: false,
            show_score: false,
            show_enemy_count: false,
            show_player_health: false,
            show_menu: false,
            show_pause_menu: false,
            show_game_over_panel: false,
            show_win_panel: false,
            projectiles_move: false,
            projectiles_expire: false,
            projectiles_damage_enemies: false,
            projectiles_despawn_on_hit: false,
            projectile_impact_before_despawn: false,
            spawners_spawn_prefabs: false,
            enemies_animate_by_movement: false,
            player_directional_animation: false,
            enemies_directional_animation: false,
            directional_attack_animation: false,
            dead_enemies_play_death_animation: false,
            dead_enemies_despawn_after_animation: false,
            enemy_drops: false,
            win_when_all_pickups_collected: false,
            win_when_all_enemies_dead: false,
            player_activates_checkpoints: false,
            respawn_at_checkpoint: false,
        }
    }

    pub fn top_down_controls(mut self, controls: TopDownControls) -> Self {
        self.top_down = Some(controls);
        self
    }

    pub fn controls(self, controls: TopDownControls) -> Self {
        self.top_down_controls(controls)
    }

    pub fn player_collects_pickups(mut self) -> Self {
        self.collect_pickups = true;
        self
    }

    pub fn doors_change_maps(mut self) -> Self {
        self.doors_change_maps = true;
        self
    }

    pub fn enemies_damage_player(mut self) -> Self {
        self.enemies_damage_player = true;
        self
    }

    pub fn dead_enemies_despawn(mut self) -> Self {
        self.dead_enemies_despawn = true;
        self
    }

    pub fn camera_follows_player(mut self) -> Self {
        self.camera_follows_player = true;
        self
    }

    pub fn pause_and_reset(mut self) -> Self {
        self.pause_and_reset = true;
        self
    }

    pub fn show_basic_ui(mut self) -> Self {
        self.show_score = true;
        self
    }

    pub fn show_score(mut self) -> Self {
        self.show_score = true;
        self
    }

    pub fn show_enemy_count(mut self) -> Self {
        self.show_enemy_count = true;
        self
    }

    pub fn show_player_health(mut self) -> Self {
        self.show_player_health = true;
        self
    }

    pub fn show_game_over_text(mut self) -> Self {
        self.show_game_over_panel = true;
        self
    }

    /// Draws a conventional title panel while the active scene is named
    /// `menu`. Scene-flow text can still supply a more specific message.
    pub fn show_menu(mut self) -> Self {
        self.show_menu = true;
        self
    }

    /// Draws a conventional paused panel while the beginner game state is
    /// paused.
    pub fn show_pause_menu(mut self) -> Self {
        self.show_pause_menu = true;
        self
    }

    /// Draws a conventional game-over panel when the player dies or the active
    /// scene is named `game_over`.
    pub fn show_game_over_panel(mut self) -> Self {
        self.show_game_over_panel = true;
        self
    }

    /// Draws a conventional win panel while the active scene is named `win`.
    pub fn show_win_panel(mut self) -> Self {
        self.show_win_panel = true;
        self
    }

    pub fn projectiles_move(mut self) -> Self {
        self.projectiles_move = true;
        self
    }

    pub fn projectiles_expire_after_lifetime(mut self) -> Self {
        self.projectiles_expire = true;
        self
    }

    pub fn projectiles_damage_enemies(mut self) -> Self {
        self.projectiles_damage_enemies = true;
        self
    }

    pub fn projectiles_despawn_on_hit(mut self) -> Self {
        self.projectiles_despawn_on_hit = true;
        self
    }

    /// Plays a projectile's optional `impact` clip before removing it after a
    /// hit. Projectiles without that clip keep their normal immediate despawn.
    pub fn projectile_impact_animation_before_despawn(mut self) -> Self {
        self.projectile_impact_before_despawn = true;
        self
    }

    /// Enables the common movement, damage, hit-despawn, and lifetime rules for
    /// player-fired projectiles.
    pub fn projectiles(mut self) -> Self {
        self.projectiles_move = true;
        self.projectiles_expire = true;
        self.projectiles_damage_enemies = true;
        self.projectiles_despawn_on_hit = true;
        self
    }

    pub fn spawners_spawn_prefabs(mut self) -> Self {
        self.spawners_spawn_prefabs = true;
        self
    }

    pub fn animate_enemies_by_movement(mut self) -> Self {
        self.enemies_animate_by_movement = true;
        self
    }

    /// Switches player walk clips by velocity using `walk_up`, `walk_down`,
    /// `walk_left`, and `walk_right` when those clips exist.
    pub fn animate_player_directionally(mut self) -> Self {
        self.player_directional_animation = true;
        self
    }

    /// Switches enemy walk clips by velocity using directional walk names.
    pub fn animate_enemies_directionally(mut self) -> Self {
        self.enemies_directional_animation = true;
        self
    }

    /// Chooses `attack_up`, `attack_down`, `attack_left`, or `attack_right`
    /// when the player attacks, falling back to the ordinary `attack` clip.
    pub fn animate_attacks_directionally(mut self) -> Self {
        self.directional_attack_animation = true;
        self
    }

    pub fn dead_enemies_play_death_animation(mut self) -> Self {
        self.dead_enemies_play_death_animation = true;
        self
    }

    pub fn dead_enemies_despawn_after_animation(mut self) -> Self {
        self.dead_enemies_despawn_after_animation = true;
        self
    }

    /// Spawns each defeated enemy's configured `.drops(...)` prefab.
    pub fn enemy_drops(mut self) -> Self {
        self.enemy_drops = true;
        self
    }

    /// Changes to the conventional `win` scene after the last pickup is gone.
    pub fn win_when_all_pickups_collected(mut self) -> Self {
        self.win_when_all_pickups_collected = true;
        self
    }

    /// Changes to the conventional `win` scene after the last enemy is dead.
    pub fn win_when_all_enemies_dead(mut self) -> Self {
        self.win_when_all_enemies_dead = true;
        self
    }

    /// Records the most recently entered checkpoint marker.
    pub fn player_activates_checkpoints(mut self) -> Self {
        self.player_activates_checkpoints = true;
        self
    }

    /// Restarts the current map and moves the player to the last activated
    /// checkpoint when the player dies.
    pub fn respawn_at_checkpoint(mut self) -> Self {
        self.respawn_at_checkpoint = true;
        self
    }

    pub fn build(self) {
        if let Some(error) = self.dependency_error() {
            panic!("{error}");
        }
        let app = self.app;

        if self.top_down.is_some()
            || self.enemies_damage_player
            || self.camera_follows_player
            || self.pause_and_reset
        {
            let mut top_down = app.use_top_down_game();
            if let Some(controls) = self.top_down {
                top_down = top_down.controls(controls);
            }
            if self.enemies_damage_player {
                top_down = top_down.with_melee_combat().with_enemy_chase();
            }
            if self.camera_follows_player {
                top_down = top_down.with_camera_follow();
            }
            if self.pause_and_reset {
                top_down = top_down.with_pause_death_ui();
            }
            if self.enemies_animate_by_movement {
                top_down = top_down.with_enemy_animation_by_movement();
            }
            if self.player_directional_animation {
                top_down = top_down.with_player_directional_animation();
            }
            if self.enemies_directional_animation {
                top_down = top_down.with_enemy_directional_animation();
            }
            if self.directional_attack_animation {
                top_down = top_down.with_directional_attack_animation();
            }
            top_down.build();
        }

        if self.top_down.is_none()
            && (self.enemies_animate_by_movement
                || self.player_directional_animation
                || self.enemies_directional_animation
                || self.dead_enemies_play_death_animation
                || self.dead_enemies_despawn_after_animation
                || self.projectiles_move
                || self.projectile_impact_before_despawn)
        {
            if self.player_directional_animation {
                app.use_behavior(RulesPlayerDirectionalAnimationBehavior)
                    .expect("player directional animation behavior should register");
            }
            if self.enemies_animate_by_movement {
                app.use_behavior(RulesEnemyAnimationByMovementBehavior)
                    .expect("enemy animation behavior should register");
            }
            if self.enemies_directional_animation {
                app.use_behavior(RulesEnemyDirectionalAnimationBehavior)
                    .expect("enemy directional animation behavior should register");
            }
            app.use_behavior(RulesAnimationUpdateBehavior)
                .expect("rules animation update behavior should register");
        }

        if self.collect_pickups {
            app.use_behavior(CollectPickupsBehavior)
                .expect("collect pickups behavior should register");
        }

        if self.doors_change_maps {
            app.use_behavior(DoorsChangeMapsBehavior)
                .expect("doors behavior should register");
        }

        if self.dead_enemies_despawn {
            app.use_behavior(DeadEnemiesDespawnBehavior)
                .expect("dead enemy despawn behavior should register");
        }

        if self.dead_enemies_play_death_animation {
            app.use_behavior(DeathAnimationBehavior)
                .expect("death animation behavior should register");
        }

        if self.dead_enemies_despawn_after_animation {
            app.use_behavior(DeathAnimationDespawnBehavior)
                .expect("death animation despawn behavior should register");
        }

        if self.enemy_drops {
            app.use_behavior(EnemyDropsBehavior)
                .expect("enemy drops behavior should register");
        }

        if self.player_activates_checkpoints {
            app.use_behavior(CheckpointActivationBehavior)
                .expect("checkpoint activation behavior should register");
        }

        if self.respawn_at_checkpoint {
            app.use_behavior(CheckpointRespawnBehavior)
                .expect("checkpoint respawn behavior should register");
        }

        if self.win_when_all_pickups_collected || self.win_when_all_enemies_dead {
            app.use_behavior(WinConditionBehavior {
                require_pickups: self.win_when_all_pickups_collected,
                require_enemies: self.win_when_all_enemies_dead,
            })
            .expect("win condition behavior should register");
        }

        if self.show_score
            || self.show_enemy_count
            || self.show_player_health
            || self.show_menu
            || self.show_pause_menu
            || self.show_game_over_panel
            || self.show_win_panel
        {
            app.use_behavior(HighLevelUiBehavior {
                show_score: self.show_score,
                show_enemy_count: self.show_enemy_count,
                show_player_health: self.show_player_health,
                show_menu: self.show_menu,
                show_pause_menu: self.show_pause_menu,
                show_game_over_panel: self.show_game_over_panel,
                show_win_panel: self.show_win_panel,
            })
            .expect("high-level UI behavior should register");
        }

        if self.projectiles_move {
            app.use_behavior(ProjectileMovementBehavior)
                .expect("projectile movement behavior should register");
        }

        if self.projectiles_expire {
            app.use_behavior(ProjectileLifetimeBehavior)
                .expect("projectile lifetime behavior should register");
        }

        if self.projectiles_damage_enemies {
            app.use_behavior(ProjectileDamageBehavior {
                despawn_on_hit: self.projectiles_despawn_on_hit,
                impact_before_despawn: self.projectile_impact_before_despawn,
            })
            .expect("projectile damage behavior should register");
        }

        if self.projectile_impact_before_despawn {
            app.use_behavior(ProjectileImpactDespawnBehavior)
                .expect("projectile impact despawn behavior should register");
        }

        if self.spawners_spawn_prefabs {
            app.use_behavior(SpawnerBehavior)
                .expect("spawner behavior should register");
        }
    }

    fn dependency_error(&self) -> Option<anyhow::Error> {
        if self.projectiles_damage_enemies && !self.projectiles_move && !self.projectiles_expire {
            return Some(bad_rule_combo_error(
                "ProjectilesDamageEnemies",
                "Projectiles",
            ));
        }
        if self.projectiles_despawn_on_hit && !self.projectiles_damage_enemies {
            return Some(bad_rule_combo_error(
                "ProjectilesDespawnOnHit",
                "ProjectilesDamageEnemies",
            ));
        }
        if self.projectile_impact_before_despawn
            && !self.projectiles_despawn_on_hit
            && !self.projectiles_damage_enemies
        {
            return Some(bad_rule_combo_error(
                "ProjectileImpactAnimationBeforeDespawn",
                "ProjectilesDespawnOnHit",
            ));
        }
        if self.respawn_at_checkpoint && !self.player_activates_checkpoints {
            return Some(bad_rule_combo_error(
                "RespawnAtCheckpoint",
                "PlayerActivatesCheckpoints",
            ));
        }
        None
    }
}

fn enemy_drops_system(game: &mut GameCtx<'_, '_>, _dt: f32) {
    let drops = game
        .entities_with::<Enemy>()
        .into_iter()
        .filter(|id| game.is_dead(*id) && !game.has::<DropSpawned>(*id))
        .filter_map(|id| {
            let drop = game.component::<DropsPrefab>(id)?.clone();
            let position = game.position(id)?;
            Some((id, position, drop))
        })
        .collect::<Vec<_>>();

    for (enemy, position, drop) in drops {
        if drop.prefab.is_empty() || drop.chance <= 0.0 {
            continue;
        }
        // A stable position-derived roll keeps examples deterministic without
        // adding an RNG dependency. The common `.drops(...)` path always uses
        // chance 1.0.
        let roll = ((position.x.to_bits() ^ position.y.to_bits()) % 10_000) as f32 / 10_000.0;
        if drop.chance >= 1.0 || roll < drop.chance {
            game.spawn_prefab_or_log(&drop.prefab, position);
        }
        game.insert_component(enemy, DropSpawned);
    }
}

fn checkpoint_activation_system(game: &mut GameCtx<'_, '_>, _dt: f32) {
    let Some(player_position) = game.player_position() else {
        return;
    };
    let checkpoint = game
        .entities_with::<Checkpoint>()
        .into_iter()
        .filter(|id| {
            game.component::<Checkpoint>(*id)
                .is_some_and(|checkpoint| checkpoint.enabled)
        })
        .find_map(|id| {
            let position = game.position(id)?;
            let collider = game.component::<game_physics::Collider>(id)?;
            let offset = (player_position - position).abs();
            (offset.x <= collider.half_extents.x && offset.y <= collider.half_extents.y)
                .then_some(position)
        });
    if let Some(position) = checkpoint {
        game.insert_resource(CheckpointState {
            position: Some(position),
        });
    }
}

fn checkpoint_respawn_system(game: &mut GameCtx<'_, '_>, _dt: f32) {
    if let Some(position) = game
        .resource::<PendingCheckpointRespawn>()
        .and_then(|pending| pending.position)
    {
        if let Some(player) = game.player_id() {
            if let Some(transform) = game.component_mut::<game_core::world::Transform>(player) {
                transform.pos = position;
            }
        }
        if let Some(pending) = game.resource_mut::<PendingCheckpointRespawn>() {
            pending.position = None;
        }
        return;
    }

    let Some(position) = game
        .resource::<CheckpointState>()
        .and_then(|checkpoint| checkpoint.position)
    else {
        return;
    };
    if !game.player_is_dead() {
        return;
    }
    // RestartMap is applied after this fixed tick. Remember the intended
    // position so the newly spawned player can be moved on the next tick.
    game.insert_resource(PendingCheckpointRespawn {
        position: Some(position),
    });
    game.restart_map_or_log();
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct PendingCheckpointRespawn {
    position: Option<glam::Vec2>,
}

fn enemy_animation_by_movement_system(game: &mut GameCtx<'_, '_>, _dt: f32) {
    for id in game.entities_with::<crate::beginner::actors::Enemy>() {
        if game.is_dead(id) {
            continue;
        }
        let Some(animation) = game.component::<crate::beginner::animation::Animation>(id) else {
            continue;
        };
        let Some(set) = game.component::<crate::beginner::animation::AnimationSet>(id) else {
            continue;
        };
        if set
            .get(&animation.current)
            .is_some_and(|clip| !clip.looping)
        {
            continue;
        }
        let moving = game
            .component::<game_core::world::Velocity>(id)
            .is_some_and(|velocity| velocity.0.length_squared() > 0.0001);
        game.play_animation(id, if moving { "walk" } else { "idle" });
    }
}

fn dead_enemies_play_death_animation_system(game: &mut GameCtx<'_, '_>, _dt: f32) {
    for id in game.enemy_ids() {
        if !game.is_dead(id) {
            continue;
        }
        if game
            .component::<DeathAnimationPolicy>(id)
            .is_some_and(|policy| policy.despawn_after_animation)
        {
            game.play_animation(id, "die");
        }
    }
}

fn dead_enemies_despawn_after_animation_system(game: &mut GameCtx<'_, '_>, _dt: f32) {
    let despawn = game
        .enemy_ids()
        .into_iter()
        .filter(|id| game.is_dead(*id))
        .filter(|id| {
            game.component::<DeathAnimationPolicy>(*id)
                .is_some_and(|policy| policy.despawn_after_animation)
        })
        .filter(|id| {
            game.component::<crate::beginner::animation::Animation>(*id)
                .is_none_or(|_| game.animation_finished(*id, "die"))
        })
        .collect::<Vec<_>>();
    let mut commands = game.commands();
    for id in despawn {
        commands.despawn(id);
    }
}

fn projectiles_move_system(game: &mut GameCtx<'_, '_>, dt: f32) {
    let velocities = game
        .entities_with::<Projectile>()
        .into_iter()
        .filter_map(|id| {
            game.component::<game_core::world::Velocity>(id)
                .map(|velocity| (id, velocity.0))
        })
        .collect::<Vec<_>>();

    for (id, velocity) in velocities {
        if game.has::<ProjectileImpact>(id) {
            continue;
        }
        if let Some(transform) = game.component_mut::<game_core::world::Transform>(id) {
            transform.pos += velocity * dt.max(0.0);
        }
    }
}

fn projectiles_expire_system(game: &mut GameCtx<'_, '_>, dt: f32) {
    let mut expired = Vec::new();
    for id in game.entities_with::<Projectile>() {
        if game.has::<ProjectileImpact>(id) {
            continue;
        }
        let Some(lifetime) = game.component_mut::<Lifetime>(id) else {
            continue;
        };
        lifetime.seconds_left -= dt.max(0.0);
        if lifetime.seconds_left <= 0.0 {
            expired.push(id);
        }
    }

    let mut commands = game.commands();
    for id in expired {
        commands.despawn(id);
    }
}

fn projectiles_damage_enemies_system(
    game: &mut GameCtx<'_, '_>,
    despawn_on_hit: bool,
    impact_before_despawn: bool,
) {
    const HIT_DISTANCE: f32 = 16.0;

    let enemies = game.living_enemy_ids();
    let projectiles = game.entities_with::<PlayerProjectile>();
    let mut despawn = Vec::new();

    for projectile in projectiles {
        if game.has::<ProjectileImpact>(projectile) {
            continue;
        }
        let Some(position) = game.position(projectile) else {
            continue;
        };
        let Some(damage) = game
            .component::<ProjectileDamage>(projectile)
            .map(|damage| damage.amount)
        else {
            continue;
        };
        let should_despawn = despawn_on_hit && game.has::<DespawnOnHit>(projectile);

        for enemy in &enemies {
            let Some(enemy_position) = game.position(*enemy) else {
                continue;
            };
            if position.distance(enemy_position) > HIT_DISTANCE {
                continue;
            }
            game.damage_entity(*enemy, damage);
            if should_despawn {
                if impact_before_despawn && game.play_animation(projectile, "impact") {
                    game.insert_component(projectile, ProjectileImpact);
                    if let Some(velocity) =
                        game.component_mut::<game_core::world::Velocity>(projectile)
                    {
                        velocity.0 = Vec2::ZERO;
                    }
                } else {
                    despawn.push(projectile);
                }
                break;
            }
        }
    }

    let mut commands = game.commands();
    for id in despawn {
        commands.despawn(id);
    }
}

fn projectile_impact_despawn_system(game: &mut GameCtx<'_, '_>, _dt: f32) {
    let despawn = game
        .entities_with::<ProjectileImpact>()
        .into_iter()
        .filter(|id| game.animation_finished(*id, "impact"))
        .collect::<Vec<_>>();
    let mut commands = game.commands();
    for id in despawn {
        commands.despawn(id);
    }
}

#[derive(Clone)]
struct SpawnRequest {
    prefab: String,
    placement: SpawnPlacement,
    at_spawner: Vec2,
}

fn spawners_spawn_prefabs_system(game: &mut GameCtx<'_, '_>, dt: f32) {
    let spawners = game
        .entities_with::<Spawner>()
        .into_iter()
        .filter_map(|id| {
            let spawner = game.component::<Spawner>(id)?.clone();
            let position = game.position(id)?;
            Some((id, spawner, position))
        })
        .collect::<Vec<_>>();
    let mut pending_by_prefab: HashMap<String, usize> = HashMap::new();
    let mut requests = Vec::new();

    for (id, snapshot, position) in spawners {
        let alive = count_alive_prefab(game, &snapshot.prefab);
        let already_pending = pending_by_prefab
            .get(&snapshot.prefab)
            .copied()
            .unwrap_or_default();
        let mut spawn_count = 0usize;

        if let Some(spawner) = game.component_mut::<Spawner>(id) {
            spawner.timer += dt.max(0.0);
            while spawner.timer >= spawner.every_seconds
                && spawner
                    .max_alive
                    .is_none_or(|max| alive + already_pending + spawn_count < max)
            {
                spawner.timer -= spawner.every_seconds;
                spawn_count += 1;
            }
        }

        if spawn_count > 0 {
            *pending_by_prefab
                .entry(snapshot.prefab.clone())
                .or_default() += spawn_count;
            for _ in 0..spawn_count {
                requests.push(SpawnRequest {
                    prefab: snapshot.prefab.clone(),
                    placement: snapshot.placement.clone(),
                    at_spawner: position,
                });
            }
        }
    }

    for request in requests {
        let position = match request.placement {
            SpawnPlacement::AtSpawner => Some(request.at_spawner),
            SpawnPlacement::NearPlayer { radius } => game
                .player_position()
                .map(|player| player + Vec2::new(radius, 0.0)),
            SpawnPlacement::AtFirstFloor => game.first_floor_center(),
        };
        if let Some(position) = position {
            game.spawn(request.prefab).at_world(position);
        }
    }
}

fn count_alive_prefab(game: &GameCtx<'_, '_>, prefab: &str) -> usize {
    game.entities_with::<PrefabName>()
        .into_iter()
        .filter(|id| {
            game.component::<PrefabName>(*id)
                .is_some_and(|name| name.matches(prefab))
                && !game.is_dead(*id)
        })
        .count()
}

#[derive(Clone, Copy)]
struct HighLevelUiOptions {
    show_score: bool,
    show_enemy_count: bool,
    show_player_health: bool,
    show_menu: bool,
    show_pause_menu: bool,
    show_game_over_panel: bool,
    show_win_panel: bool,
}

fn high_level_ui_system(game: &mut GameCtx<'_, '_>, options: HighLevelUiOptions) {
    let mut ui = game.ui().top_left();
    if options.show_score {
        ui = ui.score_label();
    }
    if options.show_enemy_count {
        ui = ui.enemy_count_label();
    }
    if options.show_player_health {
        ui = ui.player_health_bar();
    }
    ui.build();

    let scene = game.current_scene_name();
    let state = game
        .resource::<SimpleGameState>()
        .copied()
        .unwrap_or_default();
    if options.show_menu && scene.as_deref() == Some("menu") {
        game.ui()
            .panel("Menu")
            .line("Press Space, Enter, or South to Start")
            .center();
    }
    if options.show_pause_menu && state.paused {
        game.ui()
            .panel("Paused")
            .line("Press P or Escape to Resume")
            .center();
    }
    if options.show_game_over_panel && (state.player_dead || scene.as_deref() == Some("game_over"))
    {
        game.ui()
            .panel("Game Over")
            .line("Press R to Restart")
            .center();
    }
    if options.show_win_panel && scene.as_deref() == Some("win") {
        game.ui().panel("You Win!").line("Great work!").center();
    }
}

fn doors_change_maps_system(game: &mut GameCtx<'_, '_>) {
    let Some(player_pos) = game.player_position() else {
        return;
    };

    let actions = game
        .entities_with::<Door>()
        .into_iter()
        .filter_map(|id| {
            let door_pos = game.position(id)?;
            if door_pos.distance(player_pos) > DEFAULT_DOOR_TRIGGER_RANGE {
                return None;
            }

            let target = game.component::<DoorTarget>(id)?.clone();
            if target.requires_all_enemies_dead && game.enemies().alive().count() > 0 {
                return None;
            }
            Some(target.action)
        })
        .collect::<Vec<_>>();

    for action in actions {
        match action {
            DoorAction::ChangeMap(map) => game.change_map_or_log(&map),
            DoorAction::ChangeScene(scene) => game.change_scene_or_log(&scene),
            DoorAction::RestartLevel => game.restart_level(),
        }
    }
}

/// Collects ordinary pickup prefabs near the player every fixed tick.
pub struct CollectPickupsBehavior;

impl GamePlugin for CollectPickupsBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.fixed(|game: &mut GameCtx<'_, '_>, _dt| {
            game.collect_pickups_near_player(DEFAULT_PICKUP_COLLECT_RANGE);
        });
        Ok(())
    }
}

/// Activates nearby door prefabs every fixed tick.
pub struct DoorsChangeMapsBehavior;

impl GamePlugin for DoorsChangeMapsBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.fixed(|game: &mut GameCtx<'_, '_>, _dt| {
            doors_change_maps_system(game);
        });
        Ok(())
    }
}

/// Queues removal of defeated enemies every fixed tick.
pub struct DeadEnemiesDespawnBehavior;

impl GamePlugin for DeadEnemiesDespawnBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.fixed(|game: &mut GameCtx<'_, '_>, _dt| {
            game.enemies().dead().despawn();
        });
        Ok(())
    }
}

/// Starts configured death animations for defeated enemies.
pub struct DeathAnimationBehavior;

impl GamePlugin for DeathAnimationBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.fixed(dead_enemies_play_death_animation_system);
        Ok(())
    }
}

/// Removes enemies after their configured death animation has finished.
pub struct DeathAnimationDespawnBehavior;

impl GamePlugin for DeathAnimationDespawnBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.fixed(dead_enemies_despawn_after_animation_system);
        Ok(())
    }
}

/// Spawns configured drops from defeated enemies.
pub struct EnemyDropsBehavior;

impl GamePlugin for EnemyDropsBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.fixed(enemy_drops_system);
        Ok(())
    }
}

/// Records the checkpoint currently occupied by the player.
pub struct CheckpointActivationBehavior;

impl GamePlugin for CheckpointActivationBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.fixed(checkpoint_activation_system);
        Ok(())
    }
}

/// Restarts a dead player at the last activated checkpoint.
pub struct CheckpointRespawnBehavior;

impl GamePlugin for CheckpointRespawnBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.fixed(checkpoint_respawn_system);
        Ok(())
    }
}

/// Moves all projectile prefabs according to their velocity.
pub struct ProjectileMovementBehavior;

impl GamePlugin for ProjectileMovementBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.fixed(projectiles_move_system);
        Ok(())
    }
}

/// Expires projectile prefabs when their configured lifetime runs out.
pub struct ProjectileLifetimeBehavior;

impl GamePlugin for ProjectileLifetimeBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.fixed(projectiles_expire_system);
        Ok(())
    }
}

/// Damages enemies hit by player projectiles.
pub struct ProjectileDamageBehavior {
    pub despawn_on_hit: bool,
    pub impact_before_despawn: bool,
}

impl GamePlugin for ProjectileDamageBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        let despawn_on_hit = self.despawn_on_hit;
        let impact_before_despawn = self.impact_before_despawn;
        game.fixed(move |game: &mut GameCtx<'_, '_>, _dt| {
            projectiles_damage_enemies_system(game, despawn_on_hit, impact_before_despawn);
        });
        Ok(())
    }
}

/// Removes projectile impact animations after they finish.
pub struct ProjectileImpactDespawnBehavior;

impl GamePlugin for ProjectileImpactDespawnBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.fixed(projectile_impact_despawn_system);
        Ok(())
    }
}

/// Advances author-configured spawners every fixed tick.
pub struct SpawnerBehavior;

impl GamePlugin for SpawnerBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.fixed(spawners_spawn_prefabs_system);
        Ok(())
    }
}

/// Changes to the conventional `win` scene once selected objectives are met.
pub struct WinConditionBehavior {
    pub require_pickups: bool,
    pub require_enemies: bool,
}

impl GamePlugin for WinConditionBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        let require_pickups = self.require_pickups;
        let require_enemies = self.require_enemies;
        game.fixed(move |game: &mut GameCtx<'_, '_>, _dt| {
            let pickups_done = !require_pickups || game.pickups().count() == 0;
            let enemies_done = !require_enemies || game.enemies().alive().count() == 0;
            if pickups_done && enemies_done {
                game.change_scene_or_log("win");
            }
        });
        Ok(())
    }
}

/// Draws the selected high-level UI elements and panels.
pub struct HighLevelUiBehavior {
    pub show_score: bool,
    pub show_enemy_count: bool,
    pub show_player_health: bool,
    pub show_menu: bool,
    pub show_pause_menu: bool,
    pub show_game_over_panel: bool,
    pub show_win_panel: bool,
}

impl GamePlugin for HighLevelUiBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        let options = HighLevelUiOptions {
            show_score: self.show_score,
            show_enemy_count: self.show_enemy_count,
            show_player_health: self.show_player_health,
            show_menu: self.show_menu,
            show_pause_menu: self.show_pause_menu,
            show_game_over_panel: self.show_game_over_panel,
            show_win_panel: self.show_win_panel,
        };
        game.ui(move |game: &mut GameCtx<'_, '_>, _dt| {
            high_level_ui_system(game, options);
        });
        Ok(())
    }
}

/// Updates ordinary enemy walk and idle animations for rules-only games.
pub struct RulesEnemyAnimationByMovementBehavior;

impl GamePlugin for RulesEnemyAnimationByMovementBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.update(enemy_animation_by_movement_system);
        Ok(())
    }
}

/// Updates player directional walk animations for rules-only games.
pub struct RulesPlayerDirectionalAnimationBehavior;

impl GamePlugin for RulesPlayerDirectionalAnimationBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.update(player_directional_animation_system);
        Ok(())
    }
}

/// Updates enemy directional walk animations for rules-only games.
pub struct RulesEnemyDirectionalAnimationBehavior;

impl GamePlugin for RulesEnemyDirectionalAnimationBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.update(enemy_directional_animation_system);
        Ok(())
    }
}

/// Advances animations for rules-only games that do not install the preset.
pub struct RulesAnimationUpdateBehavior;

impl GamePlugin for RulesAnimationUpdateBehavior {
    fn build(&self, game: &mut GameApp<'_>) -> Result<()> {
        game.update(|game: &mut GameCtx<'_, '_>, dt| game.update_animations(dt));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use game_core::backend::TextureHandle;
    use game_core::world::{Transform, Velocity};

    use crate::app::{GameApp, GamePlugin};
    use crate::beginner::actors::{
        Checkpoint, CheckpointState, Enemy, FacingDirection, Pickup, PlayerProjectile, Projectile,
        Spawner,
    };
    use crate::beginner::animation::{Animation, AnimationSet, SpriteSheet};
    use crate::beginner::collections::Score;
    use crate::harness::GameTestHarness;

    struct RulesPlugin;

    impl GamePlugin for RulesPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> anyhow::Result<()> {
            let controls = game.input(|input| input.top_down_controls())?;

            game.player_prefab("player")
                .sprite(TextureHandle(1))
                .moves_with(controls.movement, 130.0)
                .build()?;

            game.enemy_prefab("slime")
                .sprite(TextureHandle(2))
                .health(10)
                .build()?;

            game.pickup_prefab("coin")
                .sprite(TextureHandle(3))
                .score(1)
                .despawn_on_collect()
                .build()?;

            game.map("rules")
                .tile_size(16.0)
                .tiles(["#####", "#PCE#", "#####"])
                .simple_theme(TextureHandle(10), TextureHandle(11))
                .legend('P', "player")
                .legend('C', "coin")
                .legend('E', "slime")
                .start();

            game.every_tick(|game, _dt| {
                game.enemies().alive().damage(100);
            });

            game.rules()
                .top_down_controls(controls)
                .player_collects_pickups()
                .dead_enemies_despawn()
                .camera_follows_player()
                .pause_and_reset()
                .show_basic_ui()
                .show_enemy_count()
                .show_player_health()
                .show_game_over_text()
                .show_pause_menu()
                .build();

            Ok(())
        }
    }

    struct InvalidRuleComboPlugin;

    impl GamePlugin for InvalidRuleComboPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> anyhow::Result<()> {
            game.rules().projectiles_damage_enemies().build();
            Ok(())
        }
    }

    #[test]
    fn rules_builder_reports_missing_rule_dependencies() {
        let panic = std::panic::catch_unwind(|| {
            let _ = GameTestHarness::from_plugin(InvalidRuleComboPlugin);
        })
        .expect_err("invalid rule combination should panic with a beginner diagnostic");
        let message = panic
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| panic.downcast_ref::<&str>().copied())
            .unwrap_or("<non-string panic>");

        assert!(message.contains("Rule `projectiles_damage_enemies` needs the `projectiles` rule"));
        assert!(message.contains("Add `.projectiles()`"));
    }

    #[test]
    fn rules_builder_registers_common_beginner_rules() {
        let mut game = GameTestHarness::from_plugin(RulesPlugin).unwrap();

        game.step();

        assert_eq!(game.enemy_count(), 0);
        assert_eq!(game.count::<Pickup>(), 0);
        assert_eq!(game.world().get_resource::<Score>().unwrap().value, 1);
        game.frame(0.0);
        game.assert_ui_contains("Score: 1");
        game.assert_ui_contains("Enemies: 0");
        game.assert_ui_contains("Health:");

        let player = game.player();
        game.set_entity_health(player, 0);
        game.frame(0.0);
        game.assert_ui_contains("Game Over");

        game.tap_action("pause");
        game.frame(0.0);
        game.assert_ui_contains("Paused");
    }

    struct ProjectileRulesPlugin;

    impl GamePlugin for ProjectileRulesPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> anyhow::Result<()> {
            game.asset_bag()
                .texture("player", "textures/player.png")?
                .texture("slime", "textures/slime.png")?
                .texture("bolt", "textures/bolt.png")?
                .texture("floor", "textures/floor.png")?
                .texture("wall", "textures/wall.png")?
                .build();
            let controls = game.input(|input| input.top_down_controls())?;

            game.player_prefab("player")
                .sprite("player")
                .moves_with(controls.movement, 120.0)
                .build()?;
            game.enemy_prefab("slime")
                .sprite("slime")
                .health(20)
                .build()?;
            game.projectile_prefab("bolt")
                .sprite("bolt")
                .speed(120.0)
                .damage(20)
                .lifetime(0.25)
                .despawn_on_hit()
                .build()?;
            game.map("projectiles")
                .tile_size(16.0)
                .tiles(["#######", "#P..E.#", "#######"])
                .simple_theme("floor", "wall")
                .legend('P', "player")
                .legend('E', "slime")
                .start();
            game.on_start(|game| game.spawn_start_map());

            game.rules().projectiles().build();
            game.on_action(controls.attack, |game| {
                game.player().shoot("bolt").right();
            });
            game.on_action(controls.reset, |game| {
                game.player().shoot("bolt").left();
            });
            Ok(())
        }
    }

    #[test]
    fn projectile_rules_move_damage_expire_and_despawn_player_shots() {
        let mut hit_game = GameTestHarness::from_plugin(ProjectileRulesPlugin).unwrap();
        hit_game.tap_action("attack");
        assert_eq!(hit_game.count::<Projectile>(), 1);
        let projectile = hit_game.world().ids_with::<Projectile>()[0];
        assert_eq!(hit_game.count::<PlayerProjectile>(), 1);
        assert_eq!(
            hit_game.world().get::<Velocity>(projectile).unwrap().0,
            glam::vec2(120.0, 0.0)
        );
        let enemy = hit_game.world().ids_with::<Enemy>()[0];
        assert_eq!(
            hit_game.world().get::<Transform>(projectile).unwrap().pos,
            glam::vec2(24.0, 24.0)
        );
        assert_eq!(
            hit_game.world().get::<Transform>(enemy).unwrap().pos,
            glam::vec2(72.0, 24.0)
        );
        hit_game.fixed_step(0.5);
        assert_eq!(hit_game.count::<Projectile>(), 0);
        hit_game.assert_enemy_dead(0);

        let mut expiry_game = GameTestHarness::from_plugin(ProjectileRulesPlugin).unwrap();
        expiry_game.tap_action("reset");
        assert_eq!(expiry_game.count::<Projectile>(), 1);
        expiry_game.fixed_step(0.3);
        assert_eq!(expiry_game.count::<Projectile>(), 0);
        assert_eq!(expiry_game.count::<Enemy>(), 1);
    }

    struct SpawnerRulesPlugin;

    impl GamePlugin for SpawnerRulesPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> anyhow::Result<()> {
            game.asset_bag()
                .texture("player", "textures/player.png")?
                .texture("slime", "textures/slime.png")?
                .texture("floor", "textures/floor.png")?
                .texture("wall", "textures/wall.png")?
                .build();
            let controls = game.input(|input| input.top_down_controls())?;
            game.player_prefab("player")
                .sprite("player")
                .moves_with(controls.movement, 120.0)
                .build()?;
            game.enemy_prefab("slime")
                .sprite("slime")
                .health(10)
                .build()?;
            game.spawner_prefab("spawner")
                .spawn("slime")
                .every_seconds(0.2)
                .max_alive(2)
                .near_player(32.0)
                .build()?;
            game.map("waves")
                .tile_size(16.0)
                .tiles(["#######", "#P...S#", "#######"])
                .simple_theme("floor", "wall")
                .legend('P', "player")
                .legend('S', "spawner")
                .start();
            game.on_start(|game| game.spawn_start_map());
            game.rules().spawners_spawn_prefabs().build();
            Ok(())
        }
    }

    #[test]
    fn spawner_rule_places_prefabs_near_player_and_respects_max_alive() {
        let mut game = GameTestHarness::from_plugin(SpawnerRulesPlugin).unwrap();
        assert_eq!(game.count::<Spawner>(), 1);

        game.fixed_step(0.2);
        assert_eq!(game.enemy_count(), 1);
        assert_eq!(
            game.enemy(0).position(),
            game.player().position() + glam::vec2(32.0, 0.0)
        );

        game.fixed_step(0.2);
        assert_eq!(game.enemy_count(), 2);
        game.fixed_step(0.2);
        assert_eq!(game.enemy_count(), 2);
    }

    struct AnimatedEnemyRulesPlugin;

    impl GamePlugin for AnimatedEnemyRulesPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> anyhow::Result<()> {
            let sheet = SpriteSheet::new(TextureHandle(2), 4, 1);
            game.enemy_prefab("slime")
                .spritesheet(sheet)
                .idle(0..1)
                .walk(1..2)
                .die(2..4)
                .despawn_after_death_animation()
                .build()?;
            game.map("animated-enemy")
                .tiles(["###", "#E#", "###"])
                .simple_theme(TextureHandle(10), TextureHandle(11))
                .legend('E', "slime")
                .start();
            game.on_start(|game| game.spawn_start_map());
            game.rules()
                .animate_enemies_by_movement()
                .dead_enemies_play_death_animation()
                .dead_enemies_despawn_after_animation()
                .build();
            Ok(())
        }
    }

    #[test]
    fn animation_rules_drive_enemy_movement_and_despawn_after_death_clip() {
        let mut game = GameTestHarness::from_plugin(AnimatedEnemyRulesPlugin).unwrap();
        let enemy = game.enemy(0);
        game.world_mut().get_mut::<Velocity>(enemy.id()).unwrap().0 = glam::Vec2::X;
        game.frame(0.0);
        assert_eq!(
            game.world().get::<Animation>(enemy.id()).unwrap().current,
            "walk"
        );

        game.set_enemy_health(0, 0);
        game.step();
        assert_eq!(
            game.world().get::<Animation>(enemy.id()).unwrap().current,
            "die"
        );
        assert!(game.world().get::<AnimationSet>(enemy.id()).is_some());

        game.frame(0.4);
        game.step();
        game.assert_enemy_count(0);
    }

    struct RecipeRulesPlugin;

    impl GamePlugin for RecipeRulesPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> anyhow::Result<()> {
            game.start_scene("game").scene("win");
            let controls = game.input(|input| input.top_down_controls())?;

            game.player_prefab("player")
                .sprite(TextureHandle(1))
                .health(100)
                .moves_with(controls.movement, 130.0)
                .build()?;
            game.pickup_prefab("heart")
                .sprite(TextureHandle(2))
                .heal_player(25)
                .despawn_on_collect()
                .build()?;
            game.enemy_prefab("slime")
                .sprite(TextureHandle(3))
                .health(10)
                .drops("heart")
                .build()?;
            game.checkpoint_prefab("checkpoint")
                .sprite(TextureHandle(4))
                .build()?;

            game.map("recipes")
                .tile_size(16.0)
                .tiles(["#########", "#PHEK...#", "#########"])
                .simple_theme(TextureHandle(10), TextureHandle(11))
                .legend('P', "player")
                .legend('H', "heart")
                .legend('E', "slime")
                .legend('K', "checkpoint")
                .start();
            game.on_start(|game| game.spawn_start_map());

            game.rules()
                .player_collects_pickups()
                .enemy_drops()
                .win_when_all_pickups_collected()
                .win_when_all_enemies_dead()
                .player_activates_checkpoints()
                .respawn_at_checkpoint()
                .build();
            Ok(())
        }
    }

    #[test]
    fn recipe_rules_heal_drop_win_and_respawn_at_checkpoints() {
        let mut game = GameTestHarness::from_plugin(RecipeRulesPlugin).unwrap();
        game.assert_scene("game");

        let player = game.player();
        game.set_entity_health(player, 50);
        game.collect_first_pickup();
        game.assert_player_health(75);
        assert_eq!(game.count::<Pickup>(), 0);

        game.set_enemy_health(0, 0);
        game.step();
        assert_eq!(
            game.count::<Pickup>(),
            1,
            "a defeated enemy drops its prefab"
        );
        game.assert_scene("win");

        let checkpoint = game.world().ids_with::<Checkpoint>()[0];
        let checkpoint_position = game.world().get::<Transform>(checkpoint).unwrap().pos;
        let player = game.player();
        game.move_entity_to(player, checkpoint_position);
        game.step();
        assert_eq!(
            game.world()
                .get_resource::<CheckpointState>()
                .unwrap()
                .position,
            Some(checkpoint_position)
        );

        let player = game.player();
        game.set_entity_health(player, 0);
        game.step();
        game.step();
        game.assert_player_health(100);
        assert_eq!(game.player().position(), checkpoint_position);
    }

    struct DirectionalAttackPlugin;

    impl GamePlugin for DirectionalAttackPlugin {
        fn build(&self, game: &mut GameApp<'_>) -> anyhow::Result<()> {
            let controls = game.input(|input| input.top_down_controls())?;
            let sheet = SpriteSheet::new(TextureHandle(1), 4, 1);
            game.player_prefab("player")
                .spritesheet(sheet)
                .idle(0..1)
                .attack(0..1)
                .attack_up(0..1)
                .attack_down(1..2)
                .attack_left(2..3)
                .attack_right(3..4)
                .moves_with(controls.movement, 130.0)
                .build()?;
            game.map("directional-attack")
                .tiles(["###", "#P#", "###"])
                .simple_theme(TextureHandle(10), TextureHandle(11))
                .legend('P', "player")
                .start();
            game.rules()
                .top_down_controls(controls)
                .animate_attacks_directionally()
                .build();
            Ok(())
        }
    }

    #[test]
    fn directional_attack_rule_uses_remembered_facing_and_one_shot_clips() {
        let mut game = GameTestHarness::from_plugin(DirectionalAttackPlugin).unwrap();

        game.tap_action("attack");
        let player = game.player();
        assert_eq!(
            game.world().get::<Animation>(player.id()).unwrap().current,
            "attack_down"
        );

        *game
            .world_mut()
            .get_mut::<FacingDirection>(player.id())
            .unwrap() = FacingDirection::Left;
        game.tap_action("attack");
        assert_eq!(
            game.world().get::<Animation>(player.id()).unwrap().current,
            "attack_left"
        );
    }
}
