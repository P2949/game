#[derive(Clone, Copy)]
pub struct Audio<'a> {
    system: Option<&'a crate::audio::AudioSystem>,
}

impl<'a> Audio<'a> {
    pub fn new(system: Option<&'a crate::audio::AudioSystem>) -> Self {
        Self { system }
    }

    pub fn hit(&self) {
        if let Some(system) = self.system {
            system.play_blip();
        }
    }
}
