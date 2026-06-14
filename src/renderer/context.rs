use ash::vk;
use std::time::{Duration, Instant};

use crate::renderer::commands::{
    RenderSpriteBatch, RenderSpriteRange, record_sprite_commands, resolve_draw_ranges,
    submit_frame, ui_projection, upload_sprite_vertices,
};
use crate::renderer::owned::{
    OwnedCommandPool, OwnedDescriptorSetLayout, OwnedFence, OwnedSemaphore,
};
use crate::renderer::recreate::{
    SwapchainRecreateAction, SwapchainRecreateReason, request_soft_recreate,
    swapchain_recreate_action,
};
use crate::renderer::sprite_batch::{SpriteBatch, SpriteBatchRange};
use crate::renderer::texture_registry::{TextureRegistry, TextureRegistryGuard};
use crate::renderer::vertex::SpriteVertex;
use crate::renderer::{
    DrawCommands, FONT_TEXTURE_ID, SpriteDraw, TEST_TEXTURE_ID, assets, buffer, device, frame,
    instance, pipeline, swapchain, text, texture,
};

// Rate-limit swapchain recreations as defense against a driver that reports
// suboptimal/out-of-date every frame. User-driven resizes are already debounced
// in the platform layer before a recreate request reaches the context, so no
// additional extent-stabilization wait is needed here.
const MIN_SWAPCHAIN_RECREATE_INTERVAL: Duration = Duration::from_millis(1000);
const FRAME_TIMING_LOG_INTERVAL: Duration = Duration::from_secs(1);
const UI_TEXT_LAYER: i16 = 1000;

pub struct VulkanContext {
    // Declared before the instance so it drops first once Surface becomes RAII.
    pub surface: crate::renderer::surface::Surface,
    // Keep the Vulkan loader/instance/debug messenger alive for objects created
    // from them. This RAII owner must remain intact during construction.
    pub instance: instance::VulkanInstance,
    #[allow(dead_code)]
    pub physical_device: vk::PhysicalDevice,
    #[allow(dead_code)]
    pub queue_families: device::QueueFamilies,
    pub logical_device: Option<device::LogicalDevice>,
    pub allocator: Option<gpu_allocator::vulkan::Allocator>,
    pub dynamic_sprite_buffers: Vec<Option<buffer::Buffer>>,
    pub world_sprite_batch: SpriteBatch,
    pub ui_sprite_batch: SpriteBatch,
    // Reused scratch buffers for sprite geometry assembly. Kept on the context
    // so steady-state rendering performs no per-frame heap allocation, even as
    // scene complexity grows; capacity only ever grows to the high-water mark.
    sprite_vertices: Vec<SpriteVertex>,
    scratch_batch_ranges: Vec<SpriteBatchRange>,
    world_draw_ranges: Vec<RenderSpriteRange>,
    ui_draw_ranges: Vec<RenderSpriteRange>,
    pub font_atlas: text::FontAtlas,
    // Device-child handles owned via RAII wrappers. They are `Option`/`Vec` so
    // `Drop` can release them (while the device is still alive) before the
    // logical device itself is destroyed. See `owned.rs`.
    texture_descriptor_set_layout: Option<OwnedDescriptorSetLayout>,
    // Owns every registered texture plus its descriptor set/pool; the render path
    // looks descriptor sets up by id instead of branching per texture.
    texture_registry: TextureRegistry,
    upload_command_pool: Option<OwnedCommandPool>,
    upload_fence: Option<OwnedFence>,
    pub swapchain: swapchain::Swapchain,
    pub swapchain_image_views: swapchain::SwapchainImageViews,
    pub sprite_pipeline: pipeline::GraphicsPipeline,
    image_render_finished: Vec<OwnedSemaphore>,
    frames: Vec<frame::FrameData>,
    pub current_frame: usize,
    swapchain_recreate_request: Option<SwapchainRecreateReason>,
    last_swapchain_recreate: Option<Instant>,
    last_frame_timing_log: Instant,
}

