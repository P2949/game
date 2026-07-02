//! Validation for data-driven beginner game files.
//!
//! This is the beginner safety layer: it rejects unsupported versions,
//! missing references, invalid numeric values, and rule combinations before
//! authoring data reaches runtime builders.

use super::*;

#[cfg(test)]
pub(super) fn validate_file(file: &BeginnerGameFile, label: &str) -> Result<()> {
    validate_file_with_base(file, label, None)
}

pub(super) fn validate_file_with_base(
    file: impl Into<AuthoringGameFile>,
    label: &str,
    asset_base: Option<&Path>,
) -> Result<()> {
    let file = file.into();
    if file.version != 1 {
        anyhow::bail!(
            "unsupported beginner game file version {}. Supported version: 1",
            file.version
        );
    }
    file.controls.kind(label)?;

    let textures = file
        .assets
        .textures
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let sounds = file
        .assets
        .sounds
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let music = file
        .assets
        .music
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let animation_sheets = file
        .assets
        .animation_sheets
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    reject_duplicates(label, "texture asset", &textures)?;
    reject_duplicates(label, "sound asset", &sounds)?;
    reject_duplicates(label, "music asset", &music)?;
    reject_duplicates(label, "animation sheet asset", &animation_sheets)?;

    let prefab_names = file
        .prefabs
        .iter()
        .map(BeginnerPrefabFile::name)
        .collect::<Vec<_>>();
    reject_duplicates(label, "prefab", &prefab_names)?;
    let custom_rule_names = file
        .custom_rules
        .iter()
        .map(CustomRuleFile::name)
        .collect::<Vec<_>>();
    reject_duplicates(label, "custom rule", &custom_rule_names)?;
    let map_names = file
        .maps
        .iter()
        .map(BeginnerMapFile::name)
        .collect::<Vec<_>>();
    reject_duplicates(label, "map", &map_names)?;

    if !file.maps.is_empty() {
        let starts = file.maps.iter().filter(|map| map.start()).count();
        if starts != 1 {
            anyhow::bail!(
                "beginner game file '{label}' must mark exactly one map with start: true; found {starts}"
            );
        }
    }

    let tags = file
        .prefabs
        .iter()
        .flat_map(BeginnerPrefabFile::tags)
        .collect::<Vec<_>>();
    let prefab_data = build_prefab_data_index(&file.prefabs);
    let scene_names = scene_names(&file);
    let scene_name_refs = scene_names.iter().map(String::as_str).collect::<Vec<_>>();

    for prefab in &file.prefabs {
        for (owner, texture) in prefab.texture_refs() {
            require_known(label, "texture", owner, texture, &textures)?;
        }
        for (owner, sound) in prefab.sound_refs() {
            require_known(label, "sound", owner, sound, &sounds)?;
        }
        for (owner, sheet) in prefab.animation_sheet_refs() {
            require_known(label, "animation sheet", owner, sheet, &animation_sheets)?;
        }
        for (owner, referenced) in prefab.prefab_refs() {
            require_known(label, "prefab", owner, referenced, &prefab_names)?;
        }
        for (owner, map) in prefab.map_refs() {
            require_known(label, "map", owner, map, &map_names)?;
        }
        for (owner, scene) in prefab.scene_refs() {
            if scene_name_refs.is_empty() {
                anyhow::bail!(
                    "beginner game file '{label}' {owner} changes to scene '{scene}', but no scene_flow declares scenes"
                );
            }
            require_known(label, "scene", owner, scene, &scene_name_refs)?;
        }
        prefab.validate_numbers(label)?;
    }

    for map in &file.maps {
        for (owner, texture) in map.texture_refs() {
            require_known(
                label,
                "texture",
                &format!("map '{owner}'"),
                texture,
                &textures,
            )?;
        }
        for (owner, prefab) in map.prefab_refs() {
            require_known(
                label,
                "prefab",
                &format!("map '{owner}'"),
                prefab,
                &prefab_names,
            )?;
        }
        validate_map_file(label, map, asset_base)?;
    }

    if let Some(flow) = &file.scene_flow {
        validate_scene_flow(label, flow, &map_names)?;
    }
    validate_audio(label, &file.audio, &music, &scene_name_refs)?;
    validate_actions(label, &file.actions, &prefab_names, &sounds)?;
    let names = ValidationNames {
        prefabs: &prefab_names,
        sounds: &sounds,
        music: &music,
        maps: &map_names,
        scenes: &scene_name_refs,
        tags: &tags,
    };
    validate_custom_rules(label, &file.custom_rules, &names, &prefab_data)?;

    for rule in &file.rules {
        rule.identity(label)?;
        if let BeginnerRuleFile::Script(rule) = rule {
            validate_script_rule(label, rule, &names)?;
        }
    }
    validate_rule_combinations(label, &file.rules, &file.prefabs)
}

