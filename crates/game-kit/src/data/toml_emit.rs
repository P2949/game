use std::collections::BTreeMap;
use std::fmt::Write;

use super::*;

pub(super) fn emit_authoring_game_toml(
    file: &AuthoringGameFile,
    notes: &mut Vec<String>,
) -> String {
    let mut out = String::new();
    out.push_str("version = 2\n");

    emit_assets(&mut out, &file.assets);
    emit_controls(&mut out, &file.controls, notes);

    for prefab in &file.prefabs {
        emit_prefab(&mut out, prefab, notes);
    }
    for map in &file.maps {
        emit_map(&mut out, map, notes);
    }
    if let Some(scene_flow) = &file.scene_flow {
        emit_scene_flow(&mut out, scene_flow, notes);
    }
    emit_audio(&mut out, &file.audio);
    for action in &file.actions {
        emit_action(&mut out, action);
    }
    for rule in &file.custom_rules {
        emit_custom_rule(&mut out, rule);
    }
    emit_rules(&mut out, &file.rules);

    out
}

fn emit_assets(out: &mut String, assets: &BeginnerAssetsFile) {
    out.push_str("\n[assets]\n");
    emit_string_array(out, "textures", &assets.textures);
    emit_string_array(out, "sounds", &assets.sounds);
    emit_string_array(out, "music", &assets.music);
    emit_string_array(out, "animation_sheets", &assets.animation_sheets);
}

fn emit_controls(out: &mut String, controls: &BeginnerControlsFile, notes: &mut Vec<String>) {
    let preset = match controls {
        BeginnerControlsFile::Structured(BeginnerControlsKind::TopDown) => "top-down",
        BeginnerControlsFile::Legacy(name) if name == "top_down" || name == "TopDown" => {
            notes.push(
                "Converted legacy controls spelling to controls.preset = \"top-down\".".to_owned(),
            );
            "top-down"
        }
        BeginnerControlsFile::Legacy(name) => {
            notes.push(format!(
                "Unsupported legacy controls {name:?}; emitted top-down so validation can report any follow-up issues."
            ));
            "top-down"
        }
    };
    out.push_str("\n[controls]\n");
    emit_str(out, "preset", preset);
}

