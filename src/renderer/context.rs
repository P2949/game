use ash::vk;
use std::time::{Duration, Instant};

use crate::renderer::commands::{
    RenderSpriteBatch, RenderSpriteRange, present_frame, record_sprite_commands,
    resolve_draw_ranges, submit_frame, ui_projection, upload_sprite_vertices,
};
use crate::renderer::owned::{
    OwnedCommandPool, OwnedDescriptorSetLayout, OwnedFence, OwnedSemaphore,
};
use crate::renderer::recreate::{
    SwapchainRecreateAction, SwapchainRecreateReason, request_soft_recreate,
    request_suboptimal_recreate, swapchain_recreate_action,
};
use crate::renderer::sprite_batch::{SpriteBatch, SpriteBatchRange};
use crate::renderer::texture_registry::{TextureRegistry, TextureRegistryGuard};
use crate::renderer::vertex::SpriteVertex;
use crate::renderer::{
    DrawCommands, SpriteDraw, assets, buffer, device, frame, instance, pipeline, swapchain, text,
    texture,
};

// Rate-limit soft swapchain recreations as defense against noisy request
// sources. User-driven resizes are already debounced in the platform layer, so
// keep them more responsive than SUBOPTIMAL driver feedback.
const MIN_RESIZE_SWAPCHAIN_RECREATE_INTERVAL: Duration = Duration::from_millis(350);
const MIN_SUBOPTIMAL_SWAPCHAIN_RECREATE_INTERVAL: Duration = Duration::from_millis(1000);
const FRAME_TIMING_LOG_INTERVAL: Duration = Duration::from_secs(1);
const UI_TEXT_LAYER: i16 = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderOutcome {
    Presented,
    Skipped,
}

/// Synchronization resources whose lifetime is tied to one swapchain generation.
///
/// Per-frame resources (`FrameData`) survive swapchain recreation: command
/// buffers, acquire semaphores, and frame fences can be reused for the next
/// generation after the device is idle. Present wait semaphores are different:
/// each one is associated with a swapchain image, so this bundle is recreated
/// alongside images/views whenever the swapchain generation changes.
struct SwapchainSync {
    render_finished_by_image: Vec<OwnedSemaphore>,
    image_in_flight_fences: Vec<Option<vk::Fence>>,
    generation: u64,
}

impl SwapchainSync {
    fn new(device: &ash::Device, image_count: usize, generation: u64) -> anyhow::Result<Self> {
        Ok(Self {
            render_finished_by_image: create_image_render_finished_semaphores(device, image_count)?,
            image_in_flight_fences: vec![None; image_count],
            generation,
        })
    }

    fn render_finished(&self, image_index: usize) -> vk::Semaphore {
        self.render_finished_by_image[image_index].handle()
    }

    fn in_flight_fence(&self, image_index: usize) -> Option<vk::Fence> {
        self.image_in_flight_fences[image_index]
    }

    fn mark_image_in_flight(&mut self, image_index: usize, fence: vk::Fence) {
        self.image_in_flight_fences[image_index] = Some(fence);
    }

    fn clear(&mut self) {
        self.render_finished_by_image.clear();
        self.image_in_flight_fences.clear();
    }
}

