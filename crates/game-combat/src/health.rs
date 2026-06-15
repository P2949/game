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