fn emit_prefab(out: &mut String, prefab: &BeginnerPrefabFile, notes: &mut Vec<String>) {
    out.push_str("\n[[prefab]]\n");
    match prefab {
        BeginnerPrefabFile::Player(prefab) => {
            emit_str(out, "kind", "player");
            emit_actor_fields(
                out,
                ActorFields {
                    name: &prefab.name,
                    sprite: &prefab.sprite,
                    animation_sheet: &prefab.animation_sheet,
                    speed: prefab.speed,
                    health: prefab.health,
                    tags: &prefab.tags,
                    data: &prefab.data,
                },
            );
            if let Some(melee) = &prefab.melee {
                emit_melee(out, melee);
            }
        }
        BeginnerPrefabFile::Enemy(prefab) => {
            emit_str(out, "kind", "enemy");
            emit_actor_fields(
                out,
                ActorFields {
                    name: &prefab.name,
                    sprite: &prefab.sprite,
                    animation_sheet: &prefab.animation_sheet,
                    speed: prefab.speed,
                    health: prefab.health,
                    tags: &prefab.tags,
                    data: &prefab.data,
                },
            );
            emit_bool(out, "chase_player", prefab.chase_player);
            if let Some(drops) = &prefab.drops {
                emit_str(out, "drops", drops);
            }
            if let Some(drop_chance) = prefab.drop_chance {
                emit_f32(out, "drop_chance", drop_chance);
            }
            emit_bool(
                out,
                "despawn_after_death_animation",
                prefab.despawn_after_death_animation,
            );
            if let Some(melee) = &prefab.melee {
                emit_melee(out, melee);
            }
        }
        BeginnerPrefabFile::Pickup(prefab) => {
            emit_str(out, "kind", "pickup");
            emit_str(out, "name", &prefab.name);
            emit_str(out, "sprite", &prefab.sprite);
            emit_i32(out, "score", prefab.score);
            if let Some(heal_player) = prefab.heal_player {
                emit_i32(out, "heal_player", heal_player);
            }
            if let Some(sound) = &prefab.sound {
                emit_str(out, "sound", sound);
            }
            emit_bool(out, "despawn_on_collect", prefab.despawn_on_collect);
            emit_string_array_if_any(out, "tags", &prefab.tags);
            emit_data_if_any(out, &prefab.data);
        }
        BeginnerPrefabFile::Door(prefab) => {
            emit_str(out, "kind", "door");
            emit_str(out, "name", &prefab.name);
            emit_str(out, "sprite", &prefab.sprite);
            match &prefab.action {
                DoorActionFile::ChangeMap(map) => {
                    emit_str(out, "action", "change-map");
                    emit_str(out, "map", map);
                }
                DoorActionFile::ChangeScene(scene) => {
                    emit_str(out, "action", "change-scene");
                    emit_str(out, "scene", scene);
                }
                DoorActionFile::RestartLevel => emit_str(out, "action", "restart-level"),
            }
            emit_bool(
                out,
                "requires_all_enemies_dead",
                prefab.requires_all_enemies_dead,
            );
            emit_string_array_if_any(out, "tags", &prefab.tags);
            emit_data_if_any(out, &prefab.data);
        }
        BeginnerPrefabFile::Projectile(prefab) => {
            emit_str(out, "kind", "projectile");
            emit_str(out, "name", &prefab.name);
            emit_str(out, "sprite", &prefab.sprite);
            if let Some(animation_sheet) = &prefab.animation_sheet {
                emit_str(out, "animation_sheet", animation_sheet);
            }
            emit_i32(out, "damage", prefab.damage);
            emit_f32(out, "speed", prefab.speed);
            emit_f32(out, "duration", prefab.lifetime);
            notes.push("Converted legacy projectile lifetime fields to duration.".to_owned());
            emit_bool(out, "despawn_on_hit", prefab.despawn_on_hit);
            emit_string_array_if_any(out, "tags", &prefab.tags);
            emit_data_if_any(out, &prefab.data);
        }
        BeginnerPrefabFile::Spawner(prefab) => {
            emit_str(out, "kind", "spawner");
            emit_str(out, "name", &prefab.name);
            emit_str(out, "spawn", &prefab.spawn);
            emit_f32(out, "every_seconds", prefab.every_seconds);
            if let Some(max_alive) = prefab.max_alive {
                emit_usize(out, "max_alive", max_alive);
            }
            match prefab.placement {
                SpawnPlacementFile::AtSpawner => emit_str(out, "placement", "at-spawner"),
                SpawnPlacementFile::NearPlayer(radius) => {
                    emit_str(out, "placement", "near-player");
                    emit_f32(out, "placement_radius", radius);
                }
                SpawnPlacementFile::AtFirstFloor => emit_str(out, "placement", "at-first-floor"),
            }
        }
        BeginnerPrefabFile::Trigger(prefab) => {
            emit_str(out, "kind", "trigger");
            emit_str(out, "name", &prefab.name);
            emit_vec2(out, "size", prefab.size);
            if let Some(visible_debug) = &prefab.visible_debug {
                emit_str(out, "visible_debug", visible_debug);
            }
            emit_string_array_if_any(out, "tags", &prefab.tags);
            emit_data_if_any(out, &prefab.data);
        }
        BeginnerPrefabFile::Checkpoint(prefab) => {
            emit_str(out, "kind", "checkpoint");
            emit_str(out, "name", &prefab.name);
            emit_str(out, "sprite", &prefab.sprite);
            emit_vec2(out, "size", prefab.size);
            emit_string_array_if_any(out, "tags", &prefab.tags);
            emit_data_if_any(out, &prefab.data);
        }
    }
}

struct ActorFields<'a> {
    name: &'a str,
    sprite: &'a str,
    animation_sheet: &'a Option<String>,
    speed: f32,
    health: i32,
    tags: &'a [String],
    data: &'a BTreeMap<String, f32>,
}