// All fields are private. The renderer's careful Vulkan destruction order (see
// `Drop` below and `docs/renderer-lifetime.md`) depends on no outside code
// mutating or replacing these; the public surface is `new`, `render`,
// `request_swapchain_recreate`, and the `DrawCommands` trait.
pub struct VulkanContext {
    // Declared before the instance so it drops first once Surface becomes RAII.
    surface: crate::renderer::surface::Surface,
    // Keep the Vulkan loader/instance/debug messenger alive for objects created
    // from them. This RAII owner must remain intact during construction.
    instance: instance::VulkanInstance,
    #[allow(dead_code)]
    physical_device: vk::PhysicalDevice,
    queue_families: device::QueueFamilies,
    logical_device: Option<device::LogicalDevice>,
    allocator: Option<gpu_allocator::vulkan::Allocator>,
    dynamic_sprite_buffers: Vec<Option<buffer::Buffer>>,
    world_sprite_batch: SpriteBatch,
    ui_sprite_batch: SpriteBatch,
    // Reused scratch buffers for sprite geometry assembly. Kept on the context
    // so steady-state rendering performs no per-frame heap allocation, even as
    // scene complexity grows; capacity only ever grows to the high-water mark.
    sprite_vertices: Vec<SpriteVertex>,
    scratch_batch_ranges: Vec<SpriteBatchRange>,
    world_draw_ranges: Vec<RenderSpriteRange>,
    ui_draw_ranges: Vec<RenderSpriteRange>,
    font_atlas: text::FontAtlas,
    // Device-child handles owned via RAII wrappers. They are `Option`/`Vec` so
    // `Drop` can release them (while the device is still alive) before the
    // logical device itself is destroyed. See `owned.rs`.
    texture_descriptor_set_layout: Option<OwnedDescriptorSetLayout>,
    // Owns every registered texture plus its descriptor set/pool; the render path
    // looks descriptor sets up by id instead of branching per texture.
    texture_registry: TextureRegistry,
    upload_command_pool: Option<OwnedCommandPool>,
    upload_fence: Option<OwnedFence>,
    swapchain: swapchain::Swapchain,
    swapchain_image_views: swapchain::SwapchainImageViews,
    sprite_pipeline: pipeline::GraphicsPipeline,
    swapchain_sync: SwapchainSync,
    frames: Vec<frame::FrameData>,
    current_frame: usize,
    swapchain_recreate_request: Option<SwapchainRecreateReason>,
    last_swapchain_recreate: Option<Instant>,
    last_frame_timing_log: Instant,
    frame_timing_logs: bool,
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
            logical_device.device(),
            create_upload_command_pool(
                logical_device.device(),
                selected_device.queue_families.graphics,
            )?,
        );
        let upload_fence = OwnedFence::from_handle(
            logical_device.device(),
            create_upload_fence(logical_device.device())?,
        );
        let mut allocator = buffer::create_allocator(
            instance.handle().clone(),
            logical_device.device().clone(),
            selected_device.physical_device,
        )?;
        let texture_descriptor_set_layout = OwnedDescriptorSetLayout::from_handle(
            logical_device.device(),
            texture::create_texture_descriptor_set_layout(logical_device.device())?,
        );
        // The guard owns every registered texture (and its descriptor pool) for
        // the remainder of construction. It is intentionally NOT `finish()`ed
        // here: it is kept alive across swapchain/pipeline/frame creation below so
        // that any `?` failure in those still-fallible steps drops the guard,
        // which destroys the registered textures while the device/allocator are
        // alive. `load_builtin_textures` registers the built-ins in id order, so
        // `TEST_TEXTURE_ID` / `FONT_TEXTURE_ID` keep resolving to the right entry.
        let mut texture_registry_guard =
            TextureRegistryGuard::new(logical_device.device(), &mut allocator);
        let font_atlas = assets::load_builtin_textures(
            &mut texture_registry_guard,
            texture_descriptor_set_layout.handle(),
            logical_device.graphics_queue(),
            upload_command_pool.handle(),
            upload_fence.handle(),
        )?;
        let swapchain = swapchain::Swapchain::new(
            instance.handle(),
            logical_device.device(),
            selected_device.physical_device,
            surface.loader(),
            surface.handle(),
            selected_device.queue_families,
            window.size_in_pixels(),
            vk::SwapchainKHR::null(),
        )?;
        let swapchain_image_views = swapchain::SwapchainImageViews::new(
            logical_device.device(),
            swapchain.images(),
            swapchain.format(),
        )?;
        let sprite_pipeline = pipeline::GraphicsPipeline::new_sprite(
            logical_device.device(),
            swapchain.format(),
            texture_descriptor_set_layout.handle(),
        )?;
        let swapchain_sync =
            SwapchainSync::new(logical_device.device(), swapchain.image_count(), 0)?;

        let frames: Vec<_> = (0..frame::MAX_FRAMES_IN_FLIGHT)
            .map(|_| {
                frame::FrameData::new(
                    logical_device.device(),
                    selected_device.queue_families.graphics,
                )
            })
            .collect::<anyhow::Result<_>>()?;
        let dynamic_sprite_buffers = (0..frame::MAX_FRAMES_IN_FLIGHT).map(|_| None).collect();

        // Every fallible construction step that could leak the registry has now
        // succeeded. Take the registry out of the guard (releasing its mutable
        // borrow of the allocator) so both can move into the context.
        let texture_registry = texture_registry_guard.finish();

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
            swapchain_sync,
            frames,
            current_frame: 0,
            swapchain_recreate_request: None,
            last_swapchain_recreate: None,
            last_frame_timing_log: Instant::now(),
            frame_timing_logs: frame_timing_logs_enabled(),
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

    fn request_suboptimal_swapchain_recreate(&mut self) {
        request_suboptimal_recreate(&mut self.swapchain_recreate_request);
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

    fn swapchain_recreate_ready(
        &self,
        window: &sdl3::video::Window,
        reason: SwapchainRecreateReason,
    ) -> anyhow::Result<bool> {
        // Only special-case a zero-size (minimized) window, which has no valid
        // extent to recreate for. Any nonzero size is recreatable: gating on a
        // minimum size would strand a small window with no swapchain forever.
        if self.desired_swapchain_extent(window)?.is_none() {
            return Ok(false);
        }

        let min_interval = recreate_rate_limit_for(reason);
        if let Some(last_recreate) = self.last_swapchain_recreate {
            if min_interval > Duration::ZERO
                && Instant::now().duration_since(last_recreate) < min_interval
            {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn recreate_swapchain(&mut self, window: &sdl3::video::Window) -> anyhow::Result<()> {
        let recreate_start = Instant::now();
        let (width, height) = window.size_in_pixels();
        if width == 0 || height == 0 {
            // Keep a request pending so we recreate once the window has a size
            // again, but never downgrade an already-pending hard request.
            self.swapchain_recreate_request
                .get_or_insert(SwapchainRecreateReason::Resize);
            return Ok(());
        }

        let Some(logical_device) = self.logical_device.as_ref() else {
            anyhow::bail!("cannot recreate swapchain after logical device has been destroyed");
        };

        let device = logical_device.device();

        unsafe {
            device.device_wait_idle()?;
        }

        let old_swapchain = self.swapchain.handle();
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
        let format_changed = new_swapchain.format() != self.swapchain.format();
        let new_swapchain_image_views = swapchain::SwapchainImageViews::new(
            device,
            new_swapchain.images(),
            new_swapchain.format(),
        )?;
        let new_sprite_pipeline = if format_changed {
            Some(pipeline::GraphicsPipeline::new_sprite(
                device,
                new_swapchain.format(),
                self.descriptor_set_layout(),
            )?)
        } else {
            None
        };
        let new_swapchain_sync = SwapchainSync::new(
            device,
            new_swapchain.image_count(),
            self.swapchain_sync.generation + 1,
        )?;

        if let Some(new_sprite_pipeline) = new_sprite_pipeline {
            self.sprite_pipeline.destroy();
            self.sprite_pipeline = new_sprite_pipeline;
        }

        self.swapchain_image_views.destroy();
        self.swapchain.destroy();

        self.swapchain = new_swapchain;
        self.swapchain_image_views = new_swapchain_image_views;
        // Replacing the swapchain sync bundle drops the previous present wait
        // semaphores here, which is safe because `device_wait_idle` above
        // guaranteed no frame or failed-present path can still be using them.
        self.swapchain_sync = new_swapchain_sync;
        self.swapchain_recreate_request = None;
        self.last_swapchain_recreate = Some(Instant::now());

        log::info!(
            "recreated swapchain generation {} for drawable size {width}x{height} in {:.3}ms",
            self.swapchain_sync.generation,
            duration_ms(recreate_start.elapsed())
        );

        Ok(())
    }

    pub fn render(
        &mut self,
        window: &sdl3::video::Window,
        camera: crate::game::camera::Camera2D,
    ) -> anyhow::Result<RenderOutcome> {
        if let Some(reason) = self.swapchain_recreate_request {
            let desired_extent = self.desired_swapchain_extent(window)?;
            let has_nonzero_extent = desired_extent.is_some();
            let extent_matches = desired_extent == Some(self.swapchain.extent());
            let soft_recreate_ready = self.swapchain_recreate_ready(window, reason)?;

            match swapchain_recreate_action(
                reason,
                has_nonzero_extent,
                extent_matches,
                soft_recreate_ready,
            ) {
                SwapchainRecreateAction::ClearRequest => {
                    self.swapchain_recreate_request = None;
                }
                SwapchainRecreateAction::SkipFrame => {
                    self.clear_sprite_batches();
                    return Ok(RenderOutcome::Skipped);
                }
                SwapchainRecreateAction::DeferAndRender => {
                    // Keep the request pending and continue into normal rendering.
                }
                SwapchainRecreateAction::Recreate => {
                    self.recreate_swapchain(window)?;

                    // A window that went zero-size during recreate leaves the
                    // request pending; skip this frame and retry next time.
                    if self.swapchain_recreate_request.is_some() {
                        self.clear_sprite_batches();
                        return Ok(RenderOutcome::Skipped);
                    }
                }
            }
        }

        let frame_start = Instant::now();

        let Some(logical_device) = self.logical_device.as_ref() else {
            anyhow::bail!("cannot render after logical device has been destroyed");
        };
        let device = logical_device.device().clone();
        let graphics_queue = logical_device.graphics_queue();
        let present_queue = logical_device.present_queue();

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
        let world_stats = self
            .world_sprite_batch
            .build_into(&mut self.sprite_vertices, &mut self.scratch_batch_ranges)?;
        resolve_draw_ranges(
            &self.scratch_batch_ranges,
            &mut self.world_draw_ranges,
            &self.texture_registry,
        )?;

        self.scratch_batch_ranges.clear();
        let ui_stats = self
            .ui_sprite_batch
            .build_into(&mut self.sprite_vertices, &mut self.scratch_batch_ranges)?;
        resolve_draw_ranges(
            &self.scratch_batch_ranges,
            &mut self.ui_draw_ranges,
            &self.texture_registry,
        )?;

        let dropped_invalid_sprites =
            world_stats.dropped_invalid_sprites + ui_stats.dropped_invalid_sprites;
        if dropped_invalid_sprites > 0 {
            log::debug!("dropped {dropped_invalid_sprites} invalid sprite draw(s)");
        }

        self.clear_sprite_batches();

        unsafe {
            // Frame/swapchain synchronization lifecycle:
            // 1. Wait for this frame's fence before reusing its command/buffer state.
            // 2. Acquire a swapchain image, signaling the frame-owned
            //    `image_available` semaphore.
            // 3. Wait for any fence associated with that image from an earlier
            //    submission.
            // 4. Record commands for the acquired image.
            // 5. Submit commands, waiting on `image_available`, signaling the
            //    image-owned `render_finished` semaphore, and signaling the frame
            //    fence.
            // 6. Present waits on `render_finished`.
            // 7. Advance the frame index. Swapchain-owned sync is recreated with
            //    the swapchain after out-of-date/suboptimal present paths.
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

            // Reset this frame's command buffer before acquiring an image. The
            // fence wait above guarantees the GPU finished the previous use of it,
            // so resetting here keeps all remaining post-acquire work to
            // record/submit/present — a fallible reset can no longer strand an
            // already-acquired image and its signaled acquire semaphore.
            device.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())?;

            let acquire_start = Instant::now();
            let acquire_result = self.swapchain.loader().acquire_next_image(
                self.swapchain.handle(),
                u64::MAX,
                image_available,
                vk::Fence::null(),
            );
            let acquire_duration = acquire_start.elapsed();

            let (image_index, suboptimal) = match acquire_result {
                Ok(result) => result,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.request_mandatory_swapchain_recreate();
                    return Ok(RenderOutcome::Skipped);
                }
                Err(err) => return Err(err.into()),
            };

            let mut recreate_after_present = suboptimal;

            let image_index_usize = image_index as usize;
            if let Some(image_fence) = self.swapchain_sync.in_flight_fence(image_index_usize) {
                device.wait_for_fences(std::slice::from_ref(&image_fence), true, u64::MAX)?;
            }

            let render_finished = self.swapchain_sync.render_finished(image_index_usize);
            let swapchain_extent = self.swapchain.extent();
            let world_projection = camera
                .view_projection(
                    swapchain_extent.width as f32,
                    swapchain_extent.height as f32,
                )
                .to_cols_array();
            let ui_projection = ui_projection(swapchain_extent).to_cols_array();
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

            let swapchain_image = self.swapchain.image(image_index_usize)?;
            let swapchain_image_view = self.swapchain_image_views.view(image_index_usize)?;

            let record_start = Instant::now();
            if let Err(err) = record_sprite_commands(
                &device,
                command_buffer,
                swapchain_image,
                swapchain_image_view,
                swapchain_extent,
                self.sprite_pipeline.layout(),
                self.sprite_pipeline.pipeline(),
                sprite_vertex_buffer,
                &render_batches,
            ) {
                // We already hold an acquired image and a signaled acquire
                // semaphore. Command recording failing here is a device-level
                // error and the app exits on it, so drain the device first to
                // keep teardown from racing any outstanding GPU work.
                let _ = device.device_wait_idle();
                return Err(err);
            }
            let record_duration = record_start.elapsed();

            let submit_start = Instant::now();
            if let Err(err) = submit_frame(
                &device,
                graphics_queue,
                image_available,
                command_buffer,
                in_flight,
                render_finished,
            ) {
                let _ = device.device_wait_idle();
                return Err(err.into());
            }
            self.swapchain_sync
                .mark_image_in_flight(image_index_usize, in_flight);
            let present_result = present_frame(
                self.swapchain.loader(),
                present_queue,
                self.swapchain.handle(),
                render_finished,
                image_index,
            );
            let submit_present_duration = submit_start.elapsed();

            let present_suboptimal = match present_result {
                Ok(suboptimal) => suboptimal,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.current_frame = (frame_index + 1) % frame::MAX_FRAMES_IN_FLIGHT;
                    self.request_mandatory_swapchain_recreate();
                    return Ok(RenderOutcome::Skipped);
                }
                Err(err) => return Err(err.into()),
            };

            if present_suboptimal {
                recreate_after_present = true;
            }

            self.current_frame = (frame_index + 1) % frame::MAX_FRAMES_IN_FLIGHT;

            if recreate_after_present {
                self.request_suboptimal_swapchain_recreate();
            }

            if self.frame_timing_logs
                && self.last_frame_timing_log.elapsed() >= FRAME_TIMING_LOG_INTERVAL
            {
                let total_duration = frame_start.elapsed();
                log::debug!(
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

        Ok(RenderOutcome::Presented)
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
        let stats = text::draw_text(
            &mut self.ui_sprite_batch,
            &self.font_atlas,
            text,
            pos,
            color,
            UI_TEXT_LAYER,
        );
        if stats.glyphs_dropped > 0 {
            log::debug!(
                "dropped {} invalid text glyph sprite(s)",
                stats.glyphs_dropped
            );
        }
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        unsafe {
            if let Some(logical_device) = self.logical_device.as_ref() {
                let _ = logical_device.device().device_wait_idle();

                self.sprite_pipeline.destroy();

                // Textures and buffers free both a Vulkan handle and an allocator
                // allocation, so they keep explicit destroy(device, allocator)
                // calls — a Drop impl cannot reach the externally-owned allocator.
                // The allocator is only `take()`n further down in this same Drop,
                // so it must still be present here; make that invariant explicit so
                // a future reordering that leaks GPU memory is loud rather than
                // silent. (We log rather than panic: panicking in Drop, possibly
                // mid-unwind, is worse than the leak it would report.)
                match self.allocator.as_mut() {
                    Some(allocator) => {
                        self.texture_registry
                            .destroy(logical_device.device(), allocator);

                        for vertex_buffer in &mut self.dynamic_sprite_buffers {
                            if let Some(vertex_buffer) = vertex_buffer.take() {
                                vertex_buffer.destroy(logical_device.device(), allocator);
                            }
                        }
                    }
                    None => log::error!(
                        "allocator already gone during VulkanContext::drop; \
                         texture and buffer GPU memory leaked"
                    ),
                }

                // Release every device-child RAII handle now, while the logical
                // device is still alive. Each owned wrapper destroys its handle on
                // drop; clearing the collections and taking the `Option`s forces
                // those drops to run here rather than after the device is gone.
                self.frames.clear();
                self.swapchain_sync.clear();
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

fn frame_timing_logs_enabled() -> bool {
    matches!(
        std::env::var("GAME_FRAME_TIMINGS").as_deref(),
        Ok("1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON")
    )
}

fn recreate_rate_limit_for(reason: SwapchainRecreateReason) -> Duration {
    match reason {
        SwapchainRecreateReason::Resize => MIN_RESIZE_SWAPCHAIN_RECREATE_INTERVAL,
        SwapchainRecreateReason::Suboptimal => MIN_SUBOPTIMAL_SWAPCHAIN_RECREATE_INTERVAL,
        SwapchainRecreateReason::SurfaceOutOfDate => Duration::ZERO,
    }
}

#[cfg(test)]
mod tests {
    use super::{SwapchainRecreateReason, recreate_rate_limit_for};
    use std::time::Duration;

    #[test]
    fn resize_recreate_rate_limit_is_shorter_than_suboptimal() {
        assert!(
            recreate_rate_limit_for(SwapchainRecreateReason::Resize)
                < recreate_rate_limit_for(SwapchainRecreateReason::Suboptimal)
        );
    }

    #[test]
    fn out_of_date_recreate_is_not_rate_limited() {
        assert_eq!(
            recreate_rate_limit_for(SwapchainRecreateReason::SurfaceOutOfDate),
            Duration::ZERO
        );
    }
}
