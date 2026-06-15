#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FactionId {
    Player,
    Enemy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Faction(pub FactionId);

impl Faction {
    pub fn player() -> Self {
        Self(FactionId::Player)
    }

    pub fn enemy() -> Self {
        Self(FactionId::Enemy)
    }
}
