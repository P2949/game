use anyhow::Context;
use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::Keycode;
use sdl3::video::Window;
use std::time::Instant;

use crate::input::{
    gamepad_axis_from_sdl, gamepad_button_from_sdl, key_from_sdl, mouse_button_from_sdl,
    normalize_gamepad_axis,
};
use crate::resize::ResizePolicy;
use game_core::input::InputState;

pub struct Platform {
    // Keep the SDL context alive for the lifetime of the window and event pump.
    #[allow(dead_code)]
    pub sdl: sdl3::Sdl,
    pub window: Window,
    pub event_pump: sdl3::EventPump,
    pub should_quit: bool,
    pub input: InputState,
    last_drawable_size: (u32, u32),
    pending_drawable_resize: bool,
    resize_policy: ResizePolicy,
    gamepad_subsystem: sdl3::GamepadSubsystem,
    primary_gamepad_id: Option<u32>,
    // SDL only dispatches gamepad state while this handle is kept open.
    primary_gamepad: Option<sdl3::gamepad::Gamepad>,
}

impl Platform {
    pub fn new(title: &str, width: u32, height: u32) -> anyhow::Result<Self> {
        let sdl = sdl3::init()
            .map_err(anyhow::Error::msg)
            .context("failed to initialize SDL3")?;
        let video = sdl
            .video()
            .map_err(anyhow::Error::msg)
            .context("failed to initialize SDL3 video subsystem")?;

        let mut window = video
            .window(title, width, height)
            .vulkan()
            .resizable()
            .position_centered()
            .build()
            .map_err(anyhow::Error::msg)
            .context("failed to create SDL3 Vulkan window")?;

        if !window.show() {
            anyhow::bail!("failed to show SDL3 window: {}", sdl3::get_error());
        }

        if !window.sync() {
            log::warn!("timed out waiting for the SDL3 window to become visible");
        }

        log::info!(
            "created SDL3 Vulkan window: logical={}x{}, pixels={:?}",
            width,
            height,
            window.size_in_pixels()
        );

        let event_pump = sdl
            .event_pump()
            .map_err(anyhow::Error::msg)
            .context("failed to create SDL3 event pump")?;
        let gamepad_subsystem = sdl
            .gamepad()
            .map_err(anyhow::Error::msg)
            .context("failed to initialize SDL3 gamepad subsystem")?;
        let (primary_gamepad_id, primary_gamepad) = open_first_gamepad(&gamepad_subsystem);

        let last_drawable_size = window.size_in_pixels();

        Ok(Self {
            sdl,
            window,
            event_pump,
            should_quit: false,
            input: InputState::default(),
            last_drawable_size,
            pending_drawable_resize: false,
            resize_policy: ResizePolicy::default(),
            gamepad_subsystem,
            primary_gamepad_id,
            primary_gamepad,
        })
    }

    pub fn pump_events(&mut self) {
        self.input.begin_frame();
        self.set_input_viewport_size();

        let events = self.event_pump.poll_iter().collect::<Vec<_>>();
        for event in events {
            match event {
                Event::Quit { .. } => self.should_quit = true,
                Event::KeyDown {
                    keycode: Some(keycode),
                    repeat,
                    ..
                } => {
                    if keycode == Keycode::Escape {
                        self.should_quit = true;
                    }
                    if !repeat {
                        if let Some(key) = key_from_sdl(keycode) {
                            self.input.set_key(key, true);
                        }
                    }
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(key) = key_from_sdl(keycode) {
                        self.input.set_key(key, false);
                    }
                }
                Event::MouseMotion { x, y, .. } => {
                    self.set_mouse_position_from_window_coords(x, y);
                }
                Event::MouseButtonDown {
                    mouse_btn, x, y, ..
                } => {
                    self.set_mouse_position_from_window_coords(x, y);
                    if let Some(button) = mouse_button_from_sdl(mouse_btn) {
                        self.input.set_mouse_button(button, true);
                    }
                }
                Event::MouseButtonUp {
                    mouse_btn, x, y, ..
                } => {
                    self.set_mouse_position_from_window_coords(x, y);
                    if let Some(button) = mouse_button_from_sdl(mouse_btn) {
                        self.input.set_mouse_button(button, false);
                    }
                }
                Event::ControllerDeviceAdded { .. } => self.connect_first_gamepad(),
                Event::ControllerDeviceRemoved { which, .. } => {
                    if self.primary_gamepad_id == Some(which) {
                        log::info!("primary gamepad disconnected");
                        self.primary_gamepad = None;
                        self.primary_gamepad_id = None;
                        self.input.clear_gamepad();
                        self.connect_first_gamepad();
                    }
                }
                Event::ControllerButtonDown { which, button, .. } => {
                    if self.is_primary_gamepad(which) {
                        if let Some(button) = gamepad_button_from_sdl(button) {
                            self.input.set_gamepad_button(button, true);
                        }
                    }
                }
                Event::ControllerButtonUp { which, button, .. } => {
                    if self.is_primary_gamepad(which) {
                        if let Some(button) = gamepad_button_from_sdl(button) {
                            self.input.set_gamepad_button(button, false);
                        }
                    }
                }
                Event::ControllerAxisMotion {
                    which, axis, value, ..
                } => {
                    if self.is_primary_gamepad(which) {
                        if let Some((axis, component)) = gamepad_axis_from_sdl(axis) {
                            self.input.set_gamepad_axis_component(
                                axis,
                                component,
                                normalize_gamepad_axis(value),
                            );
                        }
                    }
                }
                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::CloseRequested => self.should_quit = true,
                    // Losing keyboard focus means we will miss the key-up events
                    // for anything currently held; clear input so movement/zoom
                    // don't stay stuck on after refocusing.
                    WindowEvent::FocusLost => self.input.reset(),
                    WindowEvent::Resized(width, height)
                    | WindowEvent::PixelSizeChanged(width, height) => {
                        log::debug!("window framebuffer resized: {width}x{height}");
                    }
                    _ => {
                        log::debug!("window event: {win_event:?}");
                    }
                },
                _ => {}
            }
        }

