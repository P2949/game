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
pub fn tick_commands(game: &mut GameCtx, attack: ActionId, hit_sound: SoundHandle, dt: f32) {
    let effects = {
        let (world, input) = game.world_and_input();
        tick_effects(world, input, attack, dt)
    };
    let mut commands = game.commands();
    for _ in 0..effects.hit_sounds {
        commands.play_sound(hit_sound);
    }
    for id in effects.despawns {
        commands.despawn(id);
    }
}

fn tick_effects(world: &mut World, input: &Input, attack: ActionId, dt: f32) -> CombatEffects {
    let mut effects = CombatEffects::default();
    let Some((player_id, player_pos, player_range, player_damage)) = player_snapshot(world) else {
        return effects;
    };

    if input.pressed(attack) {
        if let Some(target) = nearest_enemy_in_range(world, player_pos, player_range) {
            if apply_damage(world, target, player_damage) {
                effects.hit_sounds += 1;
            }
        }
    }

    let mut player_damage_taken = 0;
    for id in enemy_ids(world) {
        if world.get::<Health>(id).is_some_and(Health::is_dead) {
            continue;
        }
        let Some(enemy_pos) = world.get::<Transform>(id).map(|transform| transform.pos) else {
            continue;
        };

        let Some(attack) = world.get_mut::<MeleeAttack>(id) else {
            continue;
        };
        attack.timer = (attack.timer - dt).max(0.0);
        if attack.timer == 0.0 && enemy_pos.distance(player_pos) <= attack.range {
            attack.timer = attack.cooldown;
            player_damage_taken += attack.damage;
        }
    }

    if player_damage_taken > 0 && apply_damage(world, player_id, player_damage_taken) {
        effects.hit_sounds += 1;
    }

    effects.despawns = enemy_ids(world)
        .into_iter()
        .filter(|id| world.get::<Health>(*id).is_some_and(Health::is_dead))
        .collect();
    effects
}

pub fn kill_player(world: &mut World) {
    for id in world.ids_with::<PlayerController>() {
        if let Some(health) = world.get_mut::<Health>(id) {
            health.damage(health.current);
        }
        if let Some(velocity) = world.get_mut::<Velocity>(id) {
            velocity.0 = glam::Vec2::ZERO;
        }
    }
}

pub fn player_is_dead(world: &World) -> bool {
    world
        .ids_with::<PlayerController>()
        .into_iter()
        .any(|id| world.get::<Health>(id).is_some_and(Health::is_dead))
}

fn is_enemy(world: &World, id: EntityId) -> bool {
    world
        .get::<Faction>(id)
        .is_some_and(|faction| faction.0 == FactionId::Enemy)
}

fn enemy_ids(world: &World) -> Vec<EntityId> {
    world
        .ids_with::<Faction>()
        .into_iter()
        .filter(|id| is_enemy(world, *id))
        .collect()
}

fn player_snapshot(world: &World) -> Option<(EntityId, glam::Vec2, f32, i32)> {
    world
        .ids_with::<PlayerController>()
        .into_iter()
        .find_map(|id| {
            let transform = world.get::<Transform>(id)?;
            let attack = world.get::<MeleeAttack>(id)?;
            Some((id, transform.pos, attack.range, attack.damage))
        })
}

fn nearest_enemy_in_range(world: &World, player_pos: glam::Vec2, range: f32) -> Option<EntityId> {
    enemy_ids(world)
        .into_iter()
        .filter_map(|id| {
            if world.get::<Health>(id).is_some_and(Health::is_dead) {
                return None;
            }
            let transform = world.get::<Transform>(id)?;
            let dist_sq = transform.pos.distance_squared(player_pos);
            (dist_sq <= range * range).then_some((id, dist_sq))
        })
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(id, _)| id)
}

#[cfg(test)]
mod tests {
    use game_kit::prelude::*;

    use super::{kill_player, player_is_dead, tick_effects};
    use crate::actor::PlayerController;

    const ATTACK: ActionId = ActionId(0);

    fn input(attack: bool) -> Input {
        let input = Input::default();
        if attack {
            input.with_pressed(ATTACK)
        } else {
            input
        }
    }

    fn world_with_player_and_enemy(enemy_pos: glam::Vec2) -> World {
        let mut world = World::new();
        world.spawn(
            Entity::new(glam::Vec2::ZERO)
                .with(PlayerController {
                    move_axis: Axis2dId(0),
                })
                .with(Faction::player())
                .with(Health::new(120))
                .with(MeleeAttack::new(20.0, 50)),
        );
        world.spawn(
            Entity::new(enemy_pos)
                .with(Faction::enemy())
                .with(Health::new(40))
                .with(MeleeAttack::new(5.0, 7).cooldown(0.75)),
        );
        world
    }

    #[test]
    fn player_attack_records_dead_enemy_despawn_effect() {
        let mut world = world_with_player_and_enemy(glam::vec2(10.0, 0.0));
        let effects = tick_effects(&mut world, &input(true), ATTACK, 1.0 / 120.0);

        assert_eq!(effects.hit_sounds, 1);
        assert_eq!(effects.despawns.len(), 1);
    }

    #[test]
    fn enemy_in_range_damages_player() {
        let mut world = world_with_player_and_enemy(glam::vec2(4.0, 0.0));

        let effects = tick_effects(&mut world, &input(false), ATTACK, 1.0 / 120.0);

        let player = world.ids_with::<PlayerController>()[0];
        assert_eq!(effects.hit_sounds, 1);
        assert_eq!(world.get::<Health>(player).unwrap().current, 113);
    }

    #[test]
    fn debug_kill_marks_player_dead() {
        let mut world = world_with_player_and_enemy(glam::vec2(100.0, 0.0));

        kill_player(&mut world);

        assert!(player_is_dead(&world));
    }

    #[test]
    fn no_combat_events_when_idle_and_out_of_range() {
        // Enemy is far outside its attack range and the player does not attack, so
        // no hit sounds or despawns are queued.
        let mut world = world_with_player_and_enemy(glam::vec2(100.0, 0.0));
        let effects = tick_effects(&mut world, &input(false), ATTACK, 1.0 / 120.0);

        assert_eq!(effects.hit_sounds, 0);
        assert!(effects.despawns.is_empty());
    }
}
