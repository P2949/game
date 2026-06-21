//! Data-driven numeric tuning for beginner prefabs and rules.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use game_core::world::World;

use crate::map::beginner_asset_path;

/// A named set of beginner-authored numeric values loaded from a RON file.
///
/// Tuning intentionally stays small and string-keyed. It is for values such
/// as health, speed, damage, and cooldowns—not a replacement for game data
/// structures in a content crate.
#[derive(Clone, Debug, Default)]
pub struct TuningFile {
    path: PathBuf,
    values: HashMap<String, f32>,
}

/// A floating-point prefab setting that can be supplied either as a literal or
/// as a key resolved from the current [`TuningFile`] resource at spawn time.
#[derive(Clone, Debug)]
pub struct TunedF32 {
    key: String,
    initial: f32,
}

impl TunedF32 {
    pub fn initial(&self) -> f32 {
        self.initial
    }

    pub(crate) fn resolve(&self, world: &World) -> f32 {
        world
            .get_resource::<TuningFile>()
            .and_then(|tuning| tuning.values.get(&self.key).copied())
            .unwrap_or(self.initial)
    }
}

impl From<f32> for TunedF32 {
    fn from(value: f32) -> Self {
        Self {
            key: String::new(),
            initial: value,
        }
    }
}

/// An integer prefab setting resolved from a named tuning value at spawn time.
#[derive(Clone, Debug)]
pub struct TunedI32 {
    key: String,
    initial: i32,
}

impl TunedI32 {
    pub fn initial(&self) -> i32 {
        self.initial
    }

    pub(crate) fn resolve(&self, world: &World) -> i32 {
        world
            .get_resource::<TuningFile>()
            .and_then(|tuning| tuning.values.get(&self.key).copied())
            .and_then(f32_to_i32)
            .unwrap_or(self.initial)
    }
}

impl From<i32> for TunedI32 {
    fn from(value: i32) -> Self {
        Self {
            key: String::new(),
            initial: value,
        }
    }
}

impl TuningFile {
    /// Loads a RON map such as `( "slime.health": 40.0, )`.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let requested = path.as_ref();
        let path = if requested.is_absolute() {
            requested.to_path_buf()
        } else {
            beginner_asset_path(requested.to_string_lossy().as_ref())
        };
        let text = fs::read_to_string(&path)
            .with_context(|| format!("could not read tuning file '{}'", path.display()))?;
        let values = parse_values(&path, &text)?;
        Ok(Self { path, values })
    }

    /// Returns a numeric tuning value, with a helpful error when its key is absent.
    pub fn float(&self, key: &str) -> TunedF32 {
        TunedF32 {
            key: key.to_owned(),
            initial: self.value(key),
        }
    }

    fn value(&self, key: &str) -> f32 {
        self.values.get(key).copied().unwrap_or_else(|| {
            let mut keys = self.values.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            panic!(
                "tuning file '{}' has no value named '{key}'. Available values: {}",
                self.path.display(),
                if keys.is_empty() {
                    "(none)".to_owned()
                } else {
                    keys.join(", ")
                }
            )
        })
    }

    /// Returns an integer tuning value. Fractional values are rejected instead
    /// of silently truncating damage or health.
    pub fn int(&self, key: &str) -> TunedI32 {
        let value = self.value(key);
        let initial = f32_to_i32(value).unwrap_or_else(|| {
            panic!(
                "tuning value '{key}' in '{}' must be a whole number, got {value}",
                self.path.display()
            );
        });
        TunedI32 {
            key: key.to_owned(),
            initial,
        }
    }

    /// The resolved file path, used by development-time reload support.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Re-reads the same source file, retaining its stable resource identity.
    pub fn reload(&mut self) -> Result<()> {
        let replacement = Self::from_file(&self.path)?;
        self.values = replacement.values;
        Ok(())
    }
}

fn f32_to_i32(value: f32) -> Option<i32> {
    (value.is_finite() && value.fract() == 0.0)
        .then_some(value as i64)
        .and_then(|value| i32::try_from(value).ok())
}

fn parse_values(path: &Path, text: &str) -> Result<HashMap<String, f32>> {
    // RON normally represents a map with `{ ... }`, but the concise
    // parenthesized form is easier to read in beginner documentation. Accept
    // both forms while still deserializing a concrete string-to-float map.
    let trimmed = text.trim();
    let normalized;
    let source = if let Some(inner) = trimmed
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
    {
        normalized = format!("{{{inner}}}");
        &normalized
    } else {
        trimmed
    };
    ron::de::from_str(source).with_context(|| {
        format!(
            "could not parse tuning file '{}' as a RON map",
            path.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::TuningFile;

    #[test]
    fn tuning_file_reads_named_floats_and_integer_values() {
        let path = std::env::temp_dir().join(format!(
            "game-kit-tuning-{}-{}.ron",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::write(&path, "( \"slime.health\": 40.0, \"slime.speed\": 80.5 )").unwrap();

        let tuning = TuningFile::from_file(&path).unwrap();
        assert_eq!(tuning.int("slime.health").initial(), 40);
        assert_eq!(tuning.float("slime.speed").initial(), 80.5);

        fs::remove_file(path).unwrap();
    }
}