fn reject_duplicates(label: &str, kind: &str, names: &[&str]) -> Result<()> {
    let unique = names.iter().copied().collect::<BTreeSet<_>>();
    if unique.len() != names.len() {
        anyhow::bail!("beginner game file '{label}' defines duplicate {kind} names");
    }
    Ok(())
}

fn require_known(label: &str, kind: &str, owner: &str, key: &str, known: &[&str]) -> Result<()> {
    if known.contains(&key) {
        return Ok(());
    }
    Err(unknown_reference_error(label, owner, kind, key, known))
}

fn validate_map_file(label: &str, map: &BeginnerMapFile, asset_base: Option<&Path>) -> Result<()> {
    match map {
        BeginnerMapFile::TextMap(map) => {
            validate_text_map_symbols(label, &map.name, &map.path, &map.legend, asset_base)
        }
        BeginnerMapFile::TextMapAuto(map) => {
            let path = format!("maps/{}.txt", map.name);
            validate_text_map_symbols(label, &map.name, &path, &map.legend, asset_base)
        }
        BeginnerMapFile::Tiled(map) => {
            require_asset_file(label, "Tiled map", &map.name, &map.path, asset_base)
        }
        BeginnerMapFile::Ldtk(map) => {
            require_asset_file(label, "LDtk map", &map.name, &map.path, asset_base)
        }
    }
}

fn validate_text_map_symbols(
    label: &str,
    map_name: &str,
    path: &str,
    legend: &BTreeMap<char, String>,
    asset_base: Option<&Path>,
) -> Result<()> {
    let full_path = resolved_asset_file(asset_base, path);
    let source = std::fs::read_to_string(&full_path).with_context(|| {
        format!(
            "beginner game file '{label}' map '{map_name}' could not read text map 'assets/{path}' (looked for '{}')",
            full_path.display()
        )
    })?;
    let known_symbols = legend.keys().copied().collect::<Vec<_>>();
    for (row, line) in source.lines().enumerate() {
        for (col, symbol) in line.trim_end_matches('\r').chars().enumerate() {
            if symbol == '.' || symbol == '#' || legend.contains_key(&symbol) {
                continue;
            }
            anyhow::bail!(
                "beginner game file '{label}' map '{map_name}' has an invalid symbol.\n\n{}",
                bad_map_symbol_error(map_name, symbol, row, col, &known_symbols)
            );
        }
    }
    Ok(())
}

fn require_asset_file(
    label: &str,
    kind: &str,
    owner: &str,
    path: &str,
    asset_base: Option<&Path>,
) -> Result<()> {
    let full_path = resolved_asset_file(asset_base, path);
    if full_path.is_file() {
        return Ok(());
    }
    anyhow::bail!(
        "beginner game file '{label}' {kind} '{owner}' references a missing file.\n\n{}",
        missing_file_error(kind, &format!("assets/{path}"), &full_path)
    )
}

fn resolved_asset_file(asset_base: Option<&Path>, relative: &str) -> std::path::PathBuf {
    let path = Path::new(relative);
    if path.is_absolute() {
        return path.to_path_buf();
    }
    if let Some(base) = asset_base {
        return base.join(path);
    }
    beginner_asset_path(relative)
}

fn validate_scene_flow(label: &str, flow: &SceneFlowFile, map_names: &[&str]) -> Result<()> {
    if let Some(game_map) = flow.game.as_deref() {
        require_known(label, "map", "scene_flow.game", game_map, map_names)?;
    }
    if let Some(game_over) = flow.game_over.as_deref() {
        require_known(label, "map", "scene_flow.game_over", game_over, map_names)?;
    }
    if let Some(win) = flow.win.as_deref() {
        require_known(label, "map", "scene_flow.win", win, map_names)?;
    }
    if let Some(button) = &flow.menu_button {
        require_known(
            label,
            "map",
            "scene_flow.menu_button",
            &button.map,
            map_names,
        )?;
    }
    Ok(())
}

