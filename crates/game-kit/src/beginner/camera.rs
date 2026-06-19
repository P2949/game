//! Beginner camera effects.

use glam::{Vec2, vec2};

use crate::context::GameCtx;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraShake {
    pub trauma: f32,
    pub seconds_left: f32,
    phase: f32,
    last_offset: Vec2,
}

impl CameraShake {
    pub fn new(seconds: f32, trauma: f32) -> Self {
        Self {
            trauma: trauma.max(0.0),
            seconds_left: seconds.max(0.0),
            phase: 0.0,
            last_offset: Vec2::ZERO,
        }
    }
}

impl Default for CameraShake {
    fn default() -> Self {
        Self::new(0.0, 0.0)
    }
}

impl<'a, 'w> GameCtx<'a, 'w> {
    pub fn shake_camera(&mut self, seconds: f32) {
        self.shake_camera_with_trauma(seconds, 1.0);
    }

    pub fn shake_camera_with_trauma(&mut self, seconds: f32, trauma: f32) {
        let shake = self.resource_or_insert_with(CameraShake::default);
        shake.seconds_left = shake.seconds_left.max(seconds.max(0.0));
        shake.trauma = shake.trauma.max(trauma.max(0.0));
    }

    pub fn update_camera_shake(&mut self, dt: f32) {
        let Some(mut shake) = self.resource::<CameraShake>().copied() else {
            return;
        };

        let base_center = self.camera().center() - shake.last_offset;
        if shake.seconds_left <= 0.0 || shake.trauma <= 0.0 {
            if shake.last_offset != Vec2::ZERO {
                self.camera_mut().set_center(base_center);
            }
            shake.last_offset = Vec2::ZERO;
            shake.seconds_left = 0.0;
            shake.trauma = 0.0;
            self.insert_resource(shake);
            return;
        }

        shake.seconds_left = (shake.seconds_left - dt.max(0.0)).max(0.0);
        shake.phase += dt.max(0.0) * 60.0;

        let falloff = shake.seconds_left.min(1.0);
        let amplitude = 4.0 * shake.trauma * falloff;
        let offset = vec2((shake.phase * 1.37).sin(), (shake.phase * 1.91).sin()) * amplitude;
        self.camera_mut().set_center(base_center + offset);
        shake.last_offset = offset;

        if shake.seconds_left == 0.0 {
            shake.trauma = 0.0;
        }
        self.insert_resource(shake);
    }
}
