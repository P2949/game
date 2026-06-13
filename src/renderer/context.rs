use ash::{Entry, vk};
use std::ffi::{CStr, CString};

use crate::renderer::{debug, device, frame, swapchain};

pub struct VulkanContext {
    // Keep the Vulkan loader alive for objects created from it.
    #[allow(dead_code)]
    pub entry: Entry,
    pub instance: ash::Instance,
    pub debug_utils: Option<ash::ext::debug_utils::Instance>,
    pub debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
    pub surface: crate::renderer::surface::Surface,
    #[allow(dead_code)]
    pub physical_device: vk::PhysicalDevice,
    #[allow(dead_code)]
    pub queue_families: device::QueueFamilies,
    pub logical_device: Option<device::LogicalDevice>,
    pub swapchain: swapchain::Swapchain,
    pub swapchain_image_views: swapchain::SwapchainImageViews,
    pub frames: Vec<frame::FrameData>,
    #[allow(dead_code)]
    pub current_frame: usize,
}

impl VulkanContext {
    pub fn new(window: &sdl3::video::Window) -> anyhow::Result<Self> {
        let entry = unsafe { Entry::load()? };

        let app_name = CString::new("sdl3-ash-demo")?;
        let engine_name = CString::new("no-engine")?;

        let app_info = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .application_version(vk::make_api_version(0, 0, 1, 0))
            .engine_name(&engine_name)
            .engine_version(vk::make_api_version(0, 0, 1, 0))
            .api_version(vk::API_VERSION_1_3);

        // SDL tells you which platform-specific instance extensions are needed
        // to create a surface for this window.
        let sdl_extensions = window
            .vulkan_instance_extensions()
            .map_err(anyhow::Error::msg)?;

        let mut extension_names: Vec<CString> = sdl_extensions
            .iter()
            .map(|name| CString::new(name.as_str()).expect("SDL extension name contains NUL"))
            .collect();

        if cfg!(debug_assertions) {
            extension_names.push(ash::ext::debug_utils::NAME.to_owned());
        }

        let extension_ptrs: Vec<*const i8> =
            extension_names.iter().map(|name| name.as_ptr()).collect();

        let layer_names: Vec<&CStr> = if cfg!(debug_assertions) {
            vec![debug::VALIDATION_LAYER]
        } else {
            vec![]
        };
        let layer_ptrs: Vec<*const i8> = layer_names.iter().map(|layer| layer.as_ptr()).collect();

        let mut validation_features = vk::ValidationFeaturesEXT::default()
            .enabled_validation_features(&[
                vk::ValidationFeatureEnableEXT::SYNCHRONIZATION_VALIDATION,
            ]);

        let mut debug_create_info = debug::debug_messenger_create_info();

        let mut instance_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extension_ptrs)
            .enabled_layer_names(&layer_ptrs);

        if cfg!(debug_assertions) {
            instance_info = instance_info
                .push_next(&mut validation_features)
                .push_next(&mut debug_create_info);
        }

        let instance = unsafe { entry.create_instance(&instance_info, None)? };

        let (debug_utils, debug_messenger) = if cfg!(debug_assertions) {
            let debug_utils = ash::ext::debug_utils::Instance::new(&entry, &instance);
            let messenger = unsafe {
                debug_utils
                    .create_debug_utils_messenger(&debug::debug_messenger_create_info(), None)?
            };
            (Some(debug_utils), Some(messenger))
        } else {
            (None, None)
        };

        let surface = crate::renderer::surface::Surface::new(&entry, &instance, window)?;
        let selected_device =
            device::select_physical_device(&instance, &surface.loader, surface.handle)?;
        let logical_device = device::LogicalDevice::new(
            &instance,
            selected_device.physical_device,
            selected_device.queue_families,
        )?;
        let swapchain = swapchain::Swapchain::new(
            &instance,
            &logical_device.device,
            selected_device.physical_device,
            &surface.loader,
            surface.handle,
            selected_device.queue_families,
            window.size_in_pixels(),
            vk::SwapchainKHR::null(),
        )?;
        let swapchain_image_views = swapchain::SwapchainImageViews::new(
            &logical_device.device,
            &swapchain.images,
            swapchain.format,
        )?;
        let frames: Vec<_> = (0..frame::MAX_FRAMES_IN_FLIGHT)
            .map(|_| {
                frame::FrameData::new(
                    &logical_device.device,
                    selected_device.queue_families.graphics,
                )
            })
            .collect::<anyhow::Result<_>>()?;

        Ok(Self {
            entry,
            instance,
            debug_utils,
            debug_messenger,
            surface,
            physical_device: selected_device.physical_device,
            queue_families: selected_device.queue_families,
            logical_device: Some(logical_device),
            swapchain,
            swapchain_image_views,
            frames,
            current_frame: 0,
        })
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        unsafe {
            if let Some(logical_device) = self.logical_device.as_ref() {
                let _ = logical_device.device.device_wait_idle();

                for frame in &self.frames {
                    frame.destroy(&logical_device.device);
                }

                self.swapchain_image_views.destroy(&logical_device.device);
            }

            self.swapchain.destroy();

            if let Some(logical_device) = self.logical_device.take() {
                drop(logical_device);
            }

            self.surface.destroy();

            if let (Some(debug_utils), Some(messenger)) = (&self.debug_utils, self.debug_messenger)
            {
                debug_utils.destroy_debug_utils_messenger(messenger, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}
#[allow(dead_code)]
unsafe fn record_clear_commands(
    device: &ash::Device,
    cmd: vk::CommandBuffer,
    image: vk::Image,
    image_view: vk::ImageView,
    extent: vk::Extent2D,
    t: f32,
) -> anyhow::Result<()> {
    let begin_info = vk::CommandBufferBeginInfo::default();
    unsafe {
        device.begin_command_buffer(cmd, &begin_info)?;
    }

    unsafe {
        crate::renderer::texture::transition_image(
            device,
            cmd,
            image,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            vk::PipelineStageFlags2::TOP_OF_PIPE,
            vk::AccessFlags2::empty(),
            vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
        );
    }

    let clear = vk::ClearValue {
        color: vk::ClearColorValue {
            float32: [0.02, 0.02, 0.04 + 0.04 * t.sin().abs(), 1.0],
        },
    };

    let color_attachment = vk::RenderingAttachmentInfo::default()
        .image_view(image_view)
        .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .clear_value(clear);

    let render_area = vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent,
    };

    let rendering_info = vk::RenderingInfo::default()
        .render_area(render_area)
        .layer_count(1)
        .color_attachments(std::slice::from_ref(&color_attachment));

    unsafe {
        device.cmd_begin_rendering(cmd, &rendering_info);
        device.cmd_end_rendering(cmd);

        crate::renderer::texture::transition_image(
            device,
            cmd,
            image,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            vk::ImageLayout::PRESENT_SRC_KHR,
            vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            vk::PipelineStageFlags2::BOTTOM_OF_PIPE,
            vk::AccessFlags2::empty(),
        );

        device.end_command_buffer(cmd)?;
    }
    Ok(())
}
