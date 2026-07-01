use super::effects;
use super::*;

fn asset_path(asset_base: Option<&Path>, relative: &str) -> Option<String> {
    let base = asset_base?;
    let path = Path::new(relative);
    let full_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    };
    Some(full_path.to_string_lossy().into_owned())
}

fn conventional_asset_path(
    asset_base: Option<&Path>,
    folder: &str,
    key: &str,
    extensions: &[&str],
) -> Option<String> {
    let base = asset_base?;
    let first = extensions.first()?;
    let preferred = base.join(folder).join(format!("{key}.{first}"));
    Some(
        extensions
            .iter()
            .map(|extension| base.join(folder).join(format!("{key}.{extension}")))
            .find(|candidate| candidate.is_file())
            .unwrap_or(preferred)
            .to_string_lossy()
            .into_owned(),
    )
}

pub(super) fn build_beginner_game_file(
    game: &mut GameApp<'_>,
    file: BeginnerGameFile,
    label: &str,
    asset_base: Option<&Path>,
) -> Result<TopDownControls> {
    let runtime_config = BeginnerRuntimeConfig::from_file(&file);
    game.startup(move |game: &mut StartupGameCtx<'_, '_>| {
        game.insert_resource(runtime_config.clone());
        game.insert_resource(BeginnerRuleUiText::default());
        Ok(())
    });

    let mut asset_author = game.asset_bag();
    for key in &file.assets.textures {
        asset_author = match conventional_asset_path(asset_base, "textures", key, &["png"]) {
            Some(path) => asset_author.texture(key.clone(), path)?,
            None => asset_author.texture_auto(key.clone())?,
        };
    }
    for key in &file.assets.sounds {
        asset_author =
            match conventional_asset_path(asset_base, "sounds", key, &["wav", "ogg", "mp3"]) {
                Some(path) => asset_author.sound(key.clone(), path)?,
                None => asset_author.sound_auto(key.clone())?,
            };
    }
    for key in &file.assets.music {
        asset_author =
            match conventional_asset_path(asset_base, "music", key, &["wav", "ogg", "mp3"]) {
                Some(path) => asset_author.music(key.clone(), path)?,
                None => asset_author.music_auto(key.clone())?,
            };
    }
    for key in &file.assets.animation_sheets {
        asset_author = match asset_path(asset_base, &format!("animations/{key}.ron")) {
            Some(path) => asset_author.spritesheet_from_meta(key.clone(), path)?,
            None => asset_author.animation_sheet_auto(key.clone())?,
        };
    }
    let assets = asset_author.build();

    let controls = match file.controls.kind(label)? {
        BeginnerControlsKind::TopDown => game.input(|input| input.top_down_controls())?,
    };

    for prefab in file.prefabs {
        build_prefab(game, &assets, prefab, controls)?;
    }

    for map in file.maps {
        build_map(game, map, asset_base);
    }

    if let Some(scene_flow) = file.scene_flow {
        build_scene_flow(game, scene_flow, controls);
    }

    effects::build_audio(game, file.audio);
    build_rule_ui_text(game);
    effects::build_actions(game, file.actions, controls);
    effects::build_custom_rules(game, file.custom_rules);

    let mut rules = game.rules();
    for rule in &file.rules {
        if let Some(kind) = rule.simple_kind(label)? {
            rules = apply_rule(rules, kind, controls);
        }
    }
    rules.build();
    for rule in file.rules {
        if let BeginnerRuleFile::Script(rule) = rule {
            effects::build_script_rule(game, rule, controls);
        }
    }
    Ok(controls)
}

