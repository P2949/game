use anyhow::{Context, Result, anyhow, bail};
use toml::Value;

use super::AuthoringGameFile;
use crate::diagnostics::closest_name;

pub(super) fn parse_toml_authoring_source(source: &str, label: &str) -> Result<AuthoringGameFile> {
    let value: Value = toml::from_str(source).with_context(|| toml_config_help(label))?;
    validate_known_toml_tags(&value, label)?;
    let file: super::toml_schema::GameTomlFile =
        value.try_into().with_context(|| toml_config_help(label))?;
    file.into_authoring(label)
}

fn toml_config_help(label: &str) -> String {
    format!(
        "game config '{label}' could not be read as game config. Use controls.preset = \"top-down\". Use rules.enabled = [\"top-down-controls\", \"show-score\"]."
    )
}

fn validate_known_toml_tags(value: &Value, label: &str) -> Result<()> {
    validate_tagged_array(
        value,
        label,
        "prefab",
        "kind",
        "prefab",
        "prefab kinds",
        &PREFAB_KINDS,
    )?;
    validate_tagged_array(value, label, "map", "kind", "map", "map kinds", &MAP_KINDS)?;
    validate_tagged_array(
        value,
        label,
        "action",
        "kind",
        "action",
        "action kinds",
        &ACTION_KINDS,
    )?;
    validate_tagged_array(
        value,
        label,
        "custom_rule",
        "kind",
        "custom rule",
        "custom rule kinds",
        &CUSTOM_RULE_KINDS,
    )?;
    validate_nested_effects(value, label, "rule", "then")?;
    validate_nested_effects(value, label, "custom_rule", "when_zero")?;
    Ok(())
}

fn validate_tagged_array(
    value: &Value,
    label: &str,
    array_key: &str,
    tag_key: &str,
    item_name: &str,
    known_label: &str,
    known: &[&str],
) -> Result<()> {
    let Some(entries) = value.get(array_key).and_then(Value::as_array) else {
        return Ok(());
    };

    for entry in entries {
        let Some(table) = entry.as_table() else {
            continue;
        };
        let Some(tag) = table.get(tag_key).and_then(Value::as_str) else {
            bail!(
                "game config '{label}' has a {item_name} entry without {tag_key} = \"...\". Known {known_label}: {}.",
                known.join(", ")
            );
        };
        if known.contains(&tag) {
            continue;
        }
        return Err(unknown_tag_error(label, item_name, known_label, tag, known));
    }
    Ok(())
}

fn validate_nested_effects(
    value: &Value,
    label: &str,
    array_key: &str,
    effects_key: &str,
) -> Result<()> {
    let Some(entries) = value.get(array_key).and_then(Value::as_array) else {
        return Ok(());
    };

    for entry in entries {
        let Some(table) = entry.as_table() else {
            continue;
        };
        let Some(effects) = table.get(effects_key).and_then(Value::as_array) else {
            continue;
        };
        for effect in effects {
            let Some(effect_table) = effect.as_table() else {
                continue;
            };
            let Some(action) = effect_table.get("action").and_then(Value::as_str) else {
                bail!(
                    "game config '{label}' has a rule effect without action = \"...\". Known rule effect actions: {}.",
                    EFFECT_ACTIONS.join(", ")
                );
            };
            if EFFECT_ACTIONS.contains(&action) {
                continue;
            }
            return Err(unknown_tag_error(
                label,
                "rule effect action",
                "rule effect actions",
                action,
                &EFFECT_ACTIONS,
            ));
        }
    }
    Ok(())
}

fn unknown_tag_error(
    label: &str,
    item_name: &str,
    known_label: &str,
    value: &str,
    known: &[&str],
) -> anyhow::Error {
    let suggestion = closest_name(value, known.iter().copied())
        .map(|candidate| format!(" Did you mean \"{candidate}\"?"))
        .unwrap_or_default();
    anyhow!(
        "game config '{label}' has unknown {item_name} \"{value}\". Known {known_label}: {}.{suggestion}",
        known.join(", ")
    )
}

const PREFAB_KINDS: [&str; 8] = [
    "player",
    "enemy",
    "pickup",
    "door",
    "projectile",
    "spawner",
    "trigger",
    "checkpoint",
];

const MAP_KINDS: [&str; 3] = ["text", "tiled", "ldtk"];
const ACTION_KINDS: [&str; 1] = ["player-shoots"];
const CUSTOM_RULE_KINDS: [&str; 1] = ["countdown"];
const EFFECT_ACTIONS: [&str; 17] = [
    "add-score",
    "set-score",
    "damage-tagged",
    "damage-player",
    "despawn-self",
    "play-sound",
    "play-music",
    "stop-music",
    "spawn-prefab",
    "spawn-near-player",
    "change-scene",
    "change-map",
    "restart-current-map",
    "show-ui-text",
    "heal-player",
    "set-data",
    "despawn-tagged",
];

#[cfg(test)]
mod tests {
    use super::parse_toml_authoring_source;

    #[test]
    fn parses_minimal_toml_authoring_file() {
        let source = r#"
version = 2

[game]
title = "Coin Collector"
start_map = "level-1"

[assets]
textures = ["player", "slime", "coin", "floor", "wall"]
sounds = ["coin"]

[controls]
preset = "top-down"

[[prefab]]
kind = "player"
name = "player"
sprite = "player"

[[prefab]]
kind = "enemy"
name = "slime"
sprite = "slime"
chase_player = true

[[prefab]]
kind = "pickup"
name = "coin"
sprite = "coin"
sound = "coin"

[[map]]
kind = "text"
name = "level-1"
file = "assets/maps/level-1.txt"
floor = "floor"
wall = "wall"
start = true

[map.legend]
P = "player"
E = "slime"
C = "coin"

[rules]
enabled = ["top-down-controls", "player-collects-pickups", "show-score"]
"#;

        let file = parse_toml_authoring_source(source, "game.toml").unwrap();
        assert_eq!(file.version, 1);
        assert_eq!(file.prefabs.len(), 3);
        assert_eq!(file.maps.len(), 1);
        assert_eq!(file.rules.len(), 3);
    }

    #[test]
    fn unknown_prefab_kind_uses_config_language_and_suggestion() {
        let error = parse_toml_authoring_source(
            r#"
version = 2

[[prefab]]
kind = "plaer"
name = "player"
sprite = "player"
"#,
            "game.toml",
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains("unknown prefab \"plaer\""));
        assert!(error.contains("Known prefab kinds: player, enemy, pickup"));
        assert!(error.contains("Did you mean \"player\"?"));
        assert!(!error.contains("unknown variant"));
    }

    #[test]
    fn invalid_toml_uses_primary_config_help() {
        let error = parse_toml_authoring_source("version = [", "game.toml")
            .unwrap_err()
            .to_string();

        assert!(error.contains("could not be read as game config"));
        assert!(error.contains("controls.preset = \"top-down\""));
        assert!(error.contains("rules.enabled = [\"top-down-controls\", \"show-score\"]"));
        assert!(!error.contains("RON"));
    }
}