impl VulkanContext {
    pub fn new(window: &sdl3::video::Window) -> anyhow::Result<Self> {
        let instance = instance::VulkanInstance::new(window)?;

        let surface =
            crate::renderer::surface::Surface::new(instance.entry(), instance.handle(), window)?;
        let selected_device =
            device::select_physical_device(instance.handle(), surface.loader(), surface.handle())?;
        let logical_device = device::LogicalDevice::new(
            instance.handle(),
            selected_device.physical_device,
            selected_device.queue_families,
        )?;
        // Adopt each raw handle into an owning RAII wrapper immediately, so any
        // `?` failure further down drops everything created so far (reverse
        // declaration order keeps these child resources destroyed before the
        // logical device local that owns the `VkDevice`).
        let upload_command_pool = OwnedCommandPool::from_handle(
            &logical_device.device,
            create_upload_command_pool(
                &logical_device.device,
                selected_device.queue_families.graphics,
            )?,
        );
        let upload_fence = OwnedFence::from_handle(
            &logical_device.device,
            create_upload_fence(&logical_device.device)?,
        );
        let mut allocator = buffer::create_allocator(
            instance.handle().clone(),
            logical_device.device.clone(),
            selected_device.physical_device,
        )?;
        let assets::RendererAssets {
            test_texture,
            font_texture,
            font_atlas,
        } = {
            let mut texture_upload = texture::TextureUpload {
                device: &logical_device.device,
                allocator: &mut allocator,
                queue: logical_device.graphics_queue,
                upload_pool: upload_command_pool.handle(),
                upload_fence: upload_fence.handle(),
            };
            assets::RendererAssets::load(&mut texture_upload)?
        };
        let texture_descriptor_set_layout = OwnedDescriptorSetLayout::from_handle(
            &logical_device.device,
            texture::create_texture_descriptor_set_layout(&logical_device.device)?,
        );
        // Register the built-in textures in the order their ids are defined, so
        // `TEST_TEXTURE_ID` / `FONT_TEXTURE_ID` keep resolving to the right entry.
        let mut texture_registry_guard =
            TextureRegistryGuard::new(&logical_device.device, &mut allocator);
        let test_id = texture_registry_guard.register_texture(
            texture_descriptor_set_layout.handle(),
            test_texture,
            "test texture",
        )?;
        let font_id = texture_registry_guard.register_texture(
            texture_descriptor_set_layout.handle(),
            font_texture,
            "font atlas",
        )?;
        assert_eq!(
            test_id, TEST_TEXTURE_ID,
            "built-in test texture must keep its stable TextureId"
        );
        assert_eq!(
            font_id, FONT_TEXTURE_ID,
            "built-in font texture must keep its stable TextureId"
        );
        let texture_registry = texture_registry_guard.finish();
        let swapchain = swapchain::Swapchain::new(
            instance.handle(),
            &logical_device.device,
            selected_device.physical_device,
            surface.loader(),
            surface.handle(),
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
            texture_descriptor_set_layout.handle(),
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
        let dynamic_sprite_buffers = (0..frame::MAX_FRAMES_IN_FLIGHT).map(|_| None).collect();

        Ok(Self {
            surface,
            instance,
            physical_device: selected_device.physical_device,
            queue_families: selected_device.queue_families,
            logical_device: Some(logical_device),
            allocator: Some(allocator),
            dynamic_sprite_buffers,
            world_sprite_batch: SpriteBatch::new(),
            ui_sprite_batch: SpriteBatch::new(),
            sprite_vertices: Vec::new(),
            scratch_batch_ranges: Vec::new(),
            world_draw_ranges: Vec::new(),
            ui_draw_ranges: Vec::new(),
            font_atlas,
            texture_descriptor_set_layout: Some(texture_descriptor_set_layout),
            texture_registry,
            upload_command_pool: Some(upload_command_pool),
            upload_fence: Some(upload_fence),
            swapchain,
            swapchain_image_views,
            sprite_pipeline,
            image_render_finished,
            frames,
            current_frame: 0,
            swapchain_recreate_request: None,
            last_swapchain_recreate: None,
            last_frame_timing_log: Instant::now(),
        })
    }

    /// Public entry point for callers outside the renderer (e.g. the main loop
    /// reacting to a window resize). These are always soft requests.
    pub fn request_swapchain_recreate(&mut self) {
        self.request_soft_swapchain_recreate();
    }

    fn request_soft_swapchain_recreate(&mut self) {
        request_soft_recreate(&mut self.swapchain_recreate_request);
    }

    fn request_mandatory_swapchain_recreate(&mut self) {
        self.swapchain_recreate_request = Some(SwapchainRecreateReason::SurfaceOutOfDate);
    }

    /// Handle of the texture descriptor set layout. Present for the whole life of
    /// the context (only cleared during `Drop`), so this never fails in practice.
    fn descriptor_set_layout(&self) -> vk::DescriptorSetLayout {
        self.texture_descriptor_set_layout
            .as_ref()
            .expect("texture descriptor set layout present until drop")
            .handle()
    }

    fn clear_sprite_batches(&mut self) {
        self.world_sprite_batch.clear();
        self.ui_sprite_batch.clear();
    }

    fn desired_swapchain_extent(
        &self,
        window: &sdl3::video::Window,
    ) -> anyhow::Result<Option<vk::Extent2D>> {
        let (width, height) = window.size_in_pixels();
        if width == 0 || height == 0 {
            return Ok(None);
        }

        let support = swapchain::query_swapchain_support(
            self.surface.loader(),
            self.physical_device,
            self.surface.handle(),
        )?;

        Ok(Some(swapchain::choose_extent(
            support.capabilities,
            (width, height),
        )))
    }

    fn swapchain_extent_matches_window(
        &self,
        window: &sdl3::video::Window,
    ) -> anyhow::Result<bool> {
        let Some(desired_extent) = self.desired_swapchain_extent(window)? else {
            return Ok(false);
        };

        Ok(desired_extent == self.swapchain.extent)
    }

    fn swapchain_recreate_ready(&self, window: &sdl3::video::Window) -> anyhow::Result<bool> {
        // Only special-case a zero-size (minimized) window, which has no valid
        // extent to recreate for. Any nonzero size is recreatable: gating on a
        // minimum size would strand a small window with no swapchain forever.
        if self.desired_swapchain_extent(window)?.is_none() {
            return Ok(false);
        }

        if let Some(last_recreate) = self.last_swapchain_recreate
            && Instant::now().duration_since(last_recreate) < MIN_SWAPCHAIN_RECREATE_INTERVAL
        {
            return Ok(false);
        }

        Ok(true)
    }

    pub fn recreate_swapchain(&mut self, window: &sdl3::video::Window) -> anyhow::Result<()> {
        let recreate_start = Instant::now();
        let (width, height) = window.size_in_pixels();
        if width == 0 || height == 0 {
            // Keep a request pending so we recreate once the window has a size
            // again, but never downgrade an already-pending hard request.
            self.swapchain_recreate_request
                .get_or_insert(SwapchainRecreateReason::ResizeOrSuboptimal);
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
            self.instance.handle(),
            device,
            self.physical_device,
            self.surface.loader(),
            self.surface.handle(),
            self.queue_families,
            (width, height),
            old_swapchain,
        )?;
        let format_changed = new_swapchain.format != self.swapchain.format;
        let new_swapchain_image_views = swapchain::SwapchainImageViews::new(
            device,
            &new_swapchain.images,
            new_swapchain.format,
        )?;
        let new_sprite_pipeline = if format_changed {
            Some(pipeline::GraphicsPipeline::new_sprite(
                device,
                new_swapchain.format,
                self.descriptor_set_layout(),
            )?)
        } else {
            None
        };
        let new_image_render_finished =
            create_image_render_finished_semaphores(device, new_swapchain.images.len())?;

        if let Some(new_sprite_pipeline) = new_sprite_pipeline {
            self.sprite_pipeline.destroy();
            self.sprite_pipeline = new_sprite_pipeline;
        }

        self.swapchain_image_views.destroy();
        self.swapchain.destroy();

        self.swapchain = new_swapchain;
        self.swapchain_image_views = new_swapchain_image_views;
        // Replacing the vector drops the previous owned semaphores here, which is
        // safe because `device_wait_idle` above guaranteed no frame is using them.
        self.image_render_finished = new_image_render_finished;
        self.swapchain_recreate_request = None;
        self.last_swapchain_recreate = Some(Instant::now());

        log::info!(
            "recreated swapchain for drawable size {width}x{height} in {:.3}ms",
            duration_ms(recreate_start.elapsed())
        );

        Ok(())
    }

    pub fn render(
        &mut self,
        window: &sdl3::video::Window,
        camera: crate::game::camera::Camera2D,
    ) -> anyhow::Result<()> {
        if let Some(reason) = self.swapchain_recreate_request {
            let desired_extent = self.desired_swapchain_extent(window)?;
            let has_nonzero_extent = desired_extent.is_some();
            let extent_matches = desired_extent == Some(self.swapchain.extent);
            let soft_recreate_ready = self.swapchain_recreate_ready(window)?;

            match swapchain_recreate_action(
                reason,
                has_nonzero_extent,
                extent_matches,
                soft_recreate_ready,
            ) {
                SwapchainRecreateAction::ClearRequest => {
                    self.swapchain_recreate_request = None;
                }
                SwapchainRecreateAction::Wait => {
                    self.clear_sprite_batches();
                    return Ok(());
                }
                SwapchainRecreateAction::Recreate => {
                    self.recreate_swapchain(window)?;

                    // A window that went zero-size during recreate leaves the
                    // request pending; skip this frame and retry next time.
                    if self.swapchain_recreate_request.is_some() {
                        self.clear_sprite_batches();
                        return Ok(());
                    }
                }
            }
        }

        let frame_start = Instant::now();

        let Some(logical_device) = self.logical_device.as_ref() else {
            anyhow::bail!("cannot render after logical device has been destroyed");
        };
        let device = logical_device.device.clone();
        let graphics_queue = logical_device.graphics_queue;
        let present_queue = logical_device.present_queue;

        let frame_index = self.current_frame;
        let frame = &self.frames[frame_index];
        let command_buffer = frame.command_buffer();
        let image_available = frame.image_available();
        let in_flight = frame.in_flight();

        // Assemble sprite geometry into reusable scratch buffers before touching
        // the GPU, overlapping this CPU work with the previous frame's fence
        // wait. World and UI sprites pack into one shared vertex buffer, with
        // each batch's ranges carrying absolute offsets into it.
        self.sprite_vertices.clear();
        self.world_draw_ranges.clear();
        self.ui_draw_ranges.clear();

        self.scratch_batch_ranges.clear();
        self.world_sprite_batch
            .build_into(&mut self.sprite_vertices, &mut self.scratch_batch_ranges);
        resolve_draw_ranges(
            &self.scratch_batch_ranges,
            &mut self.world_draw_ranges,
            &self.texture_registry,
        )?;

        self.scratch_batch_ranges.clear();
        self.ui_sprite_batch
            .build_into(&mut self.sprite_vertices, &mut self.scratch_batch_ranges);
        resolve_draw_ranges(
            &self.scratch_batch_ranges,
            &mut self.ui_draw_ranges,
            &self.texture_registry,
        )?;

        self.clear_sprite_batches();

        unsafe {
            let wait_start = Instant::now();
            device.wait_for_fences(std::slice::from_ref(&in_flight), true, u64::MAX)?;
            let wait_duration = wait_start.elapsed();

            // Upload this frame's vertices before acquiring a swapchain image, so a
            // fallible allocation/copy fails here rather than after we already hold
            // an acquired image and a signaled acquire semaphore (which would leave
            // the frame's sync in an awkward, half-used state). Overwriting the
            // buffer is safe now because the fence wait above — not the acquire —
            // is what guarantees the GPU finished this frame's previous use of it.
            let allocator = self
                .allocator
                .as_mut()
                .ok_or_else(|| anyhow::anyhow!("allocator has been destroyed"))?;
            let vertex_start = Instant::now();
            let sprite_vertex_buffer = upload_sprite_vertices(
                &device,
                allocator,
                &mut self.dynamic_sprite_buffers[frame_index],
                &self.sprite_vertices,
            )?;
            let vertex_duration = vertex_start.elapsed();

            let acquire_start = Instant::now();
            let acquire_result = self.swapchain.loader.acquire_next_image(
                self.swapchain.handle,
                u64::MAX,
                image_available,
                vk::Fence::null(),
            );
            let acquire_duration = acquire_start.elapsed();

            let (image_index, suboptimal) = match acquire_result {
                Ok(result) => result,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.request_mandatory_swapchain_recreate();
                    return Ok(());
                }
                Err(err) => return Err(err.into()),
            };

            let mut recreate_after_present =
                suboptimal && !self.swapchain_extent_matches_window(window)?;

            device.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())?;

            let image_index_usize = image_index as usize;
            let render_finished = self.image_render_finished[image_index_usize].handle();
            let world_projection = camera
                .view_projection(
                    self.swapchain.extent.width as f32,
                    self.swapchain.extent.height as f32,
                )
                .to_cols_array();
            let ui_projection = ui_projection(self.swapchain.extent).to_cols_array();
            let render_batches = [
                RenderSpriteBatch {
                    projection: world_projection,
                    ranges: &self.world_draw_ranges,
                },
                RenderSpriteBatch {
                    projection: ui_projection,
                    ranges: &self.ui_draw_ranges,
                },
            ];

            let record_start = Instant::now();
            record_sprite_commands(
                &device,
                command_buffer,
                self.swapchain.images[image_index_usize],
                self.swapchain_image_views.views[image_index_usize],
                self.swapchain.extent,
                self.sprite_pipeline.layout,
                self.sprite_pipeline.pipeline,
                sprite_vertex_buffer,
                &render_batches,
            )?;
            let record_duration = record_start.elapsed();

            let submit_start = Instant::now();
            let submit_result = submit_frame(
                &device,
                graphics_queue,
                present_queue,
                &self.swapchain.loader,
                self.swapchain.handle,
                image_available,
                command_buffer,
                in_flight,
                render_finished,
                image_index,
            );
            let submit_present_duration = submit_start.elapsed();

            let present_suboptimal = match submit_result {
                Ok(suboptimal) => suboptimal,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.current_frame = (frame_index + 1) % frame::MAX_FRAMES_IN_FLIGHT;
                    self.request_mandatory_swapchain_recreate();
                    return Ok(());
                }
                Err(err) => return Err(err.into()),
            };

