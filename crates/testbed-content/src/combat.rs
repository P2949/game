use crate::actor::PlayerController;
use game_kit::prelude::*;

#[derive(Default)]
struct CombatEffects {
    hit_sounds: u32,
    despawns: Vec<EntityId>,
}

/// Resolves a melee combat tick into engine commands: queued hit sounds and
/// despawns of dead enemies. Enemies are identified by [`Faction`] (Enemy) rather
/// than a content-specific tag, demonstrating reuse of `game-combat`.
pub fn tick_commands(
    game: &mut GameCtx<'_, '_>,
    attack: ActionId,
    hit_sound: SoundHandle,
    dt: f32,
) {
    let effects = tick_effects(game, attack, dt);
    let mut commands = game.commands();
    for _ in 0..effects.hit_sounds {
        commands.play_sound(hit_sound);
    }
    for id in effects.despawns {
        commands.despawn(id);
    }
}

fn tick_effects(game: &mut GameCtx<'_, '_>, attack: ActionId, dt: f32) -> CombatEffects {
    let mut effects = CombatEffects::default();
    let Some((player_id, player_pos, player_range, player_damage)) = player_snapshot(game) else {
        return effects;
    };

    if game.pressed(attack) {
        if let Some(target) = nearest_enemy_in_range(game, player_pos, player_range) {
            if game.damage(target, player_damage) {
                effects.hit_sounds += 1;
            }
        }
    }

    let mut player_damage_taken = 0;
    for id in enemy_ids(game) {
        if game.is_dead(id) {
            continue;
        }
        let Some(enemy_pos) = game.position(id) else {
            continue;
        };

        let Some(attack) = game.melee_attack_mut(id) else {
            continue;
        };
        attack.timer = (attack.timer - dt).max(0.0);
        if attack.timer == 0.0 && enemy_pos.distance(player_pos) <= attack.range {
            attack.timer = attack.cooldown;
            player_damage_taken += attack.damage;
        }
    }

    if player_damage_taken > 0 && game.damage(player_id, player_damage_taken) {
        effects.hit_sounds += 1;
    }

    effects.despawns = enemy_ids(game)
        .into_iter()
        .filter(|id| game.is_dead(*id))
        .collect();
    effects
}

pub fn kill_player(game: &mut GameCtx<'_, '_>) {
    for id in game.entities_with::<PlayerController>() {
        if let Some(health) = game.component_mut::<Health>(id) {
            health.damage(health.current);
        }
        if let Some(velocity) = game.component_mut::<Velocity>(id) {
            velocity.0 = Vec2::ZERO;
        }
    }
}

pub fn player_is_dead(game: &GameCtx<'_, '_>) -> bool {
    game.entities_with::<PlayerController>()
        .into_iter()
        .any(|id| game.is_dead(id))
}

fn is_enemy(game: &GameCtx<'_, '_>, id: EntityId) -> bool {
    game.faction(id) == Some(FactionId::Enemy)
}

fn enemy_ids(game: &GameCtx<'_, '_>) -> Vec<EntityId> {
    game.entities_where::<Faction>(|_, faction| faction.0 == FactionId::Enemy)
}

fn player_snapshot(game: &GameCtx<'_, '_>) -> Option<(EntityId, Vec2, f32, i32)> {
    game.entities_with::<PlayerController>()
        .into_iter()
        .find_map(|id| {
            let transform = game.component::<Transform>(id)?;
            let attack = game.component::<MeleeAttack>(id)?;
            Some((id, transform.pos, attack.range, attack.damage))
        })
}

fn nearest_enemy_in_range(
    game: &GameCtx<'_, '_>,
    player_pos: Vec2,
    range: f32,
) -> Option<EntityId> {
    game.nearest_by_position::<Faction>(player_pos, range, |id| {
        is_enemy(game, id) && !game.is_dead(id)
    })
}