fn build_prefab(
    game: &mut GameApp<'_>,
    assets: &crate::assets::AssetBag,
    prefab: BeginnerPrefabFile,
    controls: TopDownControls,
) -> Result<()> {
    match prefab {
        BeginnerPrefabFile::Player(player) => {
            let mut author = game
                .player_prefab(player.name)
                .sprite(player.sprite)
                .moves_with(controls.movement, player.speed)
                .health(player.health);
            if let Some(sheet) = player.animation_sheet {
                author = author.animation_sheet(assets.animation_sheet_result(&sheet)?);
            }
            if let Some(melee) = player.melee {
                author = author.melee(melee.range, melee.damage);
            }
            for tag in player.tags {
                author = author.tag(tag);
            }
            for (key, value) in player.data {
                author = author.data(key, value);
            }
            author.build()?;
        }
        BeginnerPrefabFile::Enemy(enemy) => {
            let mut author = game
                .enemy_prefab(enemy.name)
                .sprite(enemy.sprite)
                .speed(enemy.speed)
                .health(enemy.health);
            if let Some(sheet) = enemy.animation_sheet {
                author = author.animation_sheet(assets.animation_sheet_result(&sheet)?);
            }
            if enemy.chase_player {
                author = author.chases_player();
            }
            if let Some(melee) = enemy.melee {
                author = author.melee(melee.range, melee.damage);
            }
            if let Some(drop) = enemy.drops {
                author = author.drops(drop);
            }
            if let Some(chance) = enemy.drop_chance {
                author = author.drop_chance(chance);
            }
            if enemy.despawn_after_death_animation {
                author = author.despawn_after_death_animation();
            }
            for tag in enemy.tags {
                author = author.tag(tag);
            }
            for (key, value) in enemy.data {
                author = author.data(key, value);
            }
            author.build()?;
        }
        BeginnerPrefabFile::Pickup(pickup) => {
            let mut author = game
                .pickup_prefab(pickup.name)
                .sprite(pickup.sprite)
                .score(pickup.score);
            if let Some(heal) = pickup.heal_player {
                author = author.heal_player(heal);
            }
            if let Some(sound) = pickup.sound {
                author = author.play_sound(sound);
            }
            if pickup.despawn_on_collect {
                author = author.despawn_on_collect();
            }
            for tag in pickup.tags {
                author = author.tag(tag);
            }
            for (key, value) in pickup.data {
                author = author.data(key, value);
            }
            author.build()?;
        }
        BeginnerPrefabFile::Door(door) => {
            let mut author = game.door_prefab(door.name).sprite(door.sprite);
            author = match door.action {
                DoorActionFile::ChangeMap(map) => author.change_map(map),
                DoorActionFile::ChangeScene(scene) => author.change_scene(scene),
                DoorActionFile::RestartLevel => author.restart_level(),
            };
            if door.requires_all_enemies_dead {
                author = author.requires_all_enemies_dead();
            }
            for tag in door.tags {
                author = author.tag(tag);
            }
            for (key, value) in door.data {
                author = author.data(key, value);
            }
            author.build()?;
        }
        BeginnerPrefabFile::Projectile(projectile) => {
            let mut author = game
                .projectile_prefab(projectile.name)
                .sprite(projectile.sprite)
                .damage(projectile.damage)
                .speed(projectile.speed)
                .lifetime(projectile.lifetime);
            if let Some(sheet) = projectile.animation_sheet {
                author = author.animation_sheet(assets.animation_sheet_result(&sheet)?);
            }
            if projectile.despawn_on_hit {
                author = author.despawn_on_hit();
            }
            for tag in projectile.tags {
                author = author.tag(tag);
            }
            for (key, value) in projectile.data {
                author = author.data(key, value);
            }
            author.build()?;
        }
        BeginnerPrefabFile::Spawner(spawner) => {
            let mut author = game
                .spawner_prefab(spawner.name)
                .spawn(spawner.spawn)
                .every_seconds(spawner.every_seconds);
            if let Some(max_alive) = spawner.max_alive {
                author = author.max_alive(max_alive);
            }
            author = match spawner.placement {
                SpawnPlacementFile::AtSpawner => author.at_spawner(),
                SpawnPlacementFile::NearPlayer(radius) => author.near_player(radius),
                SpawnPlacementFile::AtFirstFloor => author.at_first_floor(),
            };
            author.build()?;
        }
        BeginnerPrefabFile::Trigger(trigger) => {
            let mut author = game
                .trigger_prefab(trigger.name)
                .size(vec2(trigger.size.0, trigger.size.1));
            if let Some(texture) = trigger.visible_debug {
                author = author.visible_debug(texture);
            }
            for tag in trigger.tags {
                author = author.tag(tag);
            }
            for (key, value) in trigger.data {
                author = author.data(key, value);
            }
            author.build()?;
        }
        BeginnerPrefabFile::Checkpoint(checkpoint) => {
            let mut author = game
                .checkpoint_prefab(checkpoint.name)
                .sprite(checkpoint.sprite)
                .size(vec2(checkpoint.size.0, checkpoint.size.1));
            for tag in checkpoint.tags {
                author = author.tag(tag);
            }
            for (key, value) in checkpoint.data {
                author = author.data(key, value);
            }
            author.build()?;
        }
    }
    Ok(())
}

