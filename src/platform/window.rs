use anyhow::Context;
use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::Keycode;
use sdl3::video::Window;

use crate::platform::input::InputState;

pub struct Platform {
    // Keep the SDL context alive for the lifetime of the window and event pump.
    #[allow(dead_code)]
    pub sdl: sdl3::Sdl,
    pub window: Window,
    pub event_pump: sdl3::EventPump,
    pub should_quit: bool,
    pub framebuffer_resized: bool,
    pub input: InputState,
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

        Ok(Self {
            sdl,
            window,
            event_pump,
            should_quit: false,
            framebuffer_resized: false,
            input: InputState::default(),
        })
    }

    pub fn pump_events(&mut self) {
        self.framebuffer_resized = false;
        self.input.begin_frame();

        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => self.should_quit = true,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if keycode == Keycode::Escape {
                        self.should_quit = true;
                    }
                    self.input.set_key(keycode, true);
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => self.input.set_key(keycode, false),
                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::CloseRequested => self.should_quit = true,
                    WindowEvent::Resized(width, height)
                    | WindowEvent::PixelSizeChanged(width, height) => {
                        self.framebuffer_resized = true;
                        log::debug!("window framebuffer resized: {width}x{height}");
                    }
                    _ => {
                        log::debug!("window event: {win_event:?}");
                    }
                },
                _ => {}
            }
        }
    }

    pub fn drawable_size(&self) -> (u32, u32) {
        self.window.size_in_pixels()
    }
}
