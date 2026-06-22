//! Minimal Tiled TMX importer for the beginner map workflow.
//!
//! The supported contract is intentionally small and easy to author: an
//! orthogonal XML TMX map with square tiles, one CSV tile layer named
//! `Collision` (case-insensitive; gid `0` is floor and every nonzero gid is a
//! wall), and ordinary object groups. Each object uses its `class`, `type`, or
//! `name` as its identifier. `game-kit` maps those identifiers to prefabs.

use std::path::Path;

use anyhow::{Context, Result, anyhow};

use crate::{MapCell, cell};

#[derive(Clone, Debug)]
pub struct ImportedTiledMap {
    pub tile_size: f32,
    pub collision_rows: Vec<String>,
    pub objects: Vec<ImportedTiledObject>,
}

#[derive(Clone, Debug)]
pub struct ImportedTiledObject {
    pub identifier: String,
    pub cell: MapCell,
}

/// Parses the supported XML TMX subset into collision rows and object cells.
pub fn load_tiled_map(text: &str) -> Result<ImportedTiledMap> {
    let (map_tag, _) =
        opening_tag(text, "map", 0).ok_or_else(|| anyhow!("TMX file has no <map> root element"))?;
    let width = required_usize(&map_tag, "width", "TMX map")?;
    let height = required_usize(&map_tag, "height", "TMX map")?;
    let tile_width = required_f32(&map_tag, "tilewidth", "TMX map")?;
    let tile_height = required_f32(&map_tag, "tileheight", "TMX map")?;
    if !tile_width.is_finite()
        || !tile_height.is_finite()
        || tile_width <= 0.0
        || tile_height <= 0.0
    {
        anyhow::bail!("TMX map has invalid tile dimensions {tile_width} by {tile_height}");
    }
    if (tile_width - tile_height).abs() > f32::EPSILON {
        anyhow::bail!(
            "TMX map uses {tile_width} by {tile_height} tiles; the beginner importer currently needs square tiles"
        );
    }
    if width == 0 || height == 0 {
        anyhow::bail!("TMX map dimensions must be non-zero");
    }

    let collision_rows = load_collision_layer(text, width, height)?;
    let objects = load_objects(text, width, height, tile_width, tile_height)?;
    Ok(ImportedTiledMap {
        tile_size: tile_width,
        collision_rows,
        objects,
    })
}

/// Reads and parses one TMX map file.
pub fn load_tiled_map_file(path: impl AsRef<Path>) -> Result<ImportedTiledMap> {
    let path = path.as_ref();
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read Tiled TMX map '{}'", path.display()))?;
    load_tiled_map(&text)
        .with_context(|| format!("failed to parse Tiled TMX map '{}'", path.display()))
}