fn validate_audio(label: &str, audio: &AudioFile, music: &[&str], scenes: &[&str]) -> Result<()> {
    if !audio.music_on_scene.is_empty() && scenes.is_empty() {
        anyhow::bail!(
            "beginner game file '{label}' audio.music_on_scene needs scene_flow so scene names exist"
        );
    }
    for (scene, playback) in &audio.music_on_scene {
        require_known(label, "scene", "audio.music_on_scene", scene, scenes)?;
        require_known(
            label,
            "music",
            &format!("audio.music_on_scene '{scene}'"),
            &playback.track,
            music,
        )?;
        if !playback.volume.is_finite() || playback.volume < 0.0 {
            anyhow::bail!(
                "beginner game file '{label}' audio.music_on_scene '{scene}' has invalid volume {}; use a finite non-negative number",
                playback.volume
            );
        }
    }
    for (field, volume) in [
        ("master_volume", audio.master_volume),
        ("music_volume", audio.music_volume),
        ("sfx_volume", audio.sfx_volume),
    ] {
        if let Some(volume) = volume
            && (!volume.is_finite() || volume < 0.0)
        {
            anyhow::bail!(
                "beginner game file '{label}' audio.{field} has invalid volume {volume}; use a finite non-negative number"
            );
        }
    }
    Ok(())
}

fn validate_actions(
    label: &str,
    actions: &[BeginnerActionFile],
    prefabs: &[&str],
    sounds: &[&str],
) -> Result<()> {
    for action in actions {
        match action {
            BeginnerActionFile::PlayerShoots(shoot) => {
                require_known(
                    label,
                    "prefab",
                    "action PlayerShoots",
                    &shoot.prefab,
                    prefabs,
                )?;
                if let Some(sound) = shoot.sound.as_deref() {
                    require_known(label, "sound", "action PlayerShoots", sound, sounds)?;
                }
                if !shoot.cooldown.is_finite() || shoot.cooldown < 0.0 {
                    anyhow::bail!(
                        "beginner game file '{label}' action PlayerShoots has invalid cooldown {}; use a finite non-negative number",
                        shoot.cooldown
                    );
                }
            }
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct ValidationNames<'a> {
    prefabs: &'a [&'a str],
    sounds: &'a [&'a str],
    music: &'a [&'a str],
    maps: &'a [&'a str],
    scenes: &'a [&'a str],
    tags: &'a [&'a str],
}

#[derive(Default)]
struct PrefabDataIndex {
    tags: BTreeSet<String>,
    tag_to_data_keys: BTreeMap<String, BTreeSet<String>>,
    tag_to_prefabs: BTreeMap<String, Vec<String>>,
}

fn build_prefab_data_index(prefabs: &[BeginnerPrefabFile]) -> PrefabDataIndex {
    let mut index = PrefabDataIndex::default();
    for prefab in prefabs {
        let name = prefab.name();
        let data_keys = prefab.data_keys();
        for tag in prefab.tags() {
            index.tags.insert(tag.to_owned());
            index
                .tag_to_prefabs
                .entry(tag.to_owned())
                .or_default()
                .push(name.to_owned());
            let keys = index.tag_to_data_keys.entry(tag.to_owned()).or_default();
            keys.extend(data_keys.iter().map(|key| (*key).to_owned()));
        }
    }
    index
}

