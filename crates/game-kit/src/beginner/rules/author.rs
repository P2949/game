use super::animation::{
    DeathAnimationBehavior, DeathAnimationDespawnBehavior, RulesAnimationUpdateBehavior,
    RulesEnemyAnimationByMovementBehavior, RulesEnemyDirectionalAnimationBehavior,
    RulesPlayerDirectionalAnimationBehavior,
};
use super::checkpoints::{CheckpointActivationBehavior, CheckpointRespawnBehavior};
use super::combat::{DeadEnemiesDespawnBehavior, EnemyDropsBehavior};
use super::doors::DoorsChangeMapsBehavior;
use super::pickups::CollectPickupsBehavior;
use super::projectiles::{
    ProjectileDamageBehavior, ProjectileImpactDespawnBehavior, ProjectileLifetimeBehavior,
    ProjectileMovementBehavior,
};
use super::shared::*;
use super::spawners::SpawnerBehavior;
use super::ui::HighLevelUiBehavior;
use super::win_conditions::WinConditionBehavior;

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