fn emit_actor_fields(out: &mut String, fields: ActorFields<'_>) {
    emit_str(out, "name", fields.name);
    emit_str(out, "sprite", fields.sprite);
    if let Some(animation_sheet) = fields.animation_sheet {
        emit_str(out, "animation_sheet", animation_sheet);
    }
    emit_f32(out, "speed", fields.speed);
    emit_i32(out, "health", fields.health);
    emit_string_array_if_any(out, "tags", fields.tags);
    emit_data_if_any(out, fields.data);
}

fn emit_melee(out: &mut String, melee: &MeleeFile) {
    out.push_str("\n[prefab.melee]\n");
    emit_f32(out, "range", melee.range);
    emit_i32(out, "damage", melee.damage);
}

fn emit_map(out: &mut String, map: &BeginnerMapFile, notes: &mut Vec<String>) {
    out.push_str("\n[[map]]\n");
    match map {
        BeginnerMapFile::TextMap(map) => {
            emit_str(out, "kind", "text");
            emit_text_map_fields(
                out,
                &map.name,
                &map.path,
                &map.theme,
                map.tile_size,
                &map.legend,
                map.start,
            );
        }
        BeginnerMapFile::TextMapAuto(map) => {
            emit_str(out, "kind", "text");
            let path = format!("maps/{}.txt", map.name);
            emit_text_map_fields(
                out,
                &map.name,
                &path,
                &map.theme,
                map.tile_size,
                &map.legend,
                map.start,
            );
            notes.push(format!(
                "Converted TextMapAuto({:?}) to an explicit text map file path.",
                map.name
            ));
        }
        BeginnerMapFile::Tiled(map) => {
            emit_str(out, "kind", "tiled");
            emit_str(out, "name", &map.name);
            emit_asset_file(out, "file", &map.path);
            emit_str(out, "floor", &map.theme.0);
            emit_str(out, "wall", &map.theme.1);
            emit_bool(out, "start", map.start);
            emit_string_map_section(out, "map.objects", &map.objects);
        }
        BeginnerMapFile::Ldtk(map) => {
            emit_str(out, "kind", "ldtk");
            emit_str(out, "name", &map.name);
            emit_asset_file(out, "file", &map.path);
            emit_str(out, "level", &map.level);
            emit_str(out, "floor", &map.theme.0);
            emit_str(out, "wall", &map.theme.1);
            emit_bool(out, "start", map.start);
            emit_string_map_section(out, "map.entities", &map.entities);
        }
    }
}

fn emit_text_map_fields(
    out: &mut String,
    name: &str,
    path: &str,
    theme: &(String, String),
    tile_size: f32,
    legend: &BTreeMap<char, String>,
    start: bool,
) {
    emit_str(out, "name", name);
    emit_asset_file(out, "file", path);
    emit_str(out, "floor", &theme.0);
    emit_str(out, "wall", &theme.1);
    emit_f32(out, "tile_size", tile_size);
    emit_bool(out, "start", start);
    if !legend.is_empty() {
        out.push_str("\n[map.legend]\n");
        for (symbol, prefab) in legend {
            let key = symbol.to_string();
            let _ = writeln!(out, "{} = {}", quoted_key(&key), quoted(prefab));
        }
    }
}

fn emit_scene_flow(out: &mut String, scene_flow: &SceneFlowFile, _notes: &mut Vec<String>) {
    out.push_str("\n[scene_flow]\n");
    emit_optional_str(out, "menu", &scene_flow.menu);
    emit_optional_str(out, "game", &scene_flow.game);
    emit_optional_str(out, "game_over", &scene_flow.game_over);
    emit_optional_str(out, "win", &scene_flow.win);
    emit_optional_str(out, "menu_text", &scene_flow.menu_text);
    emit_optional_str(out, "game_over_text", &scene_flow.game_over_text);
    emit_optional_str(out, "game_over_button", &scene_flow.game_over_button);
    emit_optional_str(out, "win_text", &scene_flow.win_text);
    emit_optional_str(out, "win_button", &scene_flow.win_button);
    if let Some(action) = scene_flow.start_on {
        emit_str(out, "start_on", action_to_kebab(action));
    }
    if let Some(action) = scene_flow.restart_on {
        emit_str(out, "restart_on", action_to_kebab(action));
    }
    if let Some(condition) = scene_flow.win_condition {
        emit_str(out, "win_condition", win_condition_to_kebab(condition));
    }
    if let Some(button) = &scene_flow.menu_button {
        out.push_str("\n[scene_flow.menu_button]\n");
        emit_str(out, "label", &button.label);
        emit_str(out, "map", &button.map);
    }
}

