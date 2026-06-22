//! In-memory implementations of the runtime backend traits.
//!
//! These types deliberately perform no SDL, Vulkan, or audio-device work. They
//! make it possible to exercise the real runtime loop in a normal test process
//! and inspect the frames and audio commands it produced.

use std::cell::RefCell;
use std::path::Path;

use game_core::app::RenderFrame;
use game_core::backend::{
    AudioBackend, AudioCommand, PlatformBackend, PlatformEvents, RenderBackend, RenderOutcome,
    SoundHandle, SoundLoadRequest, TextureHandle,
};
use game_core::input::{GamepadAxis, GamepadButton, InputState, Key, MouseButton};

/// A scripted, in-memory window/input source.
#[derive(Debug)]
pub struct HeadlessPlatform {
    input: InputState,
    drawable_size: glam::UVec2,
    should_quit: bool,
    resize_requested: bool,
    pending_events: Vec<InputEvent>,
}

#[derive(Debug, Clone, Copy)]
enum InputEvent {
    Key(Key, bool),
    MouseButton(MouseButton, bool),
    MousePosition(glam::Vec2),
    GamepadButton(GamepadButton, bool),
    GamepadAxis(GamepadAxis, glam::Vec2),
}

impl HeadlessPlatform {
    pub fn new(drawable_size: glam::UVec2) -> Self {
        let mut input = InputState::default();
        input.set_viewport_size(drawable_size.as_vec2());
        Self {
            input,
            drawable_size,
            should_quit: false,
            resize_requested: false,
            pending_events: Vec::new(),
        }
    }

    pub fn queue_key(&mut self, key: Key, down: bool) {
        self.pending_events.push(InputEvent::Key(key, down));
    }

    pub fn queue_mouse_button(&mut self, button: MouseButton, down: bool) {
        self.pending_events
            .push(InputEvent::MouseButton(button, down));
    }

    pub fn queue_mouse_position(&mut self, position: glam::Vec2) {
        self.pending_events
            .push(InputEvent::MousePosition(position));
    }

    pub fn queue_gamepad_button(&mut self, button: GamepadButton, down: bool) {
        self.pending_events
            .push(InputEvent::GamepadButton(button, down));
    }

    pub fn queue_gamepad_axis(&mut self, axis: GamepadAxis, value: glam::Vec2) {
        self.pending_events
            .push(InputEvent::GamepadAxis(axis, value));
    }

    pub fn resize(&mut self, drawable_size: glam::UVec2) {
        if self.drawable_size != drawable_size {
            self.drawable_size = drawable_size;
            self.resize_requested = true;
        }
    }

    pub fn input_state(&self) -> &InputState {
        &self.input
    }
}

impl Default for HeadlessPlatform {
    fn default() -> Self {
        Self::new(glam::uvec2(1280, 720))
    }
}

impl PlatformBackend for HeadlessPlatform {
    fn pump_events(&mut self) -> PlatformEvents {
        self.input.begin_frame();
        for event in self.pending_events.drain(..) {
            match event {
                InputEvent::Key(key, down) => self.input.set_key(key, down),
                InputEvent::MouseButton(button, down) => self.input.set_mouse_button(button, down),
                InputEvent::MousePosition(position) => self.input.set_mouse_position(position),
                InputEvent::GamepadButton(button, down) => {
                    self.input.set_gamepad_button(button, down)
                }
                InputEvent::GamepadAxis(axis, value) => self.input.set_gamepad_axis(axis, value),
            }
        }
        self.input.set_viewport_size(self.drawable_size.as_vec2());
        PlatformEvents {
            should_quit: self.should_quit,
        }
    }

    fn input(&self) -> &InputState {
        &self.input
    }

    fn drawable_size(&self) -> glam::UVec2 {
        self.drawable_size
    }

    fn take_stable_resize_request(&mut self) -> bool {
        std::mem::take(&mut self.resize_requested)
    }

    fn should_quit(&self) -> bool {
        self.should_quit
    }

    fn request_quit(&mut self) {
        self.should_quit = true;
    }
}

/// A renderer that stores each submitted frame for assertions.
#[derive(Default)]
pub struct HeadlessRenderer {
    frames: Vec<RenderFrame>,
    texture_reloads: Vec<Vec<(TextureHandle, String)>>,
    resize_requests: usize,
}

impl HeadlessRenderer {
    pub fn frames(&self) -> &[RenderFrame] {
        &self.frames
    }

    pub fn texture_reloads(&self) -> &[Vec<(TextureHandle, String)>] {
        &self.texture_reloads
    }

    pub fn resize_requests(&self) -> usize {
        self.resize_requests
    }
}

impl RenderBackend for HeadlessRenderer {
    fn reload_textures(&mut self, textures: &[(TextureHandle, String)]) -> anyhow::Result<usize> {
        self.texture_reloads.push(textures.to_vec());
        Ok(textures.len())
    }

    fn request_resize(&mut self) {
        self.resize_requests += 1;
    }

    fn render(
        &mut self,
        _drawable_size: glam::UVec2,
        frame: RenderFrame,
    ) -> anyhow::Result<RenderOutcome> {
        self.frames.push(frame);
        Ok(RenderOutcome::Presented)
    }
}

/// An audio backend that records commands and reload requests.
#[derive(Default)]
pub struct HeadlessAudio {
    commands: RefCell<Vec<AudioCommand>>,
    sound_reloads: Vec<Vec<(SoundHandle, SoundLoadRequest)>>,
}

impl HeadlessAudio {
    pub fn commands(&self) -> Vec<AudioCommand> {
        self.commands.borrow().clone()
    }

    pub fn sound_reloads(&self) -> &[Vec<(SoundHandle, SoundLoadRequest)>] {
        &self.sound_reloads
    }
}

impl AudioBackend for HeadlessAudio {
    fn reload_sounds(
        &mut self,
        _asset_root: &Path,
        sounds: &[(SoundHandle, SoundLoadRequest)],
    ) -> anyhow::Result<usize> {
        self.sound_reloads.push(sounds.to_vec());
        Ok(sounds
            .iter()
            .filter(|(_, request)| !matches!(request, SoundLoadRequest::Generated { .. }))
            .count())
    }

    fn submit(&self, command: AudioCommand) {
        self.commands.borrow_mut().push(command);
    }

    fn poll_diagnostics(&self) {}
}
