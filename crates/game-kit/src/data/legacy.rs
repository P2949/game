use super::*;

pub(super) fn legacy_rule_kind(name: &str, label: &str) -> Result<BeginnerRuleKind> {
    let kind = match name {
        "top_down_controls" | "TopDownControls" => BeginnerRuleKind::TopDownControls,
        "player_collects_pickups" | "PlayerCollectsPickups" => {
            BeginnerRuleKind::PlayerCollectsPickups
        }
        "enemies_damage_player" | "EnemiesDamagePlayer" => BeginnerRuleKind::EnemiesDamagePlayer,
        "dead_enemies_despawn" | "DeadEnemiesDespawn" => BeginnerRuleKind::DeadEnemiesDespawn,
        "enemy_drops" | "EnemyDrops" => BeginnerRuleKind::EnemyDrops,
        "projectiles" | "Projectiles" => BeginnerRuleKind::Projectiles,
        "projectiles_move" | "ProjectilesMove" => BeginnerRuleKind::ProjectilesMove,
        "projectiles_expire_after_lifetime" | "ProjectilesExpireAfterLifetime" => {
            BeginnerRuleKind::ProjectilesExpireAfterLifetime
        }
        "projectiles_damage_enemies" | "ProjectilesDamageEnemies" => {
            BeginnerRuleKind::ProjectilesDamageEnemies
        }
        "projectiles_despawn_on_hit" | "ProjectilesDespawnOnHit" => {
            BeginnerRuleKind::ProjectilesDespawnOnHit
        }
        "projectile_impact_animation_before_despawn" | "ProjectileImpactAnimationBeforeDespawn" => {
            BeginnerRuleKind::ProjectileImpactAnimationBeforeDespawn
        }
        "spawners_spawn_prefabs" | "SpawnersSpawnPrefabs" => BeginnerRuleKind::SpawnersSpawnPrefabs,
        "doors_change_maps" | "DoorsChangeMaps" => BeginnerRuleKind::DoorsChangeMaps,
        "player_activates_checkpoints" | "PlayerActivatesCheckpoints" => {
            BeginnerRuleKind::PlayerActivatesCheckpoints
        }
        "respawn_at_checkpoint" | "RespawnAtCheckpoint" => BeginnerRuleKind::RespawnAtCheckpoint,
        "camera_follows_player" | "CameraFollowsPlayer" => BeginnerRuleKind::CameraFollowsPlayer,
        "pause_and_reset" | "PauseAndReset" => BeginnerRuleKind::PauseAndReset,
        "show_basic_ui" | "ShowBasicUi" => BeginnerRuleKind::ShowBasicUi,
        "show_score" | "ShowScore" => BeginnerRuleKind::ShowScore,
        "show_enemy_count" | "ShowEnemyCount" => BeginnerRuleKind::ShowEnemyCount,
        "show_player_health" | "ShowPlayerHealth" => BeginnerRuleKind::ShowPlayerHealth,
        "show_menu" | "ShowMenu" => BeginnerRuleKind::ShowMenu,
        "show_pause_menu" | "ShowPauseMenu" => BeginnerRuleKind::ShowPauseMenu,
        "show_game_over_panel" | "ShowGameOverPanel" => BeginnerRuleKind::ShowGameOverPanel,
        "show_win_panel" | "ShowWinPanel" => BeginnerRuleKind::ShowWinPanel,
        "win_when_all_pickups_collected" | "WinWhenAllPickupsCollected" => {
            BeginnerRuleKind::WinWhenAllPickupsCollected
        }
        "win_when_all_enemies_dead" | "WinWhenAllEnemiesDead" => {
            BeginnerRuleKind::WinWhenAllEnemiesDead
        }
        "animate_enemies_by_movement" | "AnimateEnemiesByMovement" => {
            BeginnerRuleKind::AnimateEnemiesByMovement
        }
        "animate_player_directionally" | "AnimatePlayerDirectionally" => {
            BeginnerRuleKind::AnimatePlayerDirectionally
        }
        "animate_enemies_directionally" | "AnimateEnemiesDirectionally" => {
            BeginnerRuleKind::AnimateEnemiesDirectionally
        }
        "animate_attacks_directionally" | "AnimateAttacksDirectionally" => {
            BeginnerRuleKind::AnimateAttacksDirectionally
        }
        "dead_enemies_play_death_animation" | "DeadEnemiesPlayDeathAnimation" => {
            BeginnerRuleKind::DeadEnemiesPlayDeathAnimation
        }
        "dead_enemies_despawn_after_animation" | "DeadEnemiesDespawnAfterAnimation" => {
            BeginnerRuleKind::DeadEnemiesDespawnAfterAnimation
        }
        other => {
            let suggestion = closest_name(other, LEGACY_RULES.iter().copied())
                .map(|candidate| format!(" Did you mean '{candidate}'?"))
                .unwrap_or_default();
            anyhow::bail!(
                "beginner game file '{label}' has unknown rule '{other}'. Supported legacy rules: {}.{suggestion}",
                LEGACY_RULES.join(", ")
            );
        }
    };
    Ok(kind)
}

const LEGACY_RULES: &[&str] = &[
    "top_down_controls",
    "player_collects_pickups",
    "enemies_damage_player",
    "dead_enemies_despawn",
    "enemy_drops",
    "projectiles",
    "projectiles_move",
    "projectiles_expire_after_lifetime",
    "projectiles_damage_enemies",
    "projectiles_despawn_on_hit",
    "projectile_impact_animation_before_despawn",
    "spawners_spawn_prefabs",
    "doors_change_maps",
    "player_activates_checkpoints",
    "respawn_at_checkpoint",
    "camera_follows_player",
    "pause_and_reset",
    "show_basic_ui",
    "show_score",
    "show_enemy_count",
    "show_player_health",
    "show_menu",
    "show_pause_menu",
    "show_game_over_panel",
    "show_win_panel",
    "win_when_all_pickups_collected",
    "win_when_all_enemies_dead",
    "animate_enemies_by_movement",
    "animate_player_directionally",
    "animate_enemies_directionally",
    "animate_attacks_directionally",
    "dead_enemies_play_death_animation",
    "dead_enemies_despawn_after_animation",
];
