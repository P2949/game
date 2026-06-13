use ash::{Entry, vk};
use std::ffi::{CStr, CString};

use crate::renderer::vertex::quad_vertices;
use crate::renderer::{buffer, debug, device, frame, pipeline, swapchain, texture};

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
    pub sprite_vertex_buffer: Option<buffer::Buffer>,
    pub test_texture: Option<texture::Texture>,
    pub texture_descriptor_set_layout: vk::DescriptorSetLayout,
    pub texture_descriptor_pool: vk::DescriptorPool,
    pub texture_descriptor_set: vk::DescriptorSet,
    pub upload_command_pool: vk::CommandPool,
    pub upload_fence: vk::Fence,
    pub swapchain: swapchain::Swapchain,
    pub swapchain_image_views: swapchain::SwapchainImageViews,
    pub sprite_pipeline: pipeline::GraphicsPipeline,
    pub image_render_finished: Vec<vk::Semaphore>,
    pub frames: Vec<frame::FrameData>,
    pub current_frame: usize,
    pub needs_swapchain_recreate: bool,
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
        let upload_command_pool = create_upload_command_pool(
            &logical_device.device,
            selected_device.queue_families.graphics,
        )?;
        let upload_fence = create_upload_fence(&logical_device.device)?;
        let mut allocator = buffer::create_allocator(
            instance.clone(),
            logical_device.device.clone(),
            selected_device.physical_device,
        )?;
        let sprite_vertex_buffer = create_quad_vertex_buffer(
            &logical_device.device,
            &mut allocator,
            logical_device.graphics_queue,
            upload_command_pool,
            upload_fence,
        )?;
        let test_texture = texture::Texture::from_path(
            &logical_device.device,
            &mut allocator,
            logical_device.graphics_queue,
            upload_command_pool,
            upload_fence,
            "assets/textures/test.png",
            "test texture",
        )?;
        let texture_descriptor_set_layout =
            texture::create_texture_descriptor_set_layout(&logical_device.device)?;
        let (texture_descriptor_pool, texture_descriptor_set) =
            texture::create_texture_descriptor_set(
                &logical_device.device,
                texture_descriptor_set_layout,
                &test_texture,
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
        let sprite_pipeline = pipeline::GraphicsPipeline::new_sprite(
            &logical_device.device,
            swapchain.format,
            texture_descriptor_set_layout,
        )?;
        let image_render_finished = create_image_render_finished_semaphores(
            &logical_device.device,
            swapchain.images.len(),
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
            allocator: Some(allocator),
            sprite_vertex_buffer: Some(sprite_vertex_buffer),
            test_texture: Some(test_texture),
            texture_descriptor_set_layout,
            texture_descriptor_pool,
            texture_descriptor_set,
            upload_command_pool,
            upload_fence,
            swapchain,
            swapchain_image_views,
            sprite_pipeline,
            image_render_finished,
            frames,
            current_frame: 0,
            needs_swapchain_recreate: false,
        })
    }

    pub fn request_swapchain_recreate(&mut self) {
        self.needs_swapchain_recreate = true;
    }

    pub fn recreate_swapchain(&mut self, window: &sdl3::video::Window) -> anyhow::Result<()> {
        let (width, height) = window.size_in_pixels();
        if width == 0 || height == 0 {
            self.needs_swapchain_recreate = true;
            return Ok(());
        }

        let Some(logical_device) = self.logical_device.as_ref() else {
            anyhow::bail!("cannot recreate swapchain after logical device has been destroyed");
        };

        let device = &logical_device.device;

        unsafe {
            device.device_wait_idle()?;
        }

        let old_swapchain = self.swapchain.handle;
        let new_swapchain = swapchain::Swapchain::new(
            &self.instance,
            device,
            self.physical_device,
            &self.surface.loader,
            self.surface.handle,
            self.queue_families,
            (width, height),
            old_swapchain,
        )?;
        let new_swapchain_image_views = swapchain::SwapchainImageViews::new(
            device,
            &new_swapchain.images,
            new_swapchain.format,
        )?;
        let new_sprite_pipeline = pipeline::GraphicsPipeline::new_sprite(
            device,
            new_swapchain.format,
            self.texture_descriptor_set_layout,
        )?;
        let new_image_render_finished =
            create_image_render_finished_semaphores(device, new_swapchain.images.len())?;

        unsafe {
            for &semaphore in &self.image_render_finished {
                device.destroy_semaphore(semaphore, None);
            }

            self.sprite_pipeline.destroy(device);
            self.swapchain_image_views.destroy(device);
            self.swapchain.destroy();
        }

        self.swapchain = new_swapchain;
        self.swapchain_image_views = new_swapchain_image_views;
        self.sprite_pipeline = new_sprite_pipeline;
        self.image_render_finished = new_image_render_finished;
        self.needs_swapchain_recreate = false;

        log::info!("recreated swapchain for drawable size {width}x{height}");

        Ok(())
    }

    pub fn render(&mut self, window: &sdl3::video::Window, t: f32) -> anyhow::Result<()> {
        if self.needs_swapchain_recreate {
            self.recreate_swapchain(window)?;

            if self.needs_swapchain_recreate {
                return Ok(());
            }
        }

        let Some(logical_device) = self.logical_device.as_ref() else {
            anyhow::bail!("cannot render after logical device has been destroyed");
        };

        let device = &logical_device.device;
        let frame = &self.frames[self.current_frame];
        let sprite_vertex_buffer = self
            .sprite_vertex_buffer
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("sprite vertex buffer has been destroyed"))?;

        unsafe {
            device.wait_for_fences(std::slice::from_ref(&frame.in_flight), true, u64::MAX)?;

            let (image_index, suboptimal) = match self.swapchain.loader.acquire_next_image(
                self.swapchain.handle,
                u64::MAX,
                frame.image_available,
                vk::Fence::null(),
            ) {
                Ok(result) => result,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.needs_swapchain_recreate = true;
                    self.recreate_swapchain(window)?;
                    return Ok(());
                }
                Err(err) => return Err(err.into()),
            };

            let mut recreate_after_present = false;
            if suboptimal {
                recreate_after_present = true;
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
                self.sprite_pipeline.layout,
                self.sprite_pipeline.pipeline,
                sprite_vertex_buffer.handle,
                self.texture_descriptor_set,
                t,
            )?;

            let present_suboptimal = match submit_clear_frame(
                device,
                logical_device.graphics_queue,
                logical_device.present_queue,
                &self.swapchain.loader,
                self.swapchain.handle,
                frame,
                render_finished,
                image_index,
            ) {
                Ok(suboptimal) => suboptimal,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.current_frame = (self.current_frame + 1) % frame::MAX_FRAMES_IN_FLIGHT;
                    self.needs_swapchain_recreate = true;
                    self.recreate_swapchain(window)?;
                    return Ok(());
                }
                Err(err) => return Err(err.into()),
            };

            recreate_after_present |= present_suboptimal;

            self.current_frame = (self.current_frame + 1) % frame::MAX_FRAMES_IN_FLIGHT;

            if recreate_after_present {
                self.needs_swapchain_recreate = true;
            }
        }

        if self.needs_swapchain_recreate {
            self.recreate_swapchain(window)?;
        }

        Ok(())
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        unsafe {
            if let Some(logical_device) = self.logical_device.as_ref() {
                let _ = logical_device.device.device_wait_idle();

                self.sprite_pipeline.destroy(&logical_device.device);

                logical_device
                    .device
                    .destroy_descriptor_pool(self.texture_descriptor_pool, None);

                if let (Some(texture), Some(allocator)) =
                    (self.test_texture.take(), self.allocator.as_mut())
                {
                    texture.destroy(&logical_device.device, allocator);
                }

                if let (Some(vertex_buffer), Some(allocator)) =
                    (self.sprite_vertex_buffer.take(), self.allocator.as_mut())
                {
                    vertex_buffer.destroy(&logical_device.device, allocator);
                }

                logical_device
                    .device
                    .destroy_descriptor_set_layout(self.texture_descriptor_set_layout, None);

                logical_device.device.destroy_fence(self.upload_fence, None);
                logical_device
                    .device
                    .destroy_command_pool(self.upload_command_pool, None);

                for &semaphore in &self.image_render_finished {
                    logical_device.device.destroy_semaphore(semaphore, None);
                }

                for frame in &self.frames {
                    frame.destroy(&logical_device.device);
                }

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

fn create_upload_command_pool(
    device: &ash::Device,
    queue_family: u32,
) -> anyhow::Result<vk::CommandPool> {
    let pool_info = vk::CommandPoolCreateInfo::default()
        .queue_family_index(queue_family)
        .flags(vk::CommandPoolCreateFlags::TRANSIENT);

    let command_pool = unsafe { device.create_command_pool(&pool_info, None)? };
    log::info!("created upload command pool");
    Ok(command_pool)
}

fn create_upload_fence(device: &ash::Device) -> anyhow::Result<vk::Fence> {
    let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
    let fence = unsafe { device.create_fence(&fence_info, None)? };
    log::info!("created upload fence");
    Ok(fence)
}

fn create_image_render_finished_semaphores(
    device: &ash::Device,
    image_count: usize,
) -> anyhow::Result<Vec<vk::Semaphore>> {
    let semaphore_info = vk::SemaphoreCreateInfo::default();
    let mut semaphores = Vec::with_capacity(image_count);

    for _ in 0..image_count {
        semaphores.push(unsafe { device.create_semaphore(&semaphore_info, None)? });
    }

    Ok(semaphores)
}

fn create_quad_vertex_buffer(
    device: &ash::Device,
    allocator: &mut gpu_allocator::vulkan::Allocator,
    queue: vk::Queue,
    upload_pool: vk::CommandPool,
    upload_fence: vk::Fence,
) -> anyhow::Result<buffer::Buffer> {
    let vertices = quad_vertices(-0.75, -0.75, 1.5, 1.5);

    buffer::upload_buffer(
        device,
        allocator,
        queue,
        upload_pool,
        upload_fence,
        &vertices,
        vk::BufferUsageFlags::VERTEX_BUFFER,
        "sprite quad vertices",
    )
}

unsafe fn record_clear_commands(
    device: &ash::Device,
    cmd: vk::CommandBuffer,
    image: vk::Image,
    image_view: vk::ImageView,
    extent: vk::Extent2D,
    sprite_pipeline_layout: vk::PipelineLayout,
    sprite_pipeline: vk::Pipeline,
    sprite_vertex_buffer: vk::Buffer,
    texture_descriptor_set: vk::DescriptorSet,
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

        device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, sprite_pipeline);

        let descriptor_sets = [texture_descriptor_set];
        device.cmd_bind_descriptor_sets(
            cmd,
            vk::PipelineBindPoint::GRAPHICS,
            sprite_pipeline_layout,
            0,
            &descriptor_sets,
            &[],
        );

        let view_proj = glam::Mat4::IDENTITY.to_cols_array();
        device.cmd_push_constants(
            cmd,
            sprite_pipeline_layout,
            vk::ShaderStageFlags::VERTEX,
            0,
            bytemuck::bytes_of(&view_proj),
        );

        let vertex_buffers = [sprite_vertex_buffer];
        let offsets = [0_u64];
        device.cmd_bind_vertex_buffers(cmd, 0, &vertex_buffers, &offsets);

        device.cmd_draw(cmd, 6, 1, 0, 0);

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
) -> Result<bool, vk::Result> {
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
        let suboptimal = swapchain_loader.queue_present(present_queue, &present_info)?;
        Ok(suboptimal)
    }
}
