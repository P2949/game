use super::shared::*;

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
