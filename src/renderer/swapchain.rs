use ash::vk;

pub struct SwapchainSupport {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

pub fn query_swapchain_support(
    surface_loader: &ash::khr::surface::Instance,
    physical_device: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
) -> anyhow::Result<SwapchainSupport> {
    unsafe {
        Ok(SwapchainSupport {
            capabilities: surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface)?,
            formats: surface_loader
                .get_physical_device_surface_formats(physical_device, surface)?,
            present_modes: surface_loader
                .get_physical_device_surface_present_modes(physical_device, surface)?,
        })
    }
}
pub fn choose_surface_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
    formats
        .iter()
        .copied()
        .find(|format| {
            format.format == vk::Format::B8G8R8A8_SRGB
                && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .unwrap_or(formats[0])
}

pub fn choose_present_mode(modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
    // FIFO is guaranteed by Vulkan WSI and is the safest default.
    // MAILBOX is good for low-latency uncapped rendering if available.
    modes
        .iter()
        .copied()
        .find(|mode| *mode == vk::PresentModeKHR::MAILBOX)
        .unwrap_or(vk::PresentModeKHR::FIFO)
}

pub fn choose_extent(
    capabilities: vk::SurfaceCapabilitiesKHR,
    window_size: (u32, u32),
) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        return capabilities.current_extent;
    }

    vk::Extent2D {
        width: window_size
            .0
            .clamp(capabilities.min_image_extent.width, capabilities.max_image_extent.width),
        height: window_size
            .1
            .clamp(capabilities.min_image_extent.height, capabilities.max_image_extent.height),
    }
}
pub struct Swapchain {
    pub loader: ash::khr::swapchain::Device,
    pub handle: vk::SwapchainKHR,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub images: Vec<vk::Image>,
}

impl Swapchain {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        surface_loader: &ash::khr::surface::Instance,
        surface: vk::SurfaceKHR,
        queue_families: crate::renderer::device::QueueFamilies,
        window_size: (u32, u32),
        old_swapchain: vk::SwapchainKHR,
    ) -> anyhow::Result<Self> {
        let support = query_swapchain_support(surface_loader, physical_device, surface)?;
        let surface_format = choose_surface_format(&support.formats);
        let present_mode = choose_present_mode(&support.present_modes);
        let extent = choose_extent(support.capabilities, window_size);

        let mut image_count = support.capabilities.min_image_count + 1;
        if support.capabilities.max_image_count > 0 {
            image_count = image_count.min(support.capabilities.max_image_count);
        }

        let queue_indices = [queue_families.graphics, queue_families.present];
        let sharing_mode = if queue_families.graphics != queue_families.present {
            vk::SharingMode::CONCURRENT
        } else {
            vk::SharingMode::EXCLUSIVE
        };

        let info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(sharing_mode)
            .queue_family_indices(&queue_indices)
            .pre_transform(support.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(old_swapchain);

        let loader = ash::khr::swapchain::Device::new(instance, device);
        let handle = unsafe { loader.create_swapchain(&info, None)? };
        let images = unsafe { loader.get_swapchain_images(handle)? };

        Ok(Self {
            loader,
            handle,
            format: surface_format.format,
            extent,
            images,
        })
    }

    pub unsafe fn destroy(&self) {
        self.loader.destroy_swapchain(self.handle, None);
    }
}