fn build_map(game: &mut GameApp<'_>, map: BeginnerMapFile, asset_base: Option<&Path>) {
    match map {
        BeginnerMapFile::TextMap(map) => {
            let path = asset_path(asset_base, &map.path).unwrap_or(map.path);
            let mut author = game
                .map_from_text(map.name.as_str(), path)
                .tile_size(map.tile_size)
                .simple_theme(map.theme.0.as_str(), map.theme.1.as_str());
            for (symbol, prefab) in map.legend {
                author = author.legend(symbol, prefab);
            }
            if map.start {
                author.start();
            } else {
                author.finish();
            }
        }
        BeginnerMapFile::TextMapAuto(map) => {
            let path = asset_path(asset_base, &format!("maps/{}.txt", map.name));
            let mut author = game
                .map_from_text(
                    map.name.as_str(),
                    path.unwrap_or_else(|| format!("maps/{}.txt", map.name)),
                )
                .tile_size(map.tile_size)
                .simple_theme(map.theme.0.as_str(), map.theme.1.as_str());
            for (symbol, prefab) in map.legend {
                author = author.legend(symbol, prefab);
            }
            if map.start {
                author.start();
            } else {
                author.finish();
            }
        }
        BeginnerMapFile::Tiled(map) => {
            let path = asset_path(asset_base, &map.path).unwrap_or(map.path);
            let mut author = game
                .map_from_tiled(map.name.as_str(), path)
                .simple_theme(map.theme.0.as_str(), map.theme.1.as_str());
            for (object, prefab) in map.objects {
                author = author.object(object, prefab);
            }
            if map.start {
                author.start();
            } else {
                author.finish();
            }
        }
        BeginnerMapFile::Ldtk(map) => {
            let path = asset_path(asset_base, &map.path).unwrap_or(map.path);
            let mut author = game
                .map_from_ldtk(map.name.as_str(), path)
                .level(map.level)
                .simple_theme(map.theme.0.as_str(), map.theme.1.as_str());
            for (entity, prefab) in map.entities {
                author = author.entity(entity, prefab);
            }
            if map.start {
                author.start();
            } else {
                author.finish();
            }
        }
    }
}

fn build_scene_flow(game: &mut GameApp<'_>, flow: SceneFlowFile, controls: TopDownControls) {
    let mut author = game.use_simple_scene_flow();
    if let Some(menu) = flow.menu {
        author = author.menu(menu);
    }
    if let Some(game_scene) = flow.game {
        author = author.game(game_scene);
    }
    if let Some(game_over) = flow.game_over {
        author = author.game_over(game_over);
    }
    if let Some(win) = flow.win {
        author = author.win(win);
    }
    if let Some(text) = flow.menu_text {
        author = author.menu_text(text);
    }
    if let Some(button) = flow.menu_button {
        author = author.menu_button(button.label, button.map);
    }
    if let Some(text) = flow.game_over_text {
        author = author.game_over_text(text);
    }
    if let Some(button) = flow.game_over_button {
        author = author.game_over_button(button);
    }
    if let Some(text) = flow.win_text {
        author = author.win_text(text);
    }
    if let Some(button) = flow.win_button {
        author = author.win_button(button);
    }
    if let Some(action) = flow.start_on {
        author = author.start_on(action.resolve(controls));
    }
    if let Some(action) = flow.restart_on {
        author = author.restart_on(action.resolve(controls));
    }
    match flow.win_condition {
        Some(WinConditionFile::AllPickupsCollected) => {
            author = author.win_when_all_pickups_collected();
        }
        Some(WinConditionFile::AllEnemiesDead) => {
            author = author.win_when_all_enemies_dead();
        }
        None => {}
    }
    author.build();
}

