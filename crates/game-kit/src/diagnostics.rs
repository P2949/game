//! Beginner-facing diagnostic helpers.

use std::path::Path;

use anyhow::anyhow;

pub(crate) fn unknown_name_error<'a>(
    kind: &str,
    requested: &str,
    known: impl Iterator<Item = &'a str>,
) -> anyhow::Error {
    let known = sorted_known(known);
    let suggestion = did_you_mean(requested, &known, "\n\n");
    let known_label = known_label(kind);
    anyhow!(
        "Unknown {kind} '{requested}'.\n\n{known_label}:\n{}{suggestion}",
        known_bullets(&known)
    )
}

pub(crate) fn unknown_reference_error(
    label: &str,
    owner: &str,
    kind: &str,
    requested: &str,
    known: &[&str],
) -> anyhow::Error {
    let known = sorted_known(known.iter().copied());
    let suggestion = did_you_mean(requested, &known, " ");
    let known_label = known_label(kind);
    anyhow!(
        "beginner game file '{label}' {owner} references unknown {kind} '{requested}'. {known_label}: {}.{suggestion}",
        known_inline(&known)
    )
}

pub(crate) fn missing_file_error(
    kind: &str,
    requested_path: &str,
    looked_path: &Path,
) -> anyhow::Error {
    anyhow!(
        "Missing {kind} file '{requested_path}'.\n\nLooked for '{}'.\n\nCheck the path under assets/, or register a different file path in your asset setup.",
        looked_path.display()
    )
}

pub(crate) fn bad_map_symbol_error(
    map_name: &str,
    symbol: char,
    row: usize,
    col: usize,
    known_symbols: &[char],
) -> anyhow::Error {
    let mut known = known_symbols.to_vec();
    known.sort_unstable();
    known.dedup();
    let known = if known.is_empty() {
        "(none yet)".to_owned()
    } else {
        known
            .iter()
            .map(|symbol| format!("{symbol:?}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    anyhow!(
        "Map '{map_name}' uses symbol {symbol:?} but no legend maps it to a prefab.\n\nAt row {}, col {} add:\n    .legend({symbol:?}, \"some_prefab\")\n\nor replace the symbol with `.` or `#`.\n\nKnown legend symbols: {known}.",
        row + 1,
        col + 1,
    )
}

pub(crate) fn bad_rule_combo_error(rule: &str, missing_dependency: &str) -> anyhow::Error {
    let rule_method = rule_method_name(rule);
    let dependency_method = rule_method_name(missing_dependency);
    anyhow!(
        "Rule `{rule_method}` needs the `{dependency_method}` rule.\n\nAdd `.{dependency_method}()` before `.{rule_method}()`."
    )
}

fn rule_method_name(rule: &str) -> String {
    let mut out = String::new();
    for (index, ch) in rule.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if index > 0 {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

fn sorted_known<'a>(known: impl Iterator<Item = &'a str>) -> Vec<&'a str> {
    let mut known = known.collect::<Vec<_>>();
    known.sort_unstable();
    known.dedup();
    known
}

fn known_bullets(known: &[&str]) -> String {
    if known.is_empty() {
        return "(none registered)".to_owned();
    }
    known
        .iter()
        .map(|known| format!("- {known}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn known_inline(known: &[&str]) -> String {
    if known.is_empty() {
        "(none)".to_owned()
    } else {
        known.join(", ")
    }
}

fn known_label(kind: &str) -> String {
    match kind {
        "music" => "Known music".to_owned(),
        _ => format!("Known {kind}s"),
    }
}

fn did_you_mean(requested: &str, known: &[&str], prefix: &str) -> String {
    closest_name(requested, known.iter().copied())
        .map(|candidate| format!("{prefix}Did you mean '{candidate}'?"))
        .unwrap_or_default()
}

pub(crate) fn closest_name<'a>(
    needle: &str,
    known: impl Iterator<Item = &'a str>,
) -> Option<&'a str> {
    let candidate = known.min_by_key(|key| edit_distance(needle, key))?;
    let distance = edit_distance(needle, candidate);
    let threshold = (needle.chars().count().max(candidate.chars().count()) / 3).max(2);
    (distance <= threshold).then_some(candidate)
}

fn edit_distance(left: &str, right: &str) -> usize {
    let right = right.chars().collect::<Vec<_>>();
    let mut previous = (0..=right.len()).collect::<Vec<_>>();

    for (left_index, left_char) in left.chars().enumerate() {
        let mut current = vec![left_index + 1];
        for (right_index, right_char) in right.iter().enumerate() {
            let replace = previous[right_index] + usize::from(left_char != *right_char);
            let insert = current[right_index] + 1;
            let delete = previous[right_index + 1] + 1;
            current.push(replace.min(insert).min(delete));
        }
        previous = current;
    }

    previous[right.len()]
}

#[cfg(test)]
mod tests {
    use super::{bad_rule_combo_error, unknown_name_error};

    #[test]
    fn unknown_name_lists_known_values_and_suggests_close_matches() {
        let error = unknown_name_error("texture asset", "plaeyr", ["player", "slime"].into_iter())
            .to_string();

        assert!(error.contains("Unknown texture asset 'plaeyr'"));
        assert!(error.contains("Known texture assets:"));
        assert!(error.contains("- player"));
        assert!(error.contains("Did you mean 'player'?"));
    }

    #[test]
    fn rule_combo_error_names_the_missing_builder_call() {
        let error = bad_rule_combo_error("ProjectilesDamageEnemies", "Projectiles").to_string();

        assert!(error.contains("Rule `projectiles_damage_enemies` needs the `projectiles` rule"));
        assert!(error.contains("Add `.projectiles()`"));
        assert!(error.contains("`.projectiles_damage_enemies()`"));
    }
}
