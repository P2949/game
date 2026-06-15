#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeleeAttack {
    pub range: f32,
    pub damage: i32,
    pub cooldown: f32,
    pub timer: f32,
}

impl MeleeAttack {
    pub fn new(range: f32, damage: i32) -> Self {
        Self {
            range,
            damage,
            cooldown: 0.0,
            timer: 0.0,
        }
    }

    pub fn cooldown(mut self, cooldown: f32) -> Self {
        self.cooldown = cooldown.max(0.0);
        self
    }
}
