use glam::Vec2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Floor,
    Wall,
}

#[derive(Clone, Debug)]
pub struct TileMap {
    width: usize,
    height: usize,
    tiles: Vec<Tile>,
    tile_size: f32,
}

impl TileMap {
    pub fn from_rows(rows: &[&str], tile_size: f32) -> Self {
        Self::try_from_rows(rows, tile_size).unwrap_or_else(|_| {
            let sanitized = rows
                .iter()
                .map(|row| {
                    row.chars()
                        .map(|ch| if ch == '#' { '#' } else { '.' })
                        .collect::<String>()
                })
                .collect::<Vec<_>>();
            let sanitized = sanitized.iter().map(String::as_str).collect::<Vec<_>>();
            Self::parse_rows(&sanitized, tile_size)
        })
    }

    pub fn try_from_rows(rows: &[&str], tile_size: f32) -> anyhow::Result<Self> {
        for (row, line) in rows.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                if !matches!(ch, '.' | '#') {
                    anyhow::bail!("invalid tile character '{ch}' at row {row}, col {col}");
                }
            }
        }

        Ok(Self::parse_rows(rows, tile_size))
    }

    fn parse_rows(rows: &[&str], tile_size: f32) -> Self {
        let height = rows.len();
        let width = rows
            .iter()
            .map(|row| row.chars().count())
            .max()
            .unwrap_or(0);
        let mut tiles = vec![Tile::Floor; width * height];

        for (row, line) in rows.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                if ch == '#' {
                    tiles[row * width + col] = Tile::Wall;
                }
            }
        }

        Self {
            width,
            height,
            tiles,
            tile_size,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn tile_size(&self) -> f32 {
        self.tile_size
    }

    pub fn tile(&self, col: usize, row: usize) -> Tile {
        if col >= self.width || row >= self.height {
            return Tile::Wall;
        }
        self.tiles[row * self.width + col]
    }

    pub fn is_wall(&self, col: i32, row: i32) -> bool {
        if col < 0 || row < 0 || col as usize >= self.width || row as usize >= self.height {
            return true;
        }
        self.tiles[row as usize * self.width + col as usize] == Tile::Wall
    }

    pub fn cell_center(&self, col: usize, row: usize) -> Vec2 {
        Vec2::new(
            (col as f32 + 0.5) * self.tile_size,
            (row as f32 + 0.5) * self.tile_size,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Tile, TileMap};

    #[test]
    fn parses_walls_and_floors_without_owning_spawns() {
        let map = TileMap::from_rows(&["###", "#P.", "###"], 32.0);

        assert_eq!(map.width(), 3);
        assert_eq!(map.height(), 3);
        assert_eq!(map.tile(0, 0), Tile::Wall);
        assert_eq!(map.tile(1, 1), Tile::Floor);
        assert_eq!(map.cell_center(1, 1), glam::vec2(48.0, 48.0));
    }

    #[test]
    fn out_of_bounds_is_wall() {
        let map = TileMap::from_rows(&["."], 16.0);

        assert!(map.is_wall(-1, 0));
        assert!(map.is_wall(0, -1));
        assert!(map.is_wall(1, 0));
        assert!(map.is_wall(0, 1));
        assert!(!map.is_wall(0, 0));
    }

    #[test]
    fn try_from_rows_rejects_unknown_tile_characters() {
        let err = TileMap::try_from_rows(&[".P"], 16.0).unwrap_err();

        assert!(err.to_string().contains("invalid tile character"));
    }
}
