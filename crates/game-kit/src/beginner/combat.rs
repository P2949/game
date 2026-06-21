//! Beginner player/enemy query and melee-combat helpers.

use game_combat::{Health, MeleeAttack};
use game_core::backend::SoundHandle;
use game_core::input::ActionId;
use game_core::world::{EntityId, Transform, Velocity};
use glam::Vec2;

use crate::beginner::actors::{Enemy, FacingDirection, Player};
use crate::context::GameCtx;

#[derive(Clone, Copy, Debug)]
pub struct MeleeCombatConfig {
    pub attack: ActionId,
    pub hit_sound: Option<SoundHandle>,
    pub despawn_dead_enemies: bool,
    pub player_attack_animation: Option<&'static str>,
    pub directional_player_attack_animation: bool,
}

impl<'a, 'w> GameCtx<'a, 'w> {
    pub fn player_id(&self) -> Option<EntityId> {
        self.first_entity_with::<Player>()
    }

    pub fn player_position(&self) -> Option<Vec2> {
        self.player_id().and_then(|id| self.position(id))
    }

    pub fn player_is_dead(&self) -> bool {
        self.entities_with::<Player>()
            .into_iter()
            .any(|id| self.is_dead(id))
    }

    pub fn kill_player(&mut self) {
        for id in self.entities_with::<Player>() {
            if let Some(health) = self.component_mut::<Health>(id) {
                health.damage(health.current);
            }
            if let Some(velocity) = self.component_mut::<Velocity>(id) {
                velocity.0 = Vec2::ZERO;
            }
        }
    }

    pub fn stop_player(&mut self) {
        for id in self.entities_with::<Player>() {
            if let Some(velocity) = self.component_mut::<Velocity>(id) {
                velocity.0 = Vec2::ZERO;
            }
        }
    }

    pub fn enemy_ids(&self) -> Vec<EntityId> {
        self.entities_with::<Enemy>()
    }

    pub fn living_enemy_ids(&self) -> Vec<EntityId> {
        self.living_entities_with::<Enemy>()
    }

    pub fn nearest_living_enemy_to(&self, origin: Vec2, range: f32) -> Option<EntityId> {
        self.nearest_living_with::<Enemy>(origin, range)
    }

    pub fn damage_entity(&mut self, id: EntityId, amount: i32) -> bool {
        self.damage(id, amount)
    }

    pub fn damage_nearest_enemy_to(&mut self, origin: Vec2, range: f32, damage: i32) -> bool {
        self.nearest_living_enemy_to(origin, range)
            .is_some_and(|id| self.damage_entity(id, damage))
    }

    pub fn despawn_dead_enemies(&mut self) {
        let dead = self
            .enemy_ids()
            .into_iter()
            .filter(|id| self.is_dead(*id))
            .collect::<Vec<_>>();
        let mut commands = self.commands();
        for id in dead {
            commands.despawn(id);
        }
    }

    pub fn despawn_dead_enemies_with_optional_sound(&mut self, sound: Option<SoundHandle>) {
        let dead = self
            .enemy_ids()
            .into_iter()
            .filter(|id| self.is_dead(*id))
            .collect::<Vec<_>>();
        let mut commands = self.commands();
        for id in dead {
            if let Some(sound) = sound {
                commands.play_sound(sound);
            }
            commands.despawn(id);
        }
    }

    pub fn player_melee_attack_nearest_enemy(&mut self, hit_sound: Option<SoundHandle>) {
        let Some((_, player_pos, range, damage)) = self.player_melee_snapshot() else {
            return;
        };

        if self.damage_nearest_enemy_to(player_pos, range, damage) {
            if let Some(sound) = hit_sound {
                self.commands().play_sound(sound);
            }
        }
    }

    pub fn enemies_melee_attack_player(&mut self, dt: f32, hit_sound: Option<SoundHandle>) {
        let Some((player_id, player_pos)) = self.player_id().and_then(|id| {
            let pos = self.position(id)?;
            Some((id, pos))
        }) else {
            return;
        };

        let mut player_damage_taken = 0;
        for id in self.living_enemy_ids() {
            let Some(enemy_pos) = self.position(id) else {
                continue;
            };
            let Some(attack) = self.melee_attack_mut(id) else {
                continue;
            };
            attack.timer = (attack.timer - dt).max(0.0);
            if attack.timer == 0.0 && enemy_pos.distance(player_pos) <= attack.range {
                attack.timer = attack.cooldown;
                player_damage_taken += attack.damage;
            }
        }

        if player_damage_taken > 0 && self.damage_entity(player_id, player_damage_taken) {
            if let Some(sound) = hit_sound {
                self.commands().play_sound(sound);
            }
        }
    }

    pub fn run_simple_melee_combat(
        &mut self,
        attack: ActionId,
        hit_sound: Option<SoundHandle>,
        dt: f32,
    ) {
        if self.pressed(attack) {
            self.player_melee_attack_nearest_enemy(hit_sound);
        }
        self.enemies_melee_attack_player(dt, hit_sound);
        self.despawn_dead_enemies();
    }

    pub fn run_melee_combat(&mut self, config: MeleeCombatConfig, dt: f32) {
        if self.pressed(config.attack) {
            self.play_player_attack_animation(
                config.directional_player_attack_animation,
                config.player_attack_animation,
            );
            self.player_melee_attack_nearest_enemy(config.hit_sound);
        }
        self.enemies_melee_attack_player(dt, config.hit_sound);
        if config.despawn_dead_enemies {
            self.despawn_dead_enemies();
        }
    }

    pub(crate) fn play_player_attack_animation(
        &mut self,
        directional: bool,
        fallback: Option<&str>,
    ) {
        let Some(player) = self.player_id() else {
            return;
        };
        if directional {
            let direction = self
                .component::<FacingDirection>(player)
                .copied()
                .unwrap_or_default();
            if self.play_animation(player, direction.attack_clip()) {
                return;
            }
        }
        if let Some(animation) = fallback.or_else(|| directional.then_some("attack")) {
            self.play_animation(player, animation);
        }
    }

    fn player_melee_snapshot(&self) -> Option<(EntityId, Vec2, f32, i32)> {
        self.entities_with::<Player>().into_iter().find_map(|id| {
            let transform = self.component::<Transform>(id)?;
            let attack = self.component::<MeleeAttack>(id)?;
            Some((id, transform.pos, attack.range, attack.damage))
        })
    }
}