fn load_collision_layer(text: &str, width: usize, height: usize) -> Result<Vec<String>> {
    let mut cursor = 0;
    while let Some((layer_tag, layer_open_end)) = opening_tag(text, "layer", cursor) {
        let layer_start = layer_open_end;
        let layer_close = text[layer_start..]
            .find("</layer>")
            .map(|offset| layer_start + offset)
            .ok_or_else(|| anyhow!("TMX layer is missing its </layer> closing tag"))?;
        cursor = layer_close + "</layer>".len();
        let name = attribute(&layer_tag, "name").unwrap_or_default();
        if !name.eq_ignore_ascii_case("collision") {
            continue;
        }

        if let Some(layer_width) = attribute(&layer_tag, "width") {
            let layer_width = parse_usize(&layer_width, "Collision layer width")?;
            if layer_width != width {
                anyhow::bail!(
                    "TMX Collision layer width is {layer_width}, but the map width is {width}"
                );
            }
        }
        if let Some(layer_height) = attribute(&layer_tag, "height") {
            let layer_height = parse_usize(&layer_height, "Collision layer height")?;
            if layer_height != height {
                anyhow::bail!(
                    "TMX Collision layer height is {layer_height}, but the map height is {height}"
                );
            }
        }

        let body = &text[layer_start..layer_close];
        let (data_tag, data_open_end) = opening_tag(body, "data", 0).ok_or_else(|| {
            anyhow!("TMX Collision layer needs a <data encoding=\"csv\"> element")
        })?;
        if attribute(&data_tag, "encoding").as_deref() != Some("csv") {
            anyhow::bail!(
                "TMX Collision layer needs CSV data. In Tiled choose CSV encoding for its <data> element."
            );
        }
        if attribute(&data_tag, "compression").is_some() {
            anyhow::bail!(
                "TMX Collision layer uses compressed CSV data; remove compression for the beginner importer"
            );
        }
        let data_end = body[data_open_end..]
            .find("</data>")
            .map(|offset| data_open_end + offset)
            .ok_or_else(|| anyhow!("TMX Collision <data> element is missing </data>"))?;
        let values = body[data_open_end..data_end]
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| {
                value.parse::<u32>().map_err(|error| {
                    anyhow!("TMX Collision CSV value '{value}' is not a tile id: {error}")
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let expected = width
            .checked_mul(height)
            .ok_or_else(|| anyhow!("TMX map dimensions are too large"))?;
        if values.len() != expected {
            anyhow::bail!(
                "TMX Collision layer has {} CSV cells; expected {expected} ({width} by {height})",
                values.len()
            );
        }
        return Ok(values
            .chunks_exact(width)
            .map(|row| {
                row.iter()
                    .map(|gid| if *gid == 0 { '.' } else { '#' })
                    .collect()
            })
            .collect());
    }
    anyhow::bail!(
        "TMX map has no layer named 'Collision'. Add a tile layer named Collision with CSV data (0=floor, nonzero=wall)."
    )
}

fn load_objects(
    text: &str,
    width: usize,
    height: usize,
    tile_width: f32,
    tile_height: f32,
) -> Result<Vec<ImportedTiledObject>> {
    let mut groups_cursor = 0;
    let mut objects = Vec::new();
    while let Some((_, group_open_end)) = opening_tag(text, "objectgroup", groups_cursor) {
        let group_close = text[group_open_end..]
            .find("</objectgroup>")
            .map(|offset| group_open_end + offset)
            .ok_or_else(|| anyhow!("TMX object group is missing </objectgroup>"))?;
        let body = &text[group_open_end..group_close];
        groups_cursor = group_close + "</objectgroup>".len();

        let mut object_cursor = 0;
        while let Some((object_tag, object_open_end)) = opening_tag(body, "object", object_cursor) {
            object_cursor = object_open_end;
            let identifier = attribute(&object_tag, "class")
                .filter(|value| !value.is_empty())
                .or_else(|| attribute(&object_tag, "type").filter(|value| !value.is_empty()))
                .or_else(|| attribute(&object_tag, "name").filter(|value| !value.is_empty()))
                .ok_or_else(|| {
                    anyhow!(
                        "TMX object needs a non-empty class, type, or name so it can map to a prefab"
                    )
                })?;
            let x = required_f32(&object_tag, "x", &format!("TMX object '{identifier}'"))?;
            let y = required_f32(&object_tag, "y", &format!("TMX object '{identifier}'"))?;
            if !x.is_finite() || !y.is_finite() || x < 0.0 || y < 0.0 {
                anyhow::bail!(
                    "TMX object '{identifier}' has invalid position ({x}, {y}); objects must be inside the map"
                );
            }
            let col = (x / tile_width).floor() as usize;
            let row = (y / tile_height).floor() as usize;
            if col >= width || row >= height {
                anyhow::bail!(
                    "TMX object '{identifier}' is outside the map at cell ({col}, {row}); map size is {width} by {height}"
                );
            }
            objects.push(ImportedTiledObject {
                identifier,
                cell: cell(col, row),
            });
        }
    }
    Ok(objects)
}

fn opening_tag(source: &str, name: &str, from: usize) -> Option<(String, usize)> {
    let needle = format!("<{name}");
    let mut cursor = from;
    while let Some(found) = source[cursor..].find(&needle) {
        let start = cursor + found;
        let after_name = start + needle.len();
        let boundary = source.as_bytes().get(after_name).copied()?;
        if !(boundary.is_ascii_whitespace() || matches!(boundary, b'>' | b'/')) {
            cursor = after_name;
            continue;
        }
        let close = source[after_name..].find('>')? + after_name;
        return Some((source[start..=close].to_owned(), close + 1));
    }
    None
}

fn attribute(tag: &str, wanted: &str) -> Option<String> {
    let mut cursor = 1;
    while cursor < tag.len() {
        while tag
            .as_bytes()
            .get(cursor)
            .is_some_and(u8::is_ascii_whitespace)
        {
            cursor += 1;
        }
        if tag
            .as_bytes()
            .get(cursor)
            .is_none_or(|byte| *byte == b'>' || *byte == b'/')
        {
            break;
        }
        let key_start = cursor;
        while tag
            .as_bytes()
            .get(cursor)
            .is_some_and(|byte| !byte.is_ascii_whitespace() && *byte != b'=' && *byte != b'>')
        {
            cursor += 1;
        }
        let key = tag.get(key_start..cursor)?;
        while tag
            .as_bytes()
            .get(cursor)
            .is_some_and(u8::is_ascii_whitespace)
        {
            cursor += 1;
        }
        if tag.as_bytes().get(cursor) != Some(&b'=') {
            continue;
        }
        cursor += 1;
        while tag
            .as_bytes()
            .get(cursor)
            .is_some_and(u8::is_ascii_whitespace)
        {
            cursor += 1;
        }
        let quote = *tag.as_bytes().get(cursor)?;
        if quote != b'\'' && quote != b'\"' {
            return None;
        }
        cursor += 1;
        let value_start = cursor;
        while tag
            .as_bytes()
            .get(cursor)
            .is_some_and(|byte| *byte != quote)
        {
            cursor += 1;
        }
        let value = tag.get(value_start..cursor)?.to_owned();
        cursor += 1;
        if key == wanted {
            return Some(value);
        }
    }
    None
}

fn required_usize(tag: &str, name: &str, context: &str) -> Result<usize> {
    let value = attribute(tag, name)
        .ok_or_else(|| anyhow!("{context} is missing required '{name}' attribute"))?;
    parse_usize(&value, &format!("{context} {name}"))
}

fn parse_usize(value: &str, context: &str) -> Result<usize> {
    value
        .parse()
        .map_err(|error| anyhow!("{context} '{value}' is not a non-negative integer: {error}"))
}

fn required_f32(tag: &str, name: &str, context: &str) -> Result<f32> {
    let value = attribute(tag, name)
        .ok_or_else(|| anyhow!("{context} is missing required '{name}' attribute"))?;
    value
        .parse()
        .map_err(|error| anyhow!("{context} {name} '{value}' is not a number: {error}"))
}

#[cfg(test)]
mod tests {
    use super::load_tiled_map;

    const MAP: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<map width="3" height="2" tilewidth="16" tileheight="16">
  <layer name="Collision" width="3" height="2">
    <data encoding="csv">
      1,0,1,
      1,0,1
    </data>
  </layer>
  <objectgroup name="Actors">
    <object id="1" type="Player" x="16" y="0"/>
    <object id="2" class="Slime" x="16" y="16"/>
  </objectgroup>
</map>"#;

    #[test]
    fn imports_csv_collision_and_named_objects() {
        let map = load_tiled_map(MAP).unwrap();

        assert_eq!(map.tile_size, 16.0);
        assert_eq!(map.collision_rows, ["#.#", "#.#"]);
        assert_eq!(map.objects.len(), 2);
        assert_eq!(map.objects[0].identifier, "Player");
        assert_eq!(map.objects[0].cell.col(), 1);
        assert_eq!(map.objects[1].identifier, "Slime");
        assert_eq!(map.objects[1].cell.row(), 1);
    }

    #[test]
    fn names_the_missing_collision_layer_and_csv_requirement() {
        let no_collision = MAP.replace("Collision", "Tiles");
        let error = load_tiled_map(&no_collision).unwrap_err().to_string();
        assert!(error.contains("no layer named 'Collision'"));

        let non_csv = MAP.replace("encoding=\"csv\"", "encoding=\"base64\"");
        let error = load_tiled_map(&non_csv).unwrap_err().to_string();
        assert!(error.contains("needs CSV data"));
    }
}