fn emit_audio(out: &mut String, audio: &AudioFile) {
    if audio.music_on_scene.is_empty()
        && audio.master_volume.is_none()
        && audio.music_volume.is_none()
        && audio.sfx_volume.is_none()
    {
        return;
    }

    out.push_str("\n[audio]\n");
    emit_optional_f32(out, "master_volume", audio.master_volume);
    emit_optional_f32(out, "music_volume", audio.music_volume);
    emit_optional_f32(out, "sfx_volume", audio.sfx_volume);
    for (scene, playback) in &audio.music_on_scene {
        let _ = writeln!(out, "\n[audio.music_on_scene.{}]", quoted_key(scene));
        emit_str(out, "track", &playback.track);
        emit_f32(out, "volume", playback.volume);
        emit_optional_f32(out, "fade_in", playback.fade_in);
    }
}

fn emit_action(out: &mut String, action: &BeginnerActionFile) {
    out.push_str("\n[[action]]\n");
    match action {
        BeginnerActionFile::PlayerShoots(action) => {
            emit_str(out, "kind", "player-shoots");
            emit_str(out, "prefab", &action.prefab);
            emit_str(out, "action", action_to_kebab(action.action));
            emit_f32(out, "cooldown", action.cooldown);
            emit_str(out, "direction", shot_direction_to_kebab(action.direction));
            if let Some(sound) = &action.sound {
                emit_str(out, "sound", sound);
            }
        }
    }
}

fn emit_custom_rule(out: &mut String, rule: &CustomRuleFile) {
    out.push_str("\n[[custom_rule]]\n");
    match rule {
        CustomRuleFile::Countdown(rule) => {
            emit_str(out, "kind", "countdown");
            emit_str(out, "name", &rule.name);
            emit_str(out, "tag", &rule.tag);
            emit_str(out, "key", &rule.key);
            for effect in &rule.when_zero {
                emit_effect(out, "custom_rule.when_zero", effect);
            }
        }
    }
}

fn emit_rules(out: &mut String, rules: &[BeginnerRuleFile]) {
    let structured = rules
        .iter()
        .filter_map(|rule| match rule {
            BeginnerRuleFile::Structured(kind) => Some(rule_kind_to_kebab(*kind).to_owned()),
            BeginnerRuleFile::Legacy(name) => legacy_rule_name_to_kebab(name).map(str::to_owned),
            BeginnerRuleFile::Script(_) => None,
        })
        .collect::<Vec<_>>();
    if !structured.is_empty() {
        out.push_str("\n[rules]\n");
        emit_string_array_multiline(out, "enabled", &structured);
    }

    for rule in rules {
        let BeginnerRuleFile::Script(rule) = rule else {
            continue;
        };
        emit_script_rule(out, rule);
    }
}

fn emit_script_rule(out: &mut String, rule: &BeginnerScriptRuleFile) {
    out.push_str("\n[[rule]]\n");
    match rule {
        BeginnerScriptRuleFile::When { condition, effects } => {
            emit_condition(out, condition);
            for effect in effects {
                emit_effect(out, "rule.then", effect);
            }
        }
        BeginnerScriptRuleFile::OnEnemyDeath { prefab, effects } => {
            emit_str(out, "on", "enemy-death");
            emit_str(out, "prefab", prefab);
            for effect in effects {
                emit_effect(out, "rule.then", effect);
            }
        }
        BeginnerScriptRuleFile::EverySeconds { seconds, effects } => {
            emit_f32(out, "every_seconds", *seconds);
            for effect in effects {
                emit_effect(out, "rule.then", effect);
            }
        }
        BeginnerScriptRuleFile::OnScoreReaches { score, effects } => {
            emit_str(out, "on", "score-reaches");
            emit_i32(out, "score", *score);
            for effect in effects {
                emit_effect(out, "rule.then", effect);
            }
        }
    }
}

