use ash::vk;
use ash::vk::Handle;

pub struct Surface {
    loader: ash::khr::surface::Instance,
    handle: vk::SurfaceKHR,
}

impl Surface {
    pub fn new(
        entry: &ash::Entry,
        instance: &ash::Instance,
        window: &sdl3::video::Window,
    ) -> anyhow::Result<Self> {
        let loader = ash::khr::surface::Instance::new(entry, instance);

        // Depending on sdl3/sdl3-sys features, you may be able to avoid raw casts
        // by enabling the matching `use-ash-*` feature. The fallback below shows
        // the shape of the interop when the crate returns raw handles.
        let raw_surface = unsafe {
            window
                .vulkan_create_surface(instance.handle().as_raw() as _)
                .map_err(anyhow::Error::msg)?
        };

        let handle = vk::SurfaceKHR::from_raw(raw_surface as u64);

        Ok(Self { loader, handle })
    }

    pub fn loader(&self) -> &ash::khr::surface::Instance {
        &self.loader
    }

    pub fn handle(&self) -> vk::SurfaceKHR {
        self.handle
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_surface(self.handle, None);
        }
    }
}
