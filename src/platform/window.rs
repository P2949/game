use anyhow::Context;
use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::Keycode;
use sdl3::video::Window;
use std::time::Instant;

use crate::platform::input::InputState;
use crate::platform::resize::ResizePolicy;

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
        })
    }

    pub fn pump_events(&mut self) {
        self.input.begin_frame();

        for event in self.event_pump.poll_iter() {
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
                        self.input.set_key(keycode, true);
                    }
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => self.input.set_key(keycode, false),
                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::CloseRequested => self.should_quit = true,
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

    pub fn resize_pending(&self) -> bool {
        self.pending_drawable_resize
            && !self
                .resize_policy
                .recreate_ready(Instant::now(), self.window.size_in_pixels())
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
}