fn emit_condition(out: &mut String, condition: &RuleConditionFile) {
    match condition {
        RuleConditionFile::AllEnemiesDead => emit_str(out, "when", "all-enemies-dead"),
        RuleConditionFile::AllPickupsCollected => emit_str(out, "when", "all-pickups-collected"),
        RuleConditionFile::ScoreAtLeast(score) => {
            emit_str(out, "when", "score-at-least");
            emit_i32(out, "score", *score);
        }
        RuleConditionFile::PlayerHealthBelow(health) => {
            emit_str(out, "when", "player-health-below");
            emit_i32(out, "health", *health);
        }
        RuleConditionFile::TimerReached { name, seconds } => {
            emit_str(out, "when", "timer-reached");
            emit_str(out, "name", name);
            emit_f32(out, "seconds", *seconds);
        }
        RuleConditionFile::MapIs(map) => {
            emit_str(out, "when", "map-is");
            emit_str(out, "map", map);
        }
        RuleConditionFile::SceneIs(scene) => {
            emit_str(out, "when", "scene-is");
            emit_str(out, "scene", scene);
        }
        RuleConditionFile::TagCountZero(tag) => {
            emit_str(out, "when", "tag-count-zero");
            emit_str(out, "tag", tag);
        }
        RuleConditionFile::ActionPressed(action) => {
            emit_str(out, "when", "action-pressed");
            emit_str(out, "action", action_to_kebab(*action));
        }
    }
}

fn emit_effect(out: &mut String, table: &str, effect: &RuleEffectFile) {
    let _ = writeln!(out, "\n[[{table}]]");
    match effect {
        RuleEffectFile::AddScore(amount) => {
            emit_str(out, "action", "add-score");
            emit_i32(out, "amount", *amount);
        }
        RuleEffectFile::SetScore(score) => {
            emit_str(out, "action", "set-score");
            emit_i32(out, "score", *score);
        }
        RuleEffectFile::DamageTagged {
            tag,
            amount,
            radius,
        } => {
            emit_str(out, "action", "damage-tagged");
            emit_str(out, "tag", tag);
            emit_i32(out, "amount", *amount);
            emit_f32(out, "radius", *radius);
        }
        RuleEffectFile::DamagePlayer { amount, radius } => {
            emit_str(out, "action", "damage-player");
            emit_i32(out, "amount", *amount);
            emit_f32(out, "radius", *radius);
        }
        RuleEffectFile::DespawnSelf => emit_str(out, "action", "despawn-self"),
        RuleEffectFile::PlaySound(sound) => {
            emit_str(out, "action", "play-sound");
            emit_str(out, "sound", sound);
        }
        RuleEffectFile::PlayMusic(music) => {
            emit_str(out, "action", "play-music");
            emit_str(out, "music", music);
        }
        RuleEffectFile::StopMusic => emit_str(out, "action", "stop-music"),
        RuleEffectFile::SpawnPrefab(prefab) => {
            emit_str(out, "action", "spawn-prefab");
            emit_str(out, "prefab", prefab);
        }
        RuleEffectFile::SpawnNearPlayer { prefab, radius } => {
            emit_str(out, "action", "spawn-near-player");
            emit_str(out, "prefab", prefab);
            emit_f32(out, "radius", *radius);
        }
        RuleEffectFile::ChangeScene(scene) => {
            emit_str(out, "action", "change-scene");
            emit_str(out, "scene", scene);
        }
        RuleEffectFile::ChangeMap(map) => {
            emit_str(out, "action", "change-map");
            emit_str(out, "map", map);
        }
        RuleEffectFile::RestartCurrentMap => emit_str(out, "action", "restart-current-map"),
        RuleEffectFile::ShowUiText(text) => {
            emit_str(out, "action", "show-ui-text");
            emit_str(out, "text", text);
        }
        RuleEffectFile::HealPlayer(amount) => {
            emit_str(out, "action", "heal-player");
            emit_i32(out, "amount", *amount);
        }
        RuleEffectFile::SetData { tag, key, value } => {
            emit_str(out, "action", "set-data");
            emit_str(out, "tag", tag);
            emit_str(out, "key", key);
            emit_f32(out, "value", *value);
        }
        RuleEffectFile::DespawnTagged(tag) => {
            emit_str(out, "action", "despawn-tagged");
            emit_str(out, "tag", tag);
        }
    }
}