        self.track_drawable_size_change();
        self.set_input_viewport_size();
    }

    pub fn take_stable_resize_request(&mut self) -> bool {
        if !self.pending_drawable_resize {
            return false;
        }

        let now = Instant::now();
        let drawable_size = self.window.size_in_pixels();
        if !self.resize_policy.recreate_ready(now, drawable_size) {
            return false;
        }

        self.pending_drawable_resize = false;
        self.resize_policy.note_recreate(now);
        true
    }

    fn track_drawable_size_change(&mut self) {
        let drawable_size = self.window.size_in_pixels();
        if drawable_size == self.last_drawable_size {
            return;
        }

        self.last_drawable_size = drawable_size;
        self.pending_drawable_resize = true;
        self.resize_policy.note_resize(Instant::now());
    }

    pub fn drawable_size(&self) -> (u32, u32) {
        self.window.size_in_pixels()
    }

    fn set_input_viewport_size(&mut self) {
        let (width, height) = self.window.size_in_pixels();
        self.input
            .set_viewport_size(glam::vec2(width as f32, height as f32));
    }

    fn set_mouse_position_from_window_coords(&mut self, x: f32, y: f32) {
        let (window_width, window_height) = self.window.size();
        let (drawable_width, drawable_height) = self.window.size_in_pixels();
        let scale_x = if window_width > 0 {
            drawable_width as f32 / window_width as f32
        } else {
            1.0
        };
        let scale_y = if window_height > 0 {
            drawable_height as f32 / window_height as f32
        } else {
            1.0
        };
        self.input
            .set_mouse_position(glam::vec2(x * scale_x, y * scale_y));
    }

    fn is_primary_gamepad(&self, which: u32) -> bool {
        self.primary_gamepad_id == Some(which)
    }

    fn connect_first_gamepad(&mut self) {
        if self.primary_gamepad.is_some() {
            return;
        }

        let (id, gamepad) = open_first_gamepad(&self.gamepad_subsystem);
        if let (Some(id), Some(gamepad)) = (id, gamepad) {
            log::info!(
                "connected primary gamepad: {}",
                gamepad
                    .name()
                    .unwrap_or_else(|| "unnamed controller".to_owned())
            );
            self.primary_gamepad_id = Some(id);
            self.primary_gamepad = Some(gamepad);
        }
    }
}

fn open_first_gamepad(
    gamepad_subsystem: &sdl3::GamepadSubsystem,
) -> (Option<u32>, Option<sdl3::gamepad::Gamepad>) {
    let Ok(ids) = gamepad_subsystem.gamepads() else {
        log::warn!("failed to enumerate SDL gamepads: {}", sdl3::get_error());
        return (None, None);
    };

    for id in ids {
        match gamepad_subsystem.open(id) {
            Ok(gamepad) => return (Some(id.0), Some(gamepad)),
            Err(error) => log::warn!("failed to open SDL gamepad: {error}"),
        }
    }
    (None, None)
}
