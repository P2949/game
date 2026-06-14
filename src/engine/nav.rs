use std::cmp::Ordering;
use std::collections::BinaryHeap;

use glam::Vec2;

use crate::engine::tilemap::TileMap;

const SQRT2: f32 = std::f32::consts::SQRT_2;

pub struct NavGrid {
    width: i32,
    height: i32,
    walkable: Vec<bool>,
    tile_size: f32,
}

impl NavGrid {
    pub fn from_tilemap(map: &TileMap) -> Self {
        let width = map.width() as i32;
        let height = map.height() as i32;
        let mut walkable = vec![false; (width * height) as usize];
        for row in 0..height {
            for col in 0..width {
                walkable[(row * width + col) as usize] = !map.is_wall(col, row);
            }
        }

        Self {
            width,
            height,
            walkable,
            tile_size: map.tile_size(),
        }
    }

    fn walkable(&self, x: i32, y: i32) -> bool {
        x >= 0
            && y >= 0
            && x < self.width
            && y < self.height
            && self.walkable[(y * self.width + x) as usize]
    }

    fn to_cell(&self, point: Vec2) -> (i32, i32) {
        (
            (point.x / self.tile_size).floor() as i32,
            (point.y / self.tile_size).floor() as i32,
        )
    }

    fn center(&self, x: i32, y: i32) -> Vec2 {
        Vec2::new(
            (x as f32 + 0.5) * self.tile_size,
            (y as f32 + 0.5) * self.tile_size,
        )
    }

    pub fn find_path(&self, from: Vec2, to: Vec2) -> Option<Vec<Vec2>> {
        let start = self.to_cell(from);
        let goal = self.to_cell(to);
        if !self.walkable(start.0, start.1) || !self.walkable(goal.0, goal.1) {
            return None;
        }
        if start == goal {
            return Some(vec![self.center(goal.0, goal.1)]);
        }

        let idx = |x: i32, y: i32| (y * self.width + x) as usize;
        let len = (self.width * self.height) as usize;
        let mut g = vec![f32::INFINITY; len];
        let mut came: Vec<Option<(i32, i32)>> = vec![None; len];
        let mut closed = vec![false; len];
        let mut open = BinaryHeap::new();

        g[idx(start.0, start.1)] = 0.0;
        open.push(Node {
            f: octile(start, goal),
            pos: start,
        });

        const NEIGHBORS: [(i32, i32, f32); 8] = [
            (1, 0, 1.0),
            (-1, 0, 1.0),
            (0, 1, 1.0),
            (0, -1, 1.0),
            (1, 1, SQRT2),
            (1, -1, SQRT2),
            (-1, 1, SQRT2),
            (-1, -1, SQRT2),
        ];

        while let Some(Node { pos, .. }) = open.pop() {
            let here = idx(pos.0, pos.1);
            if closed[here] {
                continue;
            }
            closed[here] = true;
            if pos == goal {
                return Some(self.reconstruct(&came, goal));
            }

            let current_g = g[here];
            for &(dx, dy, cost) in &NEIGHBORS {
                let nx = pos.0 + dx;
                let ny = pos.1 + dy;
                if !self.walkable(nx, ny) {
                    continue;
                }
                if dx != 0
                    && dy != 0
                    && (!self.walkable(pos.0 + dx, pos.1) || !self.walkable(pos.0, pos.1 + dy))
                {
                    continue;
                }

                let next = idx(nx, ny);
                if closed[next] {
                    continue;
                }
                let tentative = current_g + cost;
                if tentative < g[next] {
                    g[next] = tentative;
                    came[next] = Some(pos);
                    open.push(Node {
                        f: tentative + octile((nx, ny), goal),
                        pos: (nx, ny),
                    });
                }
            }
        }

        None
    }

    fn reconstruct(&self, came: &[Option<(i32, i32)>], goal: (i32, i32)) -> Vec<Vec2> {
        let idx = |x: i32, y: i32| (y * self.width + x) as usize;
        let mut cells = vec![goal];
        let mut cur = goal;
        while let Some(prev) = came[idx(cur.0, cur.1)] {
            cells.push(prev);
            cur = prev;
        }
        cells.reverse();
        cells
            .iter()
            .skip(1)
            .map(|&(x, y)| self.center(x, y))
            .collect()
    }
}

fn octile(a: (i32, i32), b: (i32, i32)) -> f32 {
    let dx = (a.0 - b.0).abs() as f32;
    let dy = (a.1 - b.1).abs() as f32;
    (dx + dy) + (SQRT2 - 2.0) * dx.min(dy)
}

struct Node {
    f: f32,
    pos: (i32, i32),
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.f == other.f
    }
}

impl Eq for Node {}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f.partial_cmp(&self.f).unwrap_or(Ordering::Equal)
    }
}

#[cfg(test)]
mod tests {
    use super::NavGrid;
    use crate::engine::tilemap::TileMap;

    #[test]
    fn straight_path_skips_start_cell() {
        let map = TileMap::from_rows(&["....."], 10.0);
        let nav = NavGrid::from_tilemap(&map);

        let path = nav
            .find_path(glam::vec2(5.0, 5.0), glam::vec2(45.0, 5.0))
            .unwrap();

        assert_eq!(path.first().copied(), Some(glam::vec2(15.0, 5.0)));
        assert_eq!(path.last().copied(), Some(glam::vec2(45.0, 5.0)));
    }

    #[test]
    fn routes_around_wall_when_gap_exists() {
        let map = TileMap::from_rows(&[".....", ".###.", "....."], 10.0);
        let nav = NavGrid::from_tilemap(&map);

        let path = nav
            .find_path(glam::vec2(5.0, 5.0), glam::vec2(45.0, 25.0))
            .unwrap();

        assert_eq!(path.last().copied(), Some(glam::vec2(45.0, 25.0)));
        assert!(
            path.iter()
                .all(|point| point.y != 15.0 || point.x == 5.0 || point.x == 45.0)
        );
    }

    #[test]
    fn returns_none_for_blocked_goal() {
        let map = TileMap::from_rows(&[".#."], 10.0);
        let nav = NavGrid::from_tilemap(&map);

        assert!(
            nav.find_path(glam::vec2(5.0, 5.0), glam::vec2(15.0, 5.0))
                .is_none()
        );
    }
}