fn emit_string_array(out: &mut String, key: &str, values: &[String]) {
    let _ = writeln!(
        out,
        "{key} = [{}]",
        values
            .iter()
            .map(|value| quoted(value))
            .collect::<Vec<_>>()
            .join(", ")
    );
}

fn emit_string_array_if_any(out: &mut String, key: &str, values: &[String]) {
    if !values.is_empty() {
        emit_string_array(out, key, values);
    }
}

fn emit_string_array_multiline(out: &mut String, key: &str, values: &[String]) {
    let _ = writeln!(out, "{key} = [");
    for value in values {
        let _ = writeln!(out, "  {},", quoted(value));
    }
    out.push_str("]\n");
}

fn emit_string_map_section(out: &mut String, section: &str, values: &BTreeMap<String, String>) {
    if values.is_empty() {
        return;
    }
    let _ = writeln!(out, "\n[{section}]");
    for (key, value) in values {
        let _ = writeln!(out, "{} = {}", quoted_key(key), quoted(value));
    }
}

fn emit_data_if_any(out: &mut String, data: &BTreeMap<String, f32>) {
    if data.is_empty() {
        return;
    }
    let entries = data
        .iter()
        .map(|(key, value)| format!("{} = {}", quoted_key(key), number(*value)))
        .collect::<Vec<_>>()
        .join(", ");
    let _ = writeln!(out, "data = {{ {entries} }}");
}

fn emit_asset_file(out: &mut String, key: &str, path: &str) {
    let path = path.strip_prefix("assets/").unwrap_or(path);
    emit_str(out, key, &format!("assets/{path}"));
}

fn emit_optional_str(out: &mut String, key: &str, value: &Option<String>) {
    if let Some(value) = value {
        emit_str(out, key, value);
    }
}

fn emit_optional_f32(out: &mut String, key: &str, value: Option<f32>) {
    if let Some(value) = value {
        emit_f32(out, key, value);
    }
}

fn emit_str(out: &mut String, key: &str, value: &str) {
    let _ = writeln!(out, "{key} = {}", quoted(value));
}

fn emit_bool(out: &mut String, key: &str, value: bool) {
    let _ = writeln!(out, "{key} = {value}");
}

fn emit_i32(out: &mut String, key: &str, value: i32) {
    let _ = writeln!(out, "{key} = {value}");
}

fn emit_usize(out: &mut String, key: &str, value: usize) {
    let _ = writeln!(out, "{key} = {value}");
}

fn emit_f32(out: &mut String, key: &str, value: f32) {
    let _ = writeln!(out, "{key} = {}", number(value));
}

fn emit_vec2(out: &mut String, key: &str, value: (f32, f32)) {
    let _ = writeln!(out, "{key} = [{}, {}]", number(value.0), number(value.1));
}

fn quoted(value: &str) -> String {
    let mut quoted = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\\' => quoted.push_str("\\\\"),
            '"' => quoted.push_str("\\\""),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            _ => quoted.push(ch),
        }
    }
    quoted.push('"');
    quoted
}

fn quoted_key(value: &str) -> String {
    if !value.is_empty()
        && value
            .chars()
            .all(|ch| ch == '_' || ch == '-' || ch.is_ascii_alphanumeric())
    {
        value.to_owned()
    } else {
        quoted(value)
    }
}

