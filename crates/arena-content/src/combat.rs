use crate::actor::{EnemyTag, PlayerController};
use game_combat::{Health, MeleeAttack, apply_damage};
use game_core::audio::Audio;
use game_core::backend::SoundHandle;
use game_core::commands::CommandQueue;
use game_core::input::{ActionId, Input};
use game_core::world::World;
use game_core::world::{EntityId, Transform, Velocity};

#[derive(Default)]
struct CombatEffects {
    hit_sounds: u32,
    despawns: Vec<EntityId>,
}

pub fn tick(
    world: &mut World,
    input: &Input,
    attack: ActionId,
    audio: &mut Audio<'_>,
    hit_sound: SoundHandle,
    dt: f32,
) {
    let effects = tick_effects(world, input, attack, dt);
    for _ in 0..effects.hit_sounds {
        audio.play(hit_sound, 0.8);
    }
    for id in effects.despawns {
        world.despawn(id);
    }
}

pub fn tick_commands(
    world: &mut World,
    input: &Input,
    attack: ActionId,
    hit_sound: SoundHandle,
    dt: f32,
) {
    let effects = tick_effects(world, input, attack, dt);
    let queue = world.resource_or_insert_with(CommandQueue::new);
    for _ in 0..effects.hit_sounds {
        queue.play_sound(hit_sound);
    }
    for id in effects.despawns {
        queue.despawn(id);
    }
}

fn tick_effects(world: &mut World, input: &Input, attack: ActionId, dt: f32) -> CombatEffects {
    let mut effects = CombatEffects::default();
    let Some((player_id, player_pos, player_range, player_damage)) = player_snapshot(world) else {
        return effects;
    };

    if input.pressed(attack) {
        if let Some(target) = nearest_enemy_in_range(world, player_pos, player_range) {
            if damage_entity(world, target, player_damage) {
                effects.hit_sounds += 1;
            }
        }
    }

    let mut player_damage_taken = 0;
    for id in world.ids_with::<EnemyTag>() {
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

    if player_damage_taken > 0 && damage_entity(world, player_id, player_damage_taken) {
        effects.hit_sounds += 1;
    }

    effects.despawns = world
        .ids_with::<EnemyTag>()
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
    world
        .ids_with::<EnemyTag>()
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

fn damage_entity(world: &mut World, id: EntityId, amount: i32) -> bool {
    apply_damage(world, id, amount)
}

#[cfg(test)]
mod tests {
    use crate::actor::{EnemyTag, PlayerController};
    use game_combat::{Health, MeleeAttack};
    use game_core::audio::{Audio, AudioCommands};
    use game_core::backend::SoundHandle;
    use game_core::input::{ActionId, Axis2dId, Input};
    use game_core::world::Entity;
    use game_core::world::World;

    use super::{kill_player, player_is_dead, tick};

    const ATTACK: ActionId = ActionId(0);

    fn input(attack_pressed: bool) -> Input {
        let input = Input::default();
        if attack_pressed {
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
                .with(Health::new(100))
                .with(MeleeAttack::new(20.0, 50)),
        );
        world.spawn(
            Entity::new(enemy_pos)
                .with(EnemyTag)
                .with(Health::new(40))
                .with(MeleeAttack::new(5.0, 7).cooldown(0.75)),
        );
        world
    }

    #[test]
    fn player_attack_damages_and_despawns_dead_enemy() {
        let mut world = world_with_player_and_enemy(glam::vec2(10.0, 0.0));
        let mut commands = AudioCommands::default();
        let mut audio = Audio::new(&mut commands);

        tick(
            &mut world,
            &input(true),
            ATTACK,
            &mut audio,
            SoundHandle(0),
            1.0 / 120.0,
        );

        assert!(world.ids_with::<EnemyTag>().is_empty());
    }

    #[test]
    fn enemy_attack_damages_player() {
        let mut world = world_with_player_and_enemy(glam::vec2(4.0, 0.0));
        let mut commands = AudioCommands::default();
        let mut audio = Audio::new(&mut commands);

        tick(
            &mut world,
            &input(false),
            ATTACK,
            &mut audio,
            SoundHandle(0),
            1.0 / 120.0,
        );

        let player = world.ids_with::<PlayerController>()[0];
        assert_eq!(world.get::<Health>(player).unwrap().current, 93);
    }

    #[test]
    fn kill_player_marks_player_dead() {
        let mut world = world_with_player_and_enemy(glam::vec2(100.0, 0.0));

        kill_player(&mut world);

        assert!(player_is_dead(&world));
    }
}