            if present_suboptimal && !self.swapchain_extent_matches_window(window)? {
                recreate_after_present = true;
            }

            self.current_frame = (frame_index + 1) % frame::MAX_FRAMES_IN_FLIGHT;

            if recreate_after_present {
                self.request_soft_swapchain_recreate();
            }

            if self.last_frame_timing_log.elapsed() >= FRAME_TIMING_LOG_INTERVAL {
                let total_duration = frame_start.elapsed();
                log::info!(
                    "frame timings: total={:.3}ms fence={:.3}ms acquire={:.3}ms vertex={:.3}ms record={:.3}ms submit_present={:.3}ms",
                    duration_ms(total_duration),
                    duration_ms(wait_duration),
                    duration_ms(acquire_duration),
                    duration_ms(vertex_duration),
                    duration_ms(record_duration),
                    duration_ms(submit_present_duration),
                );
                self.last_frame_timing_log = Instant::now();
            }
        }

        Ok(())
    }
}

impl DrawCommands for VulkanContext {
    fn draw_world_sprite(&mut self, sprite: SpriteDraw) {
        self.world_sprite_batch.push(sprite);
    }

    fn draw_ui_sprite(&mut self, sprite: SpriteDraw) {
        self.ui_sprite_batch.push(sprite);
    }

