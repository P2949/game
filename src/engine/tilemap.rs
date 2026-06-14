use glam::Vec2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Floor,
    Wall,
}

pub struct TileMap {
    width: usize,
    height: usize,
    tiles: Vec<Tile>,
    tile_size: f32,
    pub spawns: Vec<(char, usize, usize)>,
}

impl TileMap {
    pub fn from_rows(rows: &[&str], tile_size: f32) -> Self {
        let height = rows.len();
        let width = rows
            .iter()
            .map(|row| row.chars().count())
            .max()
            .unwrap_or(0);
        let mut tiles = vec![Tile::Floor; width * height];
        let mut spawns = Vec::new();

        for (row, line) in rows.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                match ch {
                    '#' => tiles[row * width + col] = Tile::Wall,
                    '.' | ' ' => {}
                    marker => spawns.push((marker, col, row)),
                }
            }
        }

        Self {
            width,
            height,
            tiles,
            tile_size,
            spawns,
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
    fn parses_walls_floors_and_spawn_markers() {
        let map = TileMap::from_rows(&["###", "#P.", "###"], 32.0);

        assert_eq!(map.width(), 3);
        assert_eq!(map.height(), 3);
        assert_eq!(map.tile(0, 0), Tile::Wall);
        assert_eq!(map.tile(1, 1), Tile::Floor);
        assert_eq!(map.spawns, vec![('P', 1, 1)]);
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
}
