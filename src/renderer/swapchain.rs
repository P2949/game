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

    // Preference order: sRGB nonlinear color space throughout, BGRA before RGBA
    // (BGRA is the common desktop optimal layout), sRGB-encoded formats before
    // UNORM. This avoids settling for the surface's first advertised format on a
    // platform that offers, say, R8G8B8A8_SRGB but not B8G8R8A8_SRGB.
    const PREFERRED: [(vk::Format, vk::ColorSpaceKHR); 4] = [
        (vk::Format::B8G8R8A8_SRGB, vk::ColorSpaceKHR::SRGB_NONLINEAR),
        (vk::Format::R8G8B8A8_SRGB, vk::ColorSpaceKHR::SRGB_NONLINEAR),
        (
            vk::Format::B8G8R8A8_UNORM,
            vk::ColorSpaceKHR::SRGB_NONLINEAR,
        ),
        (
            vk::Format::R8G8B8A8_UNORM,
            vk::ColorSpaceKHR::SRGB_NONLINEAR,
        ),
    ];

    for (format, color_space) in PREFERRED {
        if let Some(found) = formats
            .iter()
            .copied()
            .find(|candidate| candidate.format == format && candidate.color_space == color_space)
        {
            return Ok(found);
        }
    }

    // Nothing preferred is advertised; use whatever the surface lists first.
    Ok(formats[0])
}

