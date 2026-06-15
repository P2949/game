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

        // Raw-handle interop safety rationale:
        //
        // * SDL creates the surface against the very Vulkan instance we pass in
        //   (`instance.handle()`), so the returned handle is a `VkSurfaceKHR`
        //   valid for that instance and that instance only.
        // * `ash::vk::SurfaceKHR` is a transparent newtype over the raw 64-bit
        //   Vulkan handle, so `from_raw(raw as u64)` reconstructs the same handle
        //   without reinterpreting memory — it is a value cast, not a pointer
        //   transmute.
        // * Ownership is single: SDL hands the handle to us and does not destroy
        //   it; `Surface::drop` destroys it exactly once via the same instance's
        //   surface loader.
        // * `Surface` is owned by `VulkanContext`, which keeps the `ash::Instance`
        //   alive for at least as long as the surface, so the instance always
        //   outlives the handle it backs.
        //
        // Depending on sdl3/sdl3-sys features, the matching `use-ash-*` feature
        // can replace this raw cast with a typed API; the fallback below is the
        // shape of the interop when the crate returns raw handles.
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
