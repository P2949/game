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
pub fn choose_surface_format(
    formats: &[vk::SurfaceFormatKHR],
) -> anyhow::Result<vk::SurfaceFormatKHR> {
    // Device selection rejects surfaces with no formats, but guard here too so a
    // direct caller can never index an empty slice and panic.
    if formats.is_empty() {
        anyhow::bail!("no surface formats available for swapchain creation");
    }

    Ok(formats
        .iter()
        .copied()
        .find(|format| {
            format.format == vk::Format::B8G8R8A8_SRGB
                && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .unwrap_or(formats[0]))
}

pub fn choose_present_mode(modes: &[vk::PresentModeKHR]) -> anyhow::Result<vk::PresentModeKHR> {
    // FIFO is guaranteed by Vulkan WSI and is the safest default.
    // MAILBOX is good for low-latency uncapped rendering if available.
    if modes.is_empty() {
        anyhow::bail!("no present modes available for swapchain creation");
    }

    Ok(modes
        .iter()
        .copied()
        .find(|mode| *mode == vk::PresentModeKHR::MAILBOX)
        .or_else(|| {
            modes
                .iter()
                .copied()
                .find(|mode| *mode == vk::PresentModeKHR::FIFO)
        })
        .unwrap_or(modes[0]))
}

pub fn choose_extent(
    capabilities: vk::SurfaceCapabilitiesKHR,
    window_size: (u32, u32),
) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        return capabilities.current_extent;
    }

    vk::Extent2D {
        width: window_size.0.clamp(
            capabilities.min_image_extent.width,
            capabilities.max_image_extent.width,
        ),
        height: window_size.1.clamp(
            capabilities.min_image_extent.height,
            capabilities.max_image_extent.height,
        ),
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
        let surface_format = choose_surface_format(&support.formats)?;
        let present_mode = choose_present_mode(&support.present_modes)?;
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
        let images = match unsafe { loader.get_swapchain_images(handle) } {
            Ok(images) => images,
            Err(err) => {
                unsafe {
                    loader.destroy_swapchain(handle, None);
                }
                return Err(err.into());
            }
        };

        log::info!(
            "created Vulkan swapchain: images={}, format={:?}, extent={}x{}",
            images.len(),
            surface_format.format,
            extent.width,
            extent.height
        );

        Ok(Self {
            loader,
            handle,
            format: surface_format.format,
            extent,
            images,
        })
    }

    pub fn destroy(&mut self) {
        if self.handle == vk::SwapchainKHR::null() {
            return;
        }

        unsafe {
            self.loader.destroy_swapchain(self.handle, None);
        }
        self.handle = vk::SwapchainKHR::null();
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        self.destroy();
    }
}

pub struct SwapchainImageViews {
    device: ash::Device,
    pub views: Vec<vk::ImageView>,
}

impl SwapchainImageViews {
    pub fn new(
        device: &ash::Device,
        images: &[vk::Image],
        format: vk::Format,
    ) -> anyhow::Result<Self> {
        let mut views = Vec::with_capacity(images.len());

        for &image in images {
            let subresource_range = vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1);

            let info = vk::ImageViewCreateInfo::default()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format)
                .subresource_range(subresource_range);

            let view = match unsafe { device.create_image_view(&info, None) } {
                Ok(view) => view,
                Err(err) => {
                    for &created_view in &views {
                        unsafe {
                            device.destroy_image_view(created_view, None);
                        }
                    }
                    return Err(err.into());
                }
            };
            views.push(view);
        }

        log::info!("created {} swapchain image views", views.len());

        Ok(Self {
            device: device.clone(),
            views,
        })
    }

    pub fn destroy(&mut self) {
        for view in self.views.drain(..) {
            unsafe {
                self.device.destroy_image_view(view, None);
            }
        }
    }
}

impl Drop for SwapchainImageViews {
    fn drop(&mut self) {
        self.destroy();
    }
}

#[cfg(test)]
mod tests {
    use super::{choose_present_mode, choose_surface_format};
    use ash::vk;

    fn format(format: vk::Format, color_space: vk::ColorSpaceKHR) -> vk::SurfaceFormatKHR {
        vk::SurfaceFormatKHR {
            format,
            color_space,
        }
    }

    #[test]
    fn empty_format_list_is_an_error() {
        assert!(choose_surface_format(&[]).is_err());
    }

    #[test]
    fn preferred_srgb_format_is_selected_when_present() {
        let formats = [
            format(
                vk::Format::R8G8B8A8_UNORM,
                vk::ColorSpaceKHR::SRGB_NONLINEAR,
            ),
            format(vk::Format::B8G8R8A8_SRGB, vk::ColorSpaceKHR::SRGB_NONLINEAR),
        ];
        let chosen = choose_surface_format(&formats).unwrap();
        assert_eq!(chosen.format, vk::Format::B8G8R8A8_SRGB);
    }

    #[test]
    fn first_format_is_used_as_fallback() {
        let formats = [
            format(
                vk::Format::R8G8B8A8_UNORM,
                vk::ColorSpaceKHR::SRGB_NONLINEAR,
            ),
            format(vk::Format::R8G8B8A8_SRGB, vk::ColorSpaceKHR::SRGB_NONLINEAR),
        ];
        let chosen = choose_surface_format(&formats).unwrap();
        assert_eq!(chosen.format, vk::Format::R8G8B8A8_UNORM);
    }

    #[test]
    fn empty_present_mode_list_is_an_error() {
        assert!(choose_present_mode(&[]).is_err());
    }

    #[test]
    fn mailbox_present_mode_is_preferred_when_present() {
        let modes = [vk::PresentModeKHR::FIFO, vk::PresentModeKHR::MAILBOX];
        assert_eq!(
            choose_present_mode(&modes).unwrap(),
            vk::PresentModeKHR::MAILBOX
        );
    }

    #[test]
    fn fifo_present_mode_is_the_fallback() {
        let modes = [vk::PresentModeKHR::FIFO, vk::PresentModeKHR::IMMEDIATE];
        assert_eq!(
            choose_present_mode(&modes).unwrap(),
            vk::PresentModeKHR::FIFO
        );
    }

    #[test]
    fn first_advertised_present_mode_is_used_when_mailbox_and_fifo_are_absent() {
        let modes = [
            vk::PresentModeKHR::IMMEDIATE,
            vk::PresentModeKHR::FIFO_RELAXED,
        ];
        assert_eq!(
            choose_present_mode(&modes).unwrap(),
            vk::PresentModeKHR::IMMEDIATE
        );
    }
}