fn number(value: f32) -> String {
    value.to_string()
}

fn action_to_kebab(action: ActionFile) -> &'static str {
    match action {
        ActionFile::Attack => "attack",
        ActionFile::Pause => "pause",
        ActionFile::Reset => "reset",
        ActionFile::Reload => "reload",
        ActionFile::MenuAccept => "menu-accept",
    }
}

fn shot_direction_to_kebab(direction: ShotDirectionFile) -> &'static str {
    match direction {
        ShotDirectionFile::TowardsMouse => "towards-mouse",
        ShotDirectionFile::Right => "right",
        ShotDirectionFile::Left => "left",
        ShotDirectionFile::Up => "up",
        ShotDirectionFile::Down => "down",
    }
}

fn win_condition_to_kebab(condition: WinConditionFile) -> &'static str {
    match condition {
        WinConditionFile::AllPickupsCollected => "all-pickups-collected",
        WinConditionFile::AllEnemiesDead => "all-enemies-dead",
    }
}

fn rule_kind_to_kebab(kind: BeginnerRuleKind) -> &'static str {
    match kind {
        BeginnerRuleKind::TopDownControls => "top-down-controls",
        BeginnerRuleKind::PlayerCollectsPickups => "player-collects-pickups",
        BeginnerRuleKind::EnemiesDamagePlayer => "enemies-damage-player",
        BeginnerRuleKind::DeadEnemiesDespawn => "dead-enemies-despawn",
        BeginnerRuleKind::EnemyDrops => "enemy-drops",
        BeginnerRuleKind::Projectiles => "projectiles",
        BeginnerRuleKind::ProjectilesMove => "projectiles-move",
        BeginnerRuleKind::ProjectilesExpireAfterLifetime => "projectiles-expire-after-duration",
        BeginnerRuleKind::ProjectilesDamageEnemies => "projectiles-damage-enemies",
        BeginnerRuleKind::ProjectilesDespawnOnHit => "projectiles-despawn-on-hit",
        BeginnerRuleKind::ProjectileImpactAnimationBeforeDespawn => {
            "projectile-impact-animation-before-despawn"
        }
        BeginnerRuleKind::SpawnersSpawnPrefabs => "spawners-spawn-prefabs",
        BeginnerRuleKind::DoorsChangeMaps => "doors-change-maps",
        BeginnerRuleKind::PlayerActivatesCheckpoints => "player-activates-checkpoints",
        BeginnerRuleKind::RespawnAtCheckpoint => "respawn-at-checkpoint",
        BeginnerRuleKind::CameraFollowsPlayer => "camera-follows-player",
        BeginnerRuleKind::PauseAndReset => "pause-and-reset",
        BeginnerRuleKind::ShowBasicUi => "show-basic-ui",
        BeginnerRuleKind::ShowScore => "show-score",
        BeginnerRuleKind::ShowEnemyCount => "show-enemy-count",
        BeginnerRuleKind::ShowPlayerHealth => "show-player-health",
        BeginnerRuleKind::ShowMenu => "show-menu",
        BeginnerRuleKind::ShowPauseMenu => "show-pause-menu",
        BeginnerRuleKind::ShowGameOverPanel => "show-game-over-panel",
        BeginnerRuleKind::ShowWinPanel => "show-win-panel",
        BeginnerRuleKind::WinWhenAllPickupsCollected => "win-when-all-pickups-collected",
        BeginnerRuleKind::WinWhenAllEnemiesDead => "win-when-all-enemies-dead",
        BeginnerRuleKind::AnimateEnemiesByMovement => "animate-enemies-by-movement",
        BeginnerRuleKind::AnimatePlayerDirectionally => "animate-player-directionally",
        BeginnerRuleKind::AnimateEnemiesDirectionally => "animate-enemies-directionally",
        BeginnerRuleKind::AnimateAttacksDirectionally => "animate-attacks-directionally",
        BeginnerRuleKind::DeadEnemiesPlayDeathAnimation => "dead-enemies-play-death-animation",
        BeginnerRuleKind::DeadEnemiesDespawnAfterAnimation => {
            "dead-enemies-despawn-after-animation"
        }
    }
}

