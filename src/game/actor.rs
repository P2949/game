use glam::Vec2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

impl Health {
    pub fn new(max: i32) -> Self {
        Self { current: max, max }
    }

    pub fn damage(&mut self, amount: i32) {
        self.current = (self.current - amount).max(0);
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Player {
    pub health: Health,
    pub speed: f32,
    pub attack_range: f32,
    pub attack_damage: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct Enemy {
    pub health: Health,
    pub speed: f32,
    pub aggro_radius: f32,
    pub attack_range: f32,
    pub attack_damage: i32,
    pub attack_cooldown: f32,
    pub path: PathFollow,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PathFollow {
    pub next: Option<Vec2>,
    pub repath_timer: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum Actor {
    Player(Player),
    Enemy(Enemy),
}