fn validate_custom_rules(
    label: &str,
    custom_rules: &[CustomRuleFile],
    names: &ValidationNames<'_>,
    prefab_data: &PrefabDataIndex,
) -> Result<()> {
    for custom_rule in custom_rules {
        match custom_rule {
            CustomRuleFile::Countdown(rule) => {
                if rule.tag.trim().is_empty() {
                    anyhow::bail!(
                        "beginner game file '{label}' custom rule '{}' has an empty tag",
                        rule.name
                    );
                }
                if rule.key.trim().is_empty() {
                    anyhow::bail!(
                        "beginner game file '{label}' custom rule '{}' has an empty countdown key",
                        rule.name
                    );
                }
                require_known(
                    label,
                    "tag",
                    &format!("custom rule '{}'", rule.name),
                    &rule.tag,
                    names.tags,
                )?;
                validate_countdown_key_for_tag(label, rule, prefab_data)?;
                for effect in &rule.when_zero {
                    match effect {
                        RuleEffectFile::DamageTagged { tag, radius, .. } => {
                            require_known(
                                label,
                                "tag",
                                &format!("custom rule '{}'", rule.name),
                                tag,
                                names.tags,
                            )?;
                            validate_radius(label, &rule.name, *radius)?;
                        }
                        RuleEffectFile::DamagePlayer { radius, .. } => {
                            validate_radius(label, &rule.name, *radius)?;
                        }
                        RuleEffectFile::AddScore(_) => {}
                        RuleEffectFile::SetScore(_) => {}
                        RuleEffectFile::DespawnSelf => {}
                        RuleEffectFile::PlaySound(sound) => {
                            require_known(
                                label,
                                "sound",
                                &format!("custom rule '{}'", rule.name),
                                sound,
                                names.sounds,
                            )?;
                        }
                        RuleEffectFile::PlayMusic(track) => {
                            require_known(
                                label,
                                "music",
                                &format!("custom rule '{}'", rule.name),
                                track,
                                names.music,
                            )?;
                        }
                        RuleEffectFile::StopMusic => {}
                        RuleEffectFile::SpawnPrefab(prefab) => {
                            require_known(
                                label,
                                "prefab",
                                &format!("custom rule '{}'", rule.name),
                                prefab,
                                names.prefabs,
                            )?;
                        }
                        RuleEffectFile::SpawnNearPlayer { prefab, radius } => {
                            require_known(
                                label,
                                "prefab",
                                &format!("custom rule '{}'", rule.name),
                                prefab,
                                names.prefabs,
                            )?;
                            validate_radius(label, &rule.name, *radius)?;
                        }
                        RuleEffectFile::ChangeScene(scene) => {
                            if names.scenes.is_empty() {
                                anyhow::bail!(
                                    "beginner game file '{label}' custom rule '{}' changes to scene '{scene}', but no scene_flow declares scenes",
                                    rule.name
                                );
                            }
                            require_known(
                                label,
                                "scene",
                                &format!("custom rule '{}'", rule.name),
                                scene,
                                names.scenes,
                            )?;
                        }
                        RuleEffectFile::ChangeMap(map) => {
                            require_known(
                                label,
                                "map",
                                &format!("custom rule '{}'", rule.name),
                                map,
                                names.maps,
                            )?;
                        }
                        RuleEffectFile::RestartCurrentMap => {}
                        RuleEffectFile::ShowUiText(text) => {
                            validate_text(label, &format!("custom rule '{}'", rule.name), text)?;
                        }
                        RuleEffectFile::HealPlayer(_) => {}
                        RuleEffectFile::SetData { tag, key, value } => {
                            require_known(
                                label,
                                "tag",
                                &format!("custom rule '{}'", rule.name),
                                tag,
                                names.tags,
                            )?;
                            validate_data_assignment(
                                label,
                                &format!("custom rule '{}'", rule.name),
                                key,
                                *value,
                            )?;
                        }
                        RuleEffectFile::DespawnTagged(tag) => {
                            require_known(
                                label,
                                "tag",
                                &format!("custom rule '{}'", rule.name),
                                tag,
                                names.tags,
                            )?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn validate_countdown_key_for_tag(
    label: &str,
    rule: &CountdownRuleFile,
    prefab_data: &PrefabDataIndex,
) -> Result<()> {
    if !prefab_data.tags.contains(&rule.tag) {
        anyhow::bail!(
            "beginner game file '{label}' custom rule '{}' counts down tag '{}', but no prefab declares that tag",
            rule.name,
            rule.tag
        );
    }

    let keys = prefab_data
        .tag_to_data_keys
        .get(&rule.tag)
        .cloned()
        .unwrap_or_default();
    if keys.contains(&rule.key) {
        return Ok(());
    }

    let prefabs = prefab_data
        .tag_to_prefabs
        .get(&rule.tag)
        .cloned()
        .unwrap_or_default();
    let known = keys.iter().map(String::as_str).collect::<Vec<_>>();
    let suggestion = closest_name(&rule.key, known.iter().copied())
        .map(|candidate| format!("\n\nDid you mean '{candidate}'?"))
        .unwrap_or_default();
    let empty_keys_note = if keys.is_empty() {
        format!(
            "\n\nNo prefab tagged '{}' declares any data keys.",
            rule.tag
        )
    } else {
        String::new()
    };

    anyhow::bail!(
        "beginner game file '{label}' custom rule '{}' counts down key '{}' on tag '{}', but no prefab with that tag declares that data key.\n\nPrefabs with tag '{}': {}\nKnown data keys for tag '{}': {}{}{}\n\nFix: add data: {{\"{}\": 3.0}} to one of those prefabs, or change the rule key to one of the known data keys.",
        rule.name,
        rule.key,
        rule.tag,
        rule.tag,
        joined_strings_or_none(&prefabs),
        rule.tag,
        joined_names_or_none(&known),
        empty_keys_note,
        suggestion,
        rule.key,
    );
}

fn joined_strings_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "(none)".to_owned()
    } else {
        values.join(", ")
    }
}

fn joined_names_or_none(values: &[&str]) -> String {
    if values.is_empty() {
        "(none)".to_owned()
    } else {
        values.join(", ")
    }
}

fn validate_radius(label: &str, owner: &str, radius: f32) -> Result<()> {
    if !radius.is_finite() || radius < 0.0 {
        anyhow::bail!(
            "beginner game file '{label}' custom rule '{owner}' has invalid radius {radius}; use a finite non-negative number"
        );
    }
    Ok(())
}

fn validate_text(label: &str, owner: &str, text: &str) -> Result<()> {
    if text.trim().is_empty() {
        anyhow::bail!("beginner game file '{label}' {owner} has empty text");
    }
    Ok(())
}

fn validate_data_assignment(label: &str, owner: &str, key: &str, value: f32) -> Result<()> {
    if key.trim().is_empty() {
        anyhow::bail!("beginner game file '{label}' {owner} has SetData with an empty key");
    }
    if !value.is_finite() {
        anyhow::bail!(
            "beginner game file '{label}' {owner} has invalid SetData value {value}; use a finite number"
        );
    }
    Ok(())
}

fn validate_non_negative(label: &str, owner: &str, field: &str, value: i32) -> Result<()> {
    if value < 0 {
        anyhow::bail!(
            "beginner game file '{label}' {owner} has negative {field} {value}; use zero or a positive number"
        );
    }
    Ok(())
}

fn validate_script_rule(
    label: &str,
    rule: &BeginnerScriptRuleFile,
    names: &ValidationNames<'_>,
) -> Result<()> {
    match rule {
        BeginnerScriptRuleFile::When { condition, effects } => {
            validate_rule_condition(label, "When", condition, names)?;
            validate_script_effects(label, "When", ScriptEffectScope::Game, effects, names)
        }
        BeginnerScriptRuleFile::OnEnemyDeath { prefab, effects } => {
            require_known(label, "prefab", "OnEnemyDeath", prefab, names.prefabs)?;
            validate_script_effects(
                label,
                "OnEnemyDeath",
                ScriptEffectScope::EnemyDeath,
                effects,
                names,
            )
        }
        BeginnerScriptRuleFile::EverySeconds { seconds, effects } => {
            if !seconds.is_finite() || *seconds <= 0.0 {
                anyhow::bail!(
                    "beginner game file '{label}' EverySeconds has invalid seconds {seconds}; use a positive number"
                );
            }
            validate_script_effects(
                label,
                "EverySeconds",
                ScriptEffectScope::Game,
                effects,
                names,
            )
        }
        BeginnerScriptRuleFile::OnScoreReaches { effects, .. } => validate_script_effects(
            label,
            "OnScoreReaches",
            ScriptEffectScope::Game,
            effects,
            names,
        ),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScriptEffectScope {
    Game,
    EnemyDeath,
}

fn validate_rule_condition(
    label: &str,
    owner: &str,
    condition: &RuleConditionFile,
    names: &ValidationNames<'_>,
) -> Result<()> {
    match condition {
        RuleConditionFile::AllEnemiesDead
        | RuleConditionFile::AllPickupsCollected
        | RuleConditionFile::ScoreAtLeast(_)
        | RuleConditionFile::PlayerHealthBelow(_)
        | RuleConditionFile::ActionPressed(_) => {}
        RuleConditionFile::TimerReached { name, seconds } => {
            if name.trim().is_empty() {
                anyhow::bail!(
                    "beginner game file '{label}' {owner} has TimerReached with an empty name"
                );
            }
            if !seconds.is_finite() || *seconds < 0.0 {
                anyhow::bail!(
                    "beginner game file '{label}' {owner} has invalid TimerReached seconds {seconds}; use a finite non-negative number"
                );
            }
        }
        RuleConditionFile::MapIs(map) => {
            require_known(label, "map", owner, map, names.maps)?;
        }
        RuleConditionFile::SceneIs(scene) => {
            if names.scenes.is_empty() {
                anyhow::bail!(
                    "beginner game file '{label}' {owner} checks scene '{scene}', but no scene_flow declares scenes"
                );
            }
            require_known(label, "scene", owner, scene, names.scenes)?;
        }
        RuleConditionFile::TagCountZero(tag) => {
            require_known(label, "tag", owner, tag, names.tags)?;
        }
    }
    Ok(())
}

fn validate_script_effects(
    label: &str,
    owner: &str,
    scope: ScriptEffectScope,
    effects: &[RuleEffectFile],
    names: &ValidationNames<'_>,
) -> Result<()> {
    for effect in effects {
        match effect {
            RuleEffectFile::AddScore(_)
            | RuleEffectFile::SetScore(_)
            | RuleEffectFile::DespawnSelf
            | RuleEffectFile::StopMusic
            | RuleEffectFile::RestartCurrentMap => {
                validate_game_effect_scope(label, owner, scope, effect)?;
            }
            RuleEffectFile::PlaySound(sound) => {
                validate_game_effect_scope(label, owner, scope, effect)?;
                require_known(label, "sound", owner, sound, names.sounds)?;
            }
            RuleEffectFile::PlayMusic(track) => {
                validate_game_effect_scope(label, owner, scope, effect)?;
                require_known(label, "music", owner, track, names.music)?;
            }
            RuleEffectFile::SpawnPrefab(prefab) => {
                validate_game_effect_scope(label, owner, scope, effect)?;
                require_known(label, "prefab", owner, prefab, names.prefabs)?;
            }
            RuleEffectFile::SpawnNearPlayer { prefab, radius } => {
                validate_game_effect_scope(label, owner, scope, effect)?;
                require_known(label, "prefab", owner, prefab, names.prefabs)?;
                if !radius.is_finite() || *radius < 0.0 {
                    anyhow::bail!(
                        "beginner game file '{label}' {owner} has invalid SpawnNearPlayer radius {radius}; use a finite non-negative number"
                    );
                }
            }
            RuleEffectFile::ChangeScene(scene) => {
                validate_game_effect_scope(label, owner, scope, effect)?;
                if names.scenes.is_empty() {
                    anyhow::bail!(
                        "beginner game file '{label}' {owner} changes to scene '{scene}', but no scene_flow declares scenes"
                    );
                }
                require_known(label, "scene", owner, scene, names.scenes)?;
            }
            RuleEffectFile::ChangeMap(map) => {
                validate_game_effect_scope(label, owner, scope, effect)?;
                require_known(label, "map", owner, map, names.maps)?;
            }
            RuleEffectFile::ShowUiText(text) => {
                validate_game_effect_scope(label, owner, scope, effect)?;
                validate_text(label, owner, text)?;
            }
            RuleEffectFile::DamagePlayer { amount, .. } => {
                validate_game_effect_scope(label, owner, scope, effect)?;
                validate_non_negative(label, owner, "DamagePlayer amount", *amount)?;
            }
            RuleEffectFile::HealPlayer(amount) => {
                validate_game_effect_scope(label, owner, scope, effect)?;
                validate_non_negative(label, owner, "HealPlayer amount", *amount)?;
            }
            RuleEffectFile::SetData { tag, key, value } => {
                validate_game_effect_scope(label, owner, scope, effect)?;
                require_known(label, "tag", owner, tag, names.tags)?;
                validate_data_assignment(label, owner, key, *value)?;
            }
            RuleEffectFile::DespawnTagged(tag) => {
                validate_game_effect_scope(label, owner, scope, effect)?;
                require_known(label, "tag", owner, tag, names.tags)?;
            }
            RuleEffectFile::DamageTagged { .. } => {
                anyhow::bail!(
                    "beginner game file '{label}' {owner} uses the countdown-only DamageTagged effect. Use DamageTagged inside custom_rules countdowns, or use script effects like AddScore, PlaySound, PlayMusic, SpawnPrefab, SpawnNearPlayer, ChangeScene, ChangeMap, DamagePlayer, HealPlayer, SetData, DespawnTagged, and ShowUiText."
                );
            }
        }
    }
    Ok(())
}

fn validate_game_effect_scope(
    label: &str,
    owner: &str,
    scope: ScriptEffectScope,
    effect: &RuleEffectFile,
) -> Result<()> {
    if scope == ScriptEffectScope::Game {
        return Ok(());
    }
    match effect {
        RuleEffectFile::AddScore(_)
        | RuleEffectFile::SetScore(_)
        | RuleEffectFile::DespawnSelf
        | RuleEffectFile::PlaySound(_)
        | RuleEffectFile::SpawnPrefab(_)
        | RuleEffectFile::SpawnNearPlayer { .. }
        | RuleEffectFile::ChangeScene(_)
        | RuleEffectFile::ChangeMap(_) => Ok(()),
        _ => {
            anyhow::bail!(
                "beginner game file '{label}' {owner} uses an effect that only works in When, EverySeconds, or OnScoreReaches rules"
            );
        }
    }
}

fn validate_rule_combinations(
    label: &str,
    rules: &[BeginnerRuleFile],
    prefabs: &[BeginnerPrefabFile],
) -> Result<()> {
    let kinds = rules
        .iter()
        .filter_map(|rule| rule.simple_kind(label).ok().flatten())
        .collect::<Vec<_>>();
    let has_checkpoint = prefabs
        .iter()
        .any(|prefab| matches!(prefab, BeginnerPrefabFile::Checkpoint(_)));
    let has_projectile = prefabs
        .iter()
        .any(|prefab| matches!(prefab, BeginnerPrefabFile::Projectile(_)));
    if !has_checkpoint
        && kinds.iter().any(|kind| {
            matches!(
                kind,
                BeginnerRuleKind::PlayerActivatesCheckpoints
                    | BeginnerRuleKind::RespawnAtCheckpoint
            )
        })
    {
        anyhow::bail!(
            "beginner game file '{label}' enables checkpoint rules but defines no Checkpoint prefab"
        );
    }
    if !has_projectile
        && kinds.iter().any(|kind| {
            matches!(
                kind,
                BeginnerRuleKind::Projectiles
                    | BeginnerRuleKind::ProjectilesMove
                    | BeginnerRuleKind::ProjectilesExpireAfterLifetime
                    | BeginnerRuleKind::ProjectilesDamageEnemies
                    | BeginnerRuleKind::ProjectilesDespawnOnHit
                    | BeginnerRuleKind::ProjectileImpactAnimationBeforeDespawn
            )
        })
    {
        anyhow::bail!(
            "beginner game file '{label}' enables projectile rules but defines no Projectile prefab.\n\nAdd a Projectile prefab such as:\n    Projectile((name: \"bolt\", sprite: \"bolt\"))"
        );
    }
    if kinds.contains(&BeginnerRuleKind::ProjectilesDamageEnemies)
        && !kinds.contains(&BeginnerRuleKind::Projectiles)
        && !kinds.contains(&BeginnerRuleKind::ProjectilesMove)
    {
        return Err(bad_rule_combo_error(
            "ProjectilesDamageEnemies",
            "Projectiles",
        ));
    }
    if kinds.contains(&BeginnerRuleKind::ProjectileImpactAnimationBeforeDespawn)
        && !kinds.contains(&BeginnerRuleKind::ProjectilesDespawnOnHit)
        && !kinds.contains(&BeginnerRuleKind::Projectiles)
    {
        return Err(bad_rule_combo_error(
            "ProjectileImpactAnimationBeforeDespawn",
            "ProjectilesDespawnOnHit",
        ));
    }
    Ok(())
}

fn scene_names(file: &AuthoringGameFile) -> Vec<String> {
    let Some(flow) = &file.scene_flow else {
        return Vec::new();
    };
    let mut names = Vec::new();
    for name in [&flow.menu, &flow.game, &flow.game_over, &flow.win]
        .into_iter()
        .filter_map(Option::as_deref)
    {
        if !names.iter().any(|candidate| candidate == name) {
            names.push(name.to_owned());
        }
    }
    names
}

pub(super) fn validate_size(label: &str, kind: &str, name: &str, size: (f32, f32)) -> Result<()> {
    if !size.0.is_finite() || !size.1.is_finite() || size.0 <= 0.0 || size.1 <= 0.0 {
        anyhow::bail!(
            "beginner game file '{label}' {kind} '{name}' has invalid size ({}, {}); use positive finite numbers",
            size.0,
            size.1
        );
    }
    Ok(())
}
