use super::shared::*;

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