fn build_rule_ui_text(game: &mut GameApp<'_>) {
    game.ui(|game: &mut GameCtx<'_, '_>, _dt| {
        let lines = game
            .resource::<BeginnerRuleUiText>()
            .map(|text| text.lines.clone())
            .unwrap_or_default();
        let Some((first, rest)) = lines.split_first() else {
            return;
        };
        let mut panel = game.ui().panel(first);
        for line in rest {
            panel = panel.line(line);
        }
        panel.center();
    });
}

fn apply_rule<'a, 'app>(
    rules: RulesAuthor<'a, 'app>,
    rule: BeginnerRuleKind,
    controls: TopDownControls,
) -> RulesAuthor<'a, 'app> {
    match rule {
        BeginnerRuleKind::TopDownControls => rules.top_down_controls(controls),
        BeginnerRuleKind::PlayerCollectsPickups => rules.player_collects_pickups(),
        BeginnerRuleKind::EnemiesDamagePlayer => rules.enemies_damage_player(),
        BeginnerRuleKind::DeadEnemiesDespawn => rules.dead_enemies_despawn(),
        BeginnerRuleKind::EnemyDrops => rules.enemy_drops(),
        BeginnerRuleKind::Projectiles => rules.projectiles(),
        BeginnerRuleKind::ProjectilesMove => rules.projectiles_move(),
        BeginnerRuleKind::ProjectilesExpireAfterLifetime => {
            rules.projectiles_expire_after_lifetime()
        }
        BeginnerRuleKind::ProjectilesDamageEnemies => rules.projectiles_damage_enemies(),
        BeginnerRuleKind::ProjectilesDespawnOnHit => rules.projectiles_despawn_on_hit(),
        BeginnerRuleKind::ProjectileImpactAnimationBeforeDespawn => {
            rules.projectile_impact_animation_before_despawn()
        }
        BeginnerRuleKind::SpawnersSpawnPrefabs => rules.spawners_spawn_prefabs(),
        BeginnerRuleKind::DoorsChangeMaps => rules.doors_change_maps(),
        BeginnerRuleKind::PlayerActivatesCheckpoints => rules.player_activates_checkpoints(),
        BeginnerRuleKind::RespawnAtCheckpoint => rules.respawn_at_checkpoint(),
        BeginnerRuleKind::CameraFollowsPlayer => rules.camera_follows_player(),
        BeginnerRuleKind::PauseAndReset => rules.pause_and_reset(),
        BeginnerRuleKind::ShowBasicUi => rules.show_basic_ui(),
        BeginnerRuleKind::ShowScore => rules.show_score(),
        BeginnerRuleKind::ShowEnemyCount => rules.show_enemy_count(),
        BeginnerRuleKind::ShowPlayerHealth => rules.show_player_health(),
        BeginnerRuleKind::ShowMenu => rules.show_menu(),
        BeginnerRuleKind::ShowPauseMenu => rules.show_pause_menu(),
        BeginnerRuleKind::ShowGameOverPanel => rules.show_game_over_panel(),
        BeginnerRuleKind::ShowWinPanel => rules.show_win_panel(),
        BeginnerRuleKind::WinWhenAllPickupsCollected => rules.win_when_all_pickups_collected(),
        BeginnerRuleKind::WinWhenAllEnemiesDead => rules.win_when_all_enemies_dead(),
        BeginnerRuleKind::AnimateEnemiesByMovement => rules.animate_enemies_by_movement(),
        BeginnerRuleKind::AnimatePlayerDirectionally => rules.animate_player_directionally(),
        BeginnerRuleKind::AnimateEnemiesDirectionally => rules.animate_enemies_directionally(),
        BeginnerRuleKind::AnimateAttacksDirectionally => rules.animate_attacks_directionally(),
        BeginnerRuleKind::DeadEnemiesPlayDeathAnimation => {
            rules.dead_enemies_play_death_animation()
        }
        BeginnerRuleKind::DeadEnemiesDespawnAfterAnimation => {
            rules.dead_enemies_despawn_after_animation()
        }
    }
}
