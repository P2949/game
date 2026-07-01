use super::*;

pub(super) fn default_controls() -> BeginnerControlsFile {
    BeginnerControlsFile::Structured(BeginnerControlsKind::TopDown)
}

pub(super) const fn default_beginner_game_version() -> u32 {
    1
}

pub(super) const fn default_player_speed() -> f32 {
    130.0
}

pub(super) const fn default_player_health() -> i32 {
    100
}

pub(super) const fn default_enemy_speed() -> f32 {
    80.0
}

pub(super) const fn default_enemy_health() -> i32 {
    30
}

pub(super) const fn default_pickup_score() -> i32 {
    1
}

pub(super) const fn default_despawn_on_collect() -> bool {
    true
}

pub(super) const fn default_projectile_damage() -> i32 {
    1
}

pub(super) const fn default_projectile_speed() -> f32 {
    300.0
}

pub(super) const fn default_projectile_lifetime() -> f32 {
    1.0
}

pub(super) const fn default_spawn_every() -> f32 {
    1.0
}

pub(super) const fn default_area_size() -> (f32, f32) {
    (32.0, 32.0)
}

pub(super) const fn default_tile_size() -> f32 {
    32.0
}

pub(super) const fn default_music_volume() -> f32 {
    1.0
}

pub(super) const fn default_shoot_cooldown() -> f32 {
    0.2
}

pub(super) const fn default_true() -> bool {
    true
}
