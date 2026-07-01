use super::shared::*;

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

/// Moves all projectile prefabs according to their velocity.
#[derive(Clone)]
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