    fn draw_ui_text(&mut self, text: &str, pos: glam::Vec2, color: glam::Vec4) {
        text::draw_text(
            &mut self.ui_sprite_batch,
            &self.font_atlas,
            text,
            pos,
            color,
            UI_TEXT_LAYER,
        );
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        unsafe {
            if let Some(logical_device) = self.logical_device.as_ref() {
                let _ = logical_device.device.device_wait_idle();

                self.sprite_pipeline.destroy();

                // Textures and buffers free both a Vulkan handle and an allocator
                // allocation, so they keep explicit destroy(device, allocator)
                // calls — a Drop impl cannot reach the externally-owned allocator.
                if let Some(allocator) = self.allocator.as_mut() {
                    self.texture_registry
                        .destroy(&logical_device.device, allocator);

                    for vertex_buffer in &mut self.dynamic_sprite_buffers {
                        if let Some(vertex_buffer) = vertex_buffer.take() {
                            vertex_buffer.destroy(&logical_device.device, allocator);
                        }
                    }
                }

                // Release every device-child RAII handle now, while the logical
                // device is still alive. Each owned wrapper destroys its handle on
                // drop; clearing the collections and taking the `Option`s forces
                // those drops to run here rather than after the device is gone.
                self.frames.clear();
                self.image_render_finished.clear();
                self.texture_descriptor_set_layout = None;
                self.upload_fence = None;
                self.upload_command_pool = None;

                self.swapchain_image_views.destroy();
            }

            if let Some(allocator) = self.allocator.take() {
                drop(allocator);
            }

            self.swapchain.destroy();

            if let Some(logical_device) = self.logical_device.take() {
                drop(logical_device);
            }
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
) -> anyhow::Result<Vec<OwnedSemaphore>> {
    // RAII makes the failure path trivial: if any semaphore creation fails, the
    // partially-filled vector drops, destroying every semaphore created so far.
    let mut semaphores = Vec::with_capacity(image_count);
    for _ in 0..image_count {
        semaphores.push(OwnedSemaphore::new(device)?);
    }
    Ok(semaphores)
}

fn duration_ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}
