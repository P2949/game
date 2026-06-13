use ash::{Entry, vk};
use gpu_allocator::MemoryLocation;
use std::ffi::{CStr, CString};

use crate::renderer::vertex::Vertex2D;
use crate::renderer::{buffer, debug, device, frame, pipeline, swapchain};

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
    pub allocator: Option<gpu_allocator::vulkan::Allocator>,
    pub triangle_vertex_buffer: Option<buffer::Buffer>,
    pub swapchain: swapchain::Swapchain,
    pub swapchain_image_views: swapchain::SwapchainImageViews,
    pub graphics_pipeline: pipeline::GraphicsPipeline,
    pub image_render_finished: Vec<vk::Semaphore>,
    pub frames: Vec<frame::FrameData>,
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
        let mut allocator = buffer::create_allocator(
            instance.clone(),
            logical_device.device.clone(),
            selected_device.physical_device,
        )?;
        let triangle_vertex_buffer =
            create_triangle_vertex_buffer(&logical_device.device, &mut allocator)?;
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
        let graphics_pipeline =
            pipeline::GraphicsPipeline::new_triangle(&logical_device.device, swapchain.format)?;
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let mut image_render_finished = Vec::with_capacity(swapchain.images.len());
        for _ in &swapchain.images {
            image_render_finished.push(unsafe {
                logical_device
                    .device
                    .create_semaphore(&semaphore_info, None)?
            });
        }

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
            allocator: Some(allocator),
            triangle_vertex_buffer: Some(triangle_vertex_buffer),
            swapchain,
            swapchain_image_views,
            graphics_pipeline,
            image_render_finished,
            frames,
            current_frame: 0,
        })
    }

    pub fn render(&mut self, t: f32) -> anyhow::Result<()> {
        let Some(logical_device) = self.logical_device.as_ref() else {
            anyhow::bail!("cannot render after logical device has been destroyed");
        };

        let device = &logical_device.device;
        let frame = &self.frames[self.current_frame];
        let triangle_vertex_buffer = self
            .triangle_vertex_buffer
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("triangle vertex buffer has been destroyed"))?;

        unsafe {
            device.wait_for_fences(std::slice::from_ref(&frame.in_flight), true, u64::MAX)?;

            let (image_index, suboptimal) = self.swapchain.loader.acquire_next_image(
                self.swapchain.handle,
                u64::MAX,
                frame.image_available,
                vk::Fence::null(),
            )?;

            if suboptimal {
                log::debug!("swapchain is suboptimal; resize handling arrives later");
            }

            device.reset_fences(std::slice::from_ref(&frame.in_flight))?;
            device
                .reset_command_buffer(frame.command_buffer, vk::CommandBufferResetFlags::empty())?;

            let image_index_usize = image_index as usize;
            let render_finished = self.image_render_finished[image_index_usize];
            record_clear_commands(
                device,
                frame.command_buffer,
                self.swapchain.images[image_index_usize],
                self.swapchain_image_views.views[image_index_usize],
                self.swapchain.extent,
                self.graphics_pipeline.pipeline,
                triangle_vertex_buffer.handle,
                t,
            )?;

            submit_clear_frame(
                device,
                logical_device.graphics_queue,
                logical_device.present_queue,
                &self.swapchain.loader,
                self.swapchain.handle,
                frame,
                render_finished,
                image_index,
            )?;
        }

        self.current_frame = (self.current_frame + 1) % frame::MAX_FRAMES_IN_FLIGHT;

        Ok(())
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        unsafe {
            if let Some(logical_device) = self.logical_device.as_ref() {
                let _ = logical_device.device.device_wait_idle();

                if let (Some(vertex_buffer), Some(allocator)) =
                    (self.triangle_vertex_buffer.take(), self.allocator.as_mut())
                {
                    vertex_buffer.destroy(&logical_device.device, allocator);
                }

                for &semaphore in &self.image_render_finished {
                    logical_device.device.destroy_semaphore(semaphore, None);
                }

                for frame in &self.frames {
                    frame.destroy(&logical_device.device);
                }

                self.graphics_pipeline.destroy(&logical_device.device);
                self.swapchain_image_views.destroy(&logical_device.device);
            }

            if let Some(allocator) = self.allocator.take() {
                drop(allocator);
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

fn create_triangle_vertex_buffer(
    device: &ash::Device,
    allocator: &mut gpu_allocator::vulkan::Allocator,
) -> anyhow::Result<buffer::Buffer> {
    let vertices = [
        Vertex2D {
            pos: [0.0, -0.5],
            color: [1.0, 0.0, 0.0],
        },
        Vertex2D {
            pos: [0.5, 0.5],
            color: [0.0, 1.0, 0.0],
        },
        Vertex2D {
            pos: [-0.5, 0.5],
            color: [0.0, 0.0, 1.0],
        },
    ];

    let size = std::mem::size_of_val(&vertices) as vk::DeviceSize;
    let mut vertex_buffer = buffer::Buffer::new(
        device,
        allocator,
        size,
        vk::BufferUsageFlags::VERTEX_BUFFER,
        MemoryLocation::CpuToGpu,
        "triangle vertices",
    )?;

    let allocation = vertex_buffer.allocation.as_mut().unwrap();
    let mapped = allocation
        .mapped_ptr()
        .expect("CpuToGpu allocation should be mapped");

    unsafe {
        std::ptr::copy_nonoverlapping(
            vertices.as_ptr() as *const u8,
            mapped.as_ptr() as *mut u8,
            size as usize,
        );
    }

    Ok(vertex_buffer)
}

unsafe fn record_clear_commands(
    device: &ash::Device,
    cmd: vk::CommandBuffer,
    image: vk::Image,
    image_view: vk::ImageView,
    extent: vk::Extent2D,
    triangle_pipeline: vk::Pipeline,
    triangle_vertex_buffer: vk::Buffer,
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
            vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
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

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent,
        };

        device.cmd_set_viewport(cmd, 0, std::slice::from_ref(&viewport));
        device.cmd_set_scissor(cmd, 0, std::slice::from_ref(&scissor));

        device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, triangle_pipeline);

        let vertex_buffers = [triangle_vertex_buffer];
        let offsets = [0_u64];
        device.cmd_bind_vertex_buffers(cmd, 0, &vertex_buffers, &offsets);

        device.cmd_draw(cmd, 3, 1, 0, 0);

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

unsafe fn submit_clear_frame(
    device: &ash::Device,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    swapchain_loader: &ash::khr::swapchain::Device,
    swapchain: vk::SwapchainKHR,
    frame: &crate::renderer::frame::FrameData,
    render_finished: vk::Semaphore,
    image_index: u32,
) -> anyhow::Result<()> {
    let wait_info = vk::SemaphoreSubmitInfo::default()
        .semaphore(frame.image_available)
        .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT);

    let cmd_info = vk::CommandBufferSubmitInfo::default().command_buffer(frame.command_buffer);

    let signal_info = vk::SemaphoreSubmitInfo::default()
        .semaphore(render_finished)
        .stage_mask(vk::PipelineStageFlags2::ALL_GRAPHICS);

    let submit_info = vk::SubmitInfo2::default()
        .wait_semaphore_infos(std::slice::from_ref(&wait_info))
        .command_buffer_infos(std::slice::from_ref(&cmd_info))
        .signal_semaphore_infos(std::slice::from_ref(&signal_info));

    unsafe {
        device.queue_submit2(
            graphics_queue,
            std::slice::from_ref(&submit_info),
            frame.in_flight,
        )?;
    }

    let wait_semaphores = [render_finished];
    let swapchains = [swapchain];
    let image_indices = [image_index];

    let present_info = vk::PresentInfoKHR::default()
        .wait_semaphores(&wait_semaphores)
        .swapchains(&swapchains)
        .image_indices(&image_indices);

    unsafe {
        swapchain_loader.queue_present(present_queue, &present_info)?;
    }

    Ok(())
}