/// Picks a composite-alpha mode from the surface's advertised set. Prefers
/// `OPAQUE` (no blending with the desktop behind the window), then the multiplied
/// modes, then `INHERIT`. A conformant surface always advertises at least one
/// mode; if none is advertised this errors rather than fabricating an unsupported
/// `OPAQUE`, so swapchain creation fails with a clear message instead of an opaque
/// validation error.
pub fn choose_composite_alpha(
    supported: vk::CompositeAlphaFlagsKHR,
) -> anyhow::Result<vk::CompositeAlphaFlagsKHR> {
    const PREFERRED: [vk::CompositeAlphaFlagsKHR; 4] = [
        vk::CompositeAlphaFlagsKHR::OPAQUE,
        vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED,
        vk::CompositeAlphaFlagsKHR::POST_MULTIPLIED,
        vk::CompositeAlphaFlagsKHR::INHERIT,
    ];

    PREFERRED
        .into_iter()
        .find(|&mode| supported.contains(mode))
        .ok_or_else(|| anyhow::anyhow!("surface advertises no supported composite alpha mode"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PresentModePreference {
    Fifo,
    Mailbox,
    Immediate,
}

fn requested_present_mode() -> PresentModePreference {
    match std::env::var("GAME_PRESENT_MODE") {
        Ok(value) => match parse_present_mode(&value) {
            Some(mode) => mode,
            None => {
                log::warn!("invalid GAME_PRESENT_MODE={value:?}; falling back to fifo");
                PresentModePreference::Fifo
            }
        },
        Err(_) => PresentModePreference::Fifo,
    }
}

fn parse_present_mode(value: &str) -> Option<PresentModePreference> {
    match value.trim().to_ascii_lowercase().as_str() {
        "fifo" => Some(PresentModePreference::Fifo),
        "mailbox" => Some(PresentModePreference::Mailbox),
        "immediate" => Some(PresentModePreference::Immediate),
        _ => None,
    }
}

pub fn choose_present_mode(modes: &[vk::PresentModeKHR]) -> anyhow::Result<vk::PresentModeKHR> {
    choose_present_mode_for_request(modes, requested_present_mode())
}

fn choose_present_mode_for_request(
    modes: &[vk::PresentModeKHR],
    requested: PresentModePreference,
) -> anyhow::Result<vk::PresentModeKHR> {
    if modes.is_empty() {
        anyhow::bail!("no present modes available for swapchain creation");
    }

    let requested_vk = match requested {
        PresentModePreference::Fifo => vk::PresentModeKHR::FIFO,
        PresentModePreference::Mailbox => vk::PresentModeKHR::MAILBOX,
        PresentModePreference::Immediate => vk::PresentModeKHR::IMMEDIATE,
    };
    let fifo = modes
        .iter()
        .copied()
        .find(|mode| *mode == vk::PresentModeKHR::FIFO)
        .unwrap_or(modes[0]);

    let selected = if modes.contains(&requested_vk) {
        requested_vk
    } else {
        if requested != PresentModePreference::Fifo {
            log::warn!("requested present mode {requested:?} is unavailable; falling back to FIFO");
        }
        fifo
    };

    Ok(selected)
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

fn validate_extent(extent: vk::Extent2D) -> anyhow::Result<()> {
    if extent.width == 0 || extent.height == 0 {
        anyhow::bail!("swapchain extent is zero");
    }
    Ok(())
}

pub(crate) struct Swapchain {
    loader: ash::khr::swapchain::Device,
    handle: vk::SwapchainKHR,
    format: vk::Format,
    extent: vk::Extent2D,
    images: Vec<vk::Image>,
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
        validate_extent(extent)?;
        let composite_alpha =
            choose_composite_alpha(support.capabilities.supported_composite_alpha)?;

        let mut image_count = support.capabilities.min_image_count + 1;
        if support.capabilities.max_image_count > 0 {
            image_count = image_count.min(support.capabilities.max_image_count);
        }

        let queue_indices = [queue_families.graphics, queue_families.present];
        let concurrent = queue_families.graphics != queue_families.present;

        let mut info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(if concurrent {
                vk::SharingMode::CONCURRENT
            } else {
                vk::SharingMode::EXCLUSIVE
            })
            .pre_transform(support.capabilities.current_transform)
            .composite_alpha(composite_alpha)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(old_swapchain);

        if concurrent {
            info = info.queue_family_indices(&queue_indices);
        }

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
            "created Vulkan swapchain: images={}, format={:?}, extent={}x{}, present_mode={:?}",
            images.len(),
            surface_format.format,
            extent.width,
            extent.height,
            present_mode
        );

        Ok(Self {
            loader,
            handle,
            format: surface_format.format,
            extent,
            images,
        })
    }

    pub fn loader(&self) -> &ash::khr::swapchain::Device {
        &self.loader
    }

    pub fn handle(&self) -> vk::SwapchainKHR {
        self.handle
    }

    pub fn format(&self) -> vk::Format {
        self.format
    }

    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }

    pub fn images(&self) -> &[vk::Image] {
        &self.images
    }

    /// Returns the swapchain image at `index`. Vulkan only hands back acquired
    /// indices that are in range, but the whole render path is `Result`-based, so
    /// an out-of-range index reports an error instead of panicking on a raw index.
    pub fn image(&self, index: usize) -> anyhow::Result<vk::Image> {
        self.images
            .get(index)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("swapchain image index {index} out of range"))
    }

    pub fn image_count(&self) -> usize {
        self.images.len()
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
    views: Vec<vk::ImageView>,
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

    /// Returns the image view at `index`, erroring (rather than panicking on a
    /// raw index) when it is out of range. See [`Swapchain::image`].
    pub fn view(&self, index: usize) -> anyhow::Result<vk::ImageView> {
        self.views
            .get(index)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("swapchain image view index {index} out of range"))
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
    use super::{
        PresentModePreference, choose_composite_alpha, choose_present_mode_for_request,
        choose_surface_format, parse_present_mode, validate_extent,
    };
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
    fn rgba_srgb_is_selected_when_bgra_srgb_is_absent() {
        // BGRA_SRGB not offered, but RGBA_SRGB is — and it must win over the
        // surface's first advertised (non-preferred-rank) format.
        let formats = [
            format(
                vk::Format::R8G8B8A8_UNORM,
                vk::ColorSpaceKHR::SRGB_NONLINEAR,
            ),
            format(vk::Format::R8G8B8A8_SRGB, vk::ColorSpaceKHR::SRGB_NONLINEAR),
        ];
        let chosen = choose_surface_format(&formats).unwrap();
        assert_eq!(chosen.format, vk::Format::R8G8B8A8_SRGB);
    }

    #[test]
    fn unorm_srgb_nonlinear_is_selected_when_no_srgb_format_exists() {
        // No sRGB-encoded format, but a UNORM + SRGB_NONLINEAR is acceptable.
        let formats = [format(
            vk::Format::R8G8B8A8_UNORM,
            vk::ColorSpaceKHR::SRGB_NONLINEAR,
        )];
        let chosen = choose_surface_format(&formats).unwrap();
        assert_eq!(chosen.format, vk::Format::R8G8B8A8_UNORM);
        assert_eq!(chosen.color_space, vk::ColorSpaceKHR::SRGB_NONLINEAR);
    }

    #[test]
    fn first_format_is_used_when_nothing_preferred_is_advertised() {
        // None of these use SRGB_NONLINEAR, so no preference matches and the
        // first advertised format is the last-resort choice.
        let formats = [
            format(
                vk::Format::R8G8B8A8_UNORM,
                vk::ColorSpaceKHR::DISPLAY_P3_NONLINEAR_EXT,
            ),
            format(
                vk::Format::B8G8R8A8_SRGB,
                vk::ColorSpaceKHR::DISPLAY_P3_NONLINEAR_EXT,
            ),
        ];
        let chosen = choose_surface_format(&formats).unwrap();
        assert_eq!(chosen.format, vk::Format::R8G8B8A8_UNORM);
        assert_eq!(
            chosen.color_space,
            vk::ColorSpaceKHR::DISPLAY_P3_NONLINEAR_EXT
        );
    }

    #[test]
    fn composite_alpha_prefers_opaque_when_supported() {
        let supported = vk::CompositeAlphaFlagsKHR::OPAQUE | vk::CompositeAlphaFlagsKHR::INHERIT;
        assert_eq!(
            choose_composite_alpha(supported).unwrap(),
            vk::CompositeAlphaFlagsKHR::OPAQUE
        );
    }

    #[test]
    fn composite_alpha_falls_through_preference_order() {
        let supported = vk::CompositeAlphaFlagsKHR::POST_MULTIPLIED
            | vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED;
        // PRE_MULTIPLIED outranks POST_MULTIPLIED.
        assert_eq!(
            choose_composite_alpha(supported).unwrap(),
            vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED
        );
        assert_eq!(
            choose_composite_alpha(vk::CompositeAlphaFlagsKHR::INHERIT).unwrap(),
            vk::CompositeAlphaFlagsKHR::INHERIT
        );
    }

    #[test]
    fn composite_alpha_errors_when_none_advertised() {
        assert!(choose_composite_alpha(vk::CompositeAlphaFlagsKHR::empty()).is_err());
    }

    #[test]
    fn empty_present_mode_list_is_an_error() {
        assert!(choose_present_mode_for_request(&[], PresentModePreference::Fifo).is_err());
    }

    #[test]
    fn present_mode_parser_accepts_expected_values() {
        assert_eq!(
            parse_present_mode("fifo"),
            Some(PresentModePreference::Fifo)
        );
        assert_eq!(
            parse_present_mode("MAILBOX"),
            Some(PresentModePreference::Mailbox)
        );
        assert_eq!(
            parse_present_mode(" immediate "),
            Some(PresentModePreference::Immediate)
        );
        assert_eq!(parse_present_mode("bad"), None);
    }

    #[test]
    fn fifo_present_mode_is_the_default_request() {
        let modes = [vk::PresentModeKHR::FIFO, vk::PresentModeKHR::MAILBOX];
        assert_eq!(
            choose_present_mode_for_request(&modes, PresentModePreference::Fifo).unwrap(),
            vk::PresentModeKHR::FIFO
        );
    }

    #[test]
    fn requested_mailbox_is_used_when_available() {
        let modes = [vk::PresentModeKHR::FIFO, vk::PresentModeKHR::MAILBOX];
        assert_eq!(
            choose_present_mode_for_request(&modes, PresentModePreference::Mailbox).unwrap(),
            vk::PresentModeKHR::MAILBOX
        );
    }

    #[test]
    fn requested_immediate_falls_back_to_fifo_when_unavailable() {
        let modes = [vk::PresentModeKHR::FIFO, vk::PresentModeKHR::MAILBOX];
        assert_eq!(
            choose_present_mode_for_request(&modes, PresentModePreference::Immediate).unwrap(),
            vk::PresentModeKHR::FIFO
        );
    }

    #[test]
    fn zero_width_extent_is_rejected() {
        assert!(
            validate_extent(vk::Extent2D {
                width: 0,
                height: 1
            })
            .is_err()
        );
    }

    #[test]
    fn zero_height_extent_is_rejected() {
        assert!(
            validate_extent(vk::Extent2D {
                width: 1,
                height: 0
            })
            .is_err()
        );
    }

    #[test]
    fn nonzero_extent_is_accepted() {
        assert!(
            validate_extent(vk::Extent2D {
                width: 1,
                height: 1
            })
            .is_ok()
        );
    }
}
