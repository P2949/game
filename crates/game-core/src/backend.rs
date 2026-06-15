use crate::app::RenderFrame;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextureHandle(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SoundHandle(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FontHandle(pub u32);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextureLoadRequest {
    pub path: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SoundLoadRequest {
    pub path: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontLoadRequest {
    pub path: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderOutcome {
    Presented,
    Skipped,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AudioCommand {
    Play {
        sound: SoundHandle,
        volume: f32,
        looping: bool,
    },
    StopMusic,
    PlayMusic {
        sound: SoundHandle,
        volume: f32,
    },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlatformEvents {
    pub should_quit: bool,
}

pub trait RenderBackend {
    fn load_texture(&mut self, request: TextureLoadRequest) -> anyhow::Result<TextureHandle>;
    fn render(&mut self, frame: RenderFrame) -> anyhow::Result<RenderOutcome>;
}

pub trait AudioBackend {
    fn load_sound(&mut self, request: SoundLoadRequest) -> anyhow::Result<SoundHandle>;
    fn submit(&self, command: AudioCommand);
    fn poll_diagnostics(&self);
}

pub trait PlatformBackend {
    fn pump_events(&mut self) -> PlatformEvents;
    fn drawable_size(&self) -> glam::UVec2;
    fn should_quit(&self) -> bool;
}