fn legacy_rule_name_to_kebab(name: &str) -> Option<&'static str> {
    match name {
        "top_down_controls" | "TopDownControls" => Some("top-down-controls"),
        "player_collects_pickups" | "PlayerCollectsPickups" => Some("player-collects-pickups"),
        "enemies_damage_player" | "EnemiesDamagePlayer" => Some("enemies-damage-player"),
        "dead_enemies_despawn" | "DeadEnemiesDespawn" => Some("dead-enemies-despawn"),
        "enemy_drops" | "EnemyDrops" => Some("enemy-drops"),
        "projectiles" | "Projectiles" => Some("projectiles"),
        "projectiles_move" | "ProjectilesMove" => Some("projectiles-move"),
        "projectiles_expire_after_lifetime" | "ProjectilesExpireAfterLifetime" => {
            Some("projectiles-expire-after-duration")
        }
        "projectiles_damage_enemies" | "ProjectilesDamageEnemies" => {
            Some("projectiles-damage-enemies")
        }
        "projectiles_despawn_on_hit" | "ProjectilesDespawnOnHit" => {
            Some("projectiles-despawn-on-hit")
        }
        "projectile_impact_animation_before_despawn" | "ProjectileImpactAnimationBeforeDespawn" => {
            Some("projectile-impact-animation-before-despawn")
        }
        "spawners_spawn_prefabs" | "SpawnersSpawnPrefabs" => Some("spawners-spawn-prefabs"),
        "doors_change_maps" | "DoorsChangeMaps" => Some("doors-change-maps"),
        "player_activates_checkpoints" | "PlayerActivatesCheckpoints" => {
            Some("player-activates-checkpoints")
        }
        "respawn_at_checkpoint" | "RespawnAtCheckpoint" => Some("respawn-at-checkpoint"),
        "camera_follows_player" | "CameraFollowsPlayer" => Some("camera-follows-player"),
        "pause_and_reset" | "PauseAndReset" => Some("pause-and-reset"),
        "show_basic_ui" | "ShowBasicUi" => Some("show-basic-ui"),
        "show_score" | "ShowScore" => Some("show-score"),
        "show_enemy_count" | "ShowEnemyCount" => Some("show-enemy-count"),
        "show_player_health" | "ShowPlayerHealth" => Some("show-player-health"),
        "show_menu" | "ShowMenu" => Some("show-menu"),
        "show_pause_menu" | "ShowPauseMenu" => Some("show-pause-menu"),
        "show_game_over_panel" | "ShowGameOverPanel" => Some("show-game-over-panel"),
        "show_win_panel" | "ShowWinPanel" => Some("show-win-panel"),
        "win_when_all_pickups_collected" | "WinWhenAllPickupsCollected" => {
            Some("win-when-all-pickups-collected")
        }
        "win_when_all_enemies_dead" | "WinWhenAllEnemiesDead" => Some("win-when-all-enemies-dead"),
        "animate_enemies_by_movement" | "AnimateEnemiesByMovement" => {
            Some("animate-enemies-by-movement")
        }
        "animate_player_directionally" | "AnimatePlayerDirectionally" => {
            Some("animate-player-directionally")
        }
        "animate_enemies_directionally" | "AnimateEnemiesDirectionally" => {
            Some("animate-enemies-directionally")
        }
        "animate_attacks_directionally" | "AnimateAttacksDirectionally" => {
            Some("animate-attacks-directionally")
        }
        "dead_enemies_play_death_animation" | "DeadEnemiesPlayDeathAnimation" => {
            Some("dead-enemies-play-death-animation")
        }
        "dead_enemies_despawn_after_animation" | "DeadEnemiesDespawnAfterAnimation" => {
            Some("dead-enemies-despawn-after-animation")
        }
        _ => None,
    }
}
