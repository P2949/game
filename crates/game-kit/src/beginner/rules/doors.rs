use super::shared::*;

const DEFAULT_DOOR_TRIGGER_RANGE: f32 = 28.0;

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
