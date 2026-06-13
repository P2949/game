use ash::{Entry, vk};
use gpu_allocator::MemoryLocation;
use std::ffi::{CStr, CString};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::renderer::sprite_batch::{SpriteBatch, SpriteBatchRange};
use crate::renderer::vertex::SpriteVertex;
use crate::renderer::{
    DrawCommands, FONT_TEXTURE_ID, SpriteDraw, TEST_TEXTURE_ID, buffer, debug, device, frame,
    pipeline, swapchain, text, texture,
};

const INITIAL_SPRITE_VERTEX_BUFFER_BYTES: vk::DeviceSize = 1024 * 1024;
// Rate-limit swapchain recreations as defense against a driver that reports
// suboptimal/out-of-date every frame. User-driven resizes are already debounced
// in the platform layer before a recreate request reaches the context, so no
// additional extent-stabilization wait is needed here.
const MIN_SWAPCHAIN_RECREATE_INTERVAL: Duration = Duration::from_millis(1000);
const UI_TEXT_LAYER: i16 = 1000;

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
    pub test_texture: Option<texture::Texture>,
    pub font_texture: Option<texture::Texture>,
    pub font_atlas: text::FontAtlas,
    pub texture_descriptor_set_layout: vk::DescriptorSetLayout,
    pub texture_descriptor_pool: vk::DescriptorPool,
    pub texture_descriptor_set: vk::DescriptorSet,
    pub font_descriptor_pool: vk::DescriptorPool,
    pub font_descriptor_set: vk::DescriptorSet,
    pub upload_command_pool: vk::CommandPool,
    pub upload_fence: vk::Fence,
    pub swapchain: swapchain::Swapchain,
    pub swapchain_image_views: swapchain::SwapchainImageViews,
    pub sprite_pipeline: pipeline::GraphicsPipeline,
    pub image_render_finished: Vec<vk::Semaphore>,
    pub frames: Vec<frame::FrameData>,
    pub current_frame: usize,
    pub needs_swapchain_recreate: bool,
    last_swapchain_recreate: Option<Instant>,
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
            if !validation_layer_available(&entry)? {
                anyhow::bail!(
                    "debug build requested {}, but the Vulkan validation layer is not installed",
                    debug::VALIDATION_LAYER.to_string_lossy()
                );
            }
            vec![debug::VALIDATION_LAYER]
        } else {
            vec![]
        };
        let layer_ptrs: Vec<*const i8> = layer_names.iter().map(|layer| layer.as_ptr()).collect();

        let validation_enables = [vk::ValidationFeatureEnableEXT::SYNCHRONIZATION_VALIDATION];
        let mut validation_features =
            vk::ValidationFeaturesEXT::default().enabled_validation_features(&validation_enables);

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
        let (test_texture, font_texture, font_atlas) = {
            let mut texture_upload = texture::TextureUpload {
                device: &logical_device.device,
                allocator: &mut allocator,
                queue: logical_device.graphics_queue,
                upload_pool: upload_command_pool,
                upload_fence,
            };

            let test_texture = texture::Texture::from_path(
                &mut texture_upload,
                asset_path("assets/textures/test.png"),
                texture::TextureColorSpace::SrgbColor,
                "test texture",
            )?;
            let font_atlas_image = text::build_ascii_atlas(
                asset_path("assets/fonts/DejaVuSans.ttf"),
                FONT_TEXTURE_ID,
            )?;
            let font_texture = texture::Texture::from_rgba8(
                &mut texture_upload,
                font_atlas_image.width,
                font_atlas_image.height,
                &font_atlas_image.pixels,
                texture::TextureColorSpace::LinearData,
                "font atlas",
            )?;

            (test_texture, font_texture, font_atlas_image.atlas)
        };
        let texture_descriptor_set_layout =
            texture::create_texture_descriptor_set_layout(&logical_device.device)?;
        let (texture_descriptor_pool, texture_descriptor_set) =
            texture::create_texture_descriptor_set(
                &logical_device.device,
                texture_descriptor_set_layout,
                &test_texture,
            )?;
        let (font_descriptor_pool, font_descriptor_set) = texture::create_texture_descriptor_set(
            &logical_device.device,
            texture_descriptor_set_layout,
            &font_texture,
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
        let dynamic_sprite_buffers = (0..frame::MAX_FRAMES_IN_FLIGHT).map(|_| None).collect();

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
            dynamic_sprite_buffers,
            world_sprite_batch: SpriteBatch::new(),
            ui_sprite_batch: SpriteBatch::new(),
            sprite_vertices: Vec::new(),
            scratch_batch_ranges: Vec::new(),
            world_draw_ranges: Vec::new(),
            ui_draw_ranges: Vec::new(),
            test_texture: Some(test_texture),
            font_texture: Some(font_texture),
            font_atlas,
            texture_descriptor_set_layout,
            texture_descriptor_pool,
            texture_descriptor_set,
            font_descriptor_pool,
            font_descriptor_set,
            upload_command_pool,
            upload_fence,
            swapchain,
            swapchain_image_views,
            sprite_pipeline,
            image_render_finished,
            frames,
            current_frame: 0,
            needs_swapchain_recreate: false,
            last_swapchain_recreate: None,
        })
    }

    pub fn request_swapchain_recreate(&mut self) {
        self.needs_swapchain_recreate = true;
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
            &self.surface.loader,
            self.physical_device,
            self.surface.handle,
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
        let format_changed = new_swapchain.format != self.swapchain.format;
        let new_swapchain_image_views = match swapchain::SwapchainImageViews::new(
            device,
            &new_swapchain.images,
            new_swapchain.format,
        ) {
            Ok(views) => views,
            Err(err) => {
                unsafe {
                    new_swapchain.destroy();
                }
                return Err(err);
            }
        };
        let new_sprite_pipeline = if format_changed {
            match pipeline::GraphicsPipeline::new_sprite(
                device,
                new_swapchain.format,
                self.texture_descriptor_set_layout,
            ) {
                Ok(pipeline) => Some(pipeline),
                Err(err) => {
                    unsafe {
                        new_swapchain_image_views.destroy(device);
                        new_swapchain.destroy();
                    }
                    return Err(err);
                }
            }
        } else {
            None
        };
        let new_image_render_finished =
            match create_image_render_finished_semaphores(device, new_swapchain.images.len()) {
                Ok(semaphores) => semaphores,
                Err(err) => {
                    unsafe {
                        if let Some(new_sprite_pipeline) = &new_sprite_pipeline {
                            new_sprite_pipeline.destroy(device);
                        }
                        new_swapchain_image_views.destroy(device);
                        new_swapchain.destroy();
                    }
                    return Err(err);
                }
            };

        unsafe {
            for &semaphore in &self.image_render_finished {
                device.destroy_semaphore(semaphore, None);
            }

            if let Some(new_sprite_pipeline) = new_sprite_pipeline {
                self.sprite_pipeline.destroy(device);
                self.sprite_pipeline = new_sprite_pipeline;
            }
            self.swapchain_image_views.destroy(device);
            self.swapchain.destroy();
        }

        self.swapchain = new_swapchain;
        self.swapchain_image_views = new_swapchain_image_views;
        self.image_render_finished = new_image_render_finished;
        self.needs_swapchain_recreate = false;
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
        if self.needs_swapchain_recreate {
            if self.swapchain_extent_matches_window(window)? {
                self.needs_swapchain_recreate = false;
            } else if !self.swapchain_recreate_ready(window)? {
                self.clear_sprite_batches();
                return Ok(());
            } else {
                self.recreate_swapchain(window)?;

                if self.needs_swapchain_recreate {
                    self.clear_sprite_batches();
                    return Ok(());
                }
            }
        }

        let Some(logical_device) = self.logical_device.as_ref() else {
            anyhow::bail!("cannot render after logical device has been destroyed");
        };
        let device = logical_device.device.clone();
        let graphics_queue = logical_device.graphics_queue;
        let present_queue = logical_device.present_queue;

        let frame_index = self.current_frame;
        let frame = &self.frames[frame_index];
        let command_buffer = frame.command_buffer;
        let image_available = frame.image_available;
        let in_flight = frame.in_flight;

        // Assemble sprite geometry into reusable scratch buffers before touching
        // the GPU, overlapping this CPU work with the previous frame's fence
        // wait. World and UI sprites pack into one shared vertex buffer, with
        // each batch's ranges carrying absolute offsets into it.
        self.sprite_vertices.clear();
        self.world_draw_ranges.clear();
        self.ui_draw_ranges.clear();
        let test_set = self.texture_descriptor_set;
        let font_set = self.font_descriptor_set;

        self.scratch_batch_ranges.clear();
        self.world_sprite_batch
            .build_into(&mut self.sprite_vertices, &mut self.scratch_batch_ranges);
        resolve_draw_ranges(
            &self.scratch_batch_ranges,
            &mut self.world_draw_ranges,
            test_set,
            font_set,
        )?;

        self.scratch_batch_ranges.clear();
        self.ui_sprite_batch
            .build_into(&mut self.sprite_vertices, &mut self.scratch_batch_ranges);
        resolve_draw_ranges(
            &self.scratch_batch_ranges,
            &mut self.ui_draw_ranges,
            test_set,
            font_set,
        )?;

        self.clear_sprite_batches();

        unsafe {
            device.wait_for_fences(std::slice::from_ref(&in_flight), true, u64::MAX)?;

            let acquire_result = self.swapchain.loader.acquire_next_image(
                self.swapchain.handle,
                u64::MAX,
                image_available,
                vk::Fence::null(),
            );

            let (image_index, suboptimal) = match acquire_result {
                Ok(result) => result,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.request_swapchain_recreate();
                    return Ok(());
                }
                Err(err) => return Err(err.into()),
            };

            let mut recreate_after_present =
                suboptimal && !self.swapchain_extent_matches_window(window)?;

            // Overwriting this frame's vertex buffer is only safe now that the
            // fence wait above has guaranteed the GPU finished its previous use.
            let allocator = self
                .allocator
                .as_mut()
                .ok_or_else(|| anyhow::anyhow!("allocator has been destroyed"))?;
            let sprite_vertex_buffer = upload_sprite_vertices(
                &device,
                allocator,
                &mut self.dynamic_sprite_buffers[frame_index],
                &self.sprite_vertices,
            )?;

            device.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())?;

            let image_index_usize = image_index as usize;
            let render_finished = self.image_render_finished[image_index_usize];
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

            let present_suboptimal = match submit_result {
                Ok(suboptimal) => suboptimal,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.current_frame = (frame_index + 1) % frame::MAX_FRAMES_IN_FLIGHT;
                    self.request_swapchain_recreate();
                    return Ok(());
                }
                Err(err) => return Err(err.into()),
            };

            if present_suboptimal && !self.swapchain_extent_matches_window(window)? {
                recreate_after_present = true;
            }

            self.current_frame = (frame_index + 1) % frame::MAX_FRAMES_IN_FLIGHT;

            if recreate_after_present {
                self.needs_swapchain_recreate = true;
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

                self.sprite_pipeline.destroy(&logical_device.device);

                logical_device
                    .device
                    .destroy_descriptor_pool(self.texture_descriptor_pool, None);
                logical_device
                    .device
                    .destroy_descriptor_pool(self.font_descriptor_pool, None);

                if let (Some(texture), Some(allocator)) =
                    (self.test_texture.take(), self.allocator.as_mut())
                {
                    texture.destroy(&logical_device.device, allocator);
                }

                if let (Some(texture), Some(allocator)) =
                    (self.font_texture.take(), self.allocator.as_mut())
                {
                    texture.destroy(&logical_device.device, allocator);
                }

                if let Some(allocator) = self.allocator.as_mut() {
                    for vertex_buffer in &mut self.dynamic_sprite_buffers {
                        if let Some(vertex_buffer) = vertex_buffer.take() {
                            vertex_buffer.destroy(&logical_device.device, allocator);
                        }
                    }
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
        match unsafe { device.create_semaphore(&semaphore_info, None) } {
            Ok(semaphore) => semaphores.push(semaphore),
            Err(err) => {
                for &semaphore in &semaphores {
                    unsafe {
                        device.destroy_semaphore(semaphore, None);
                    }
                }
                return Err(err.into());
            }
        }
    }

    Ok(semaphores)
}

struct RenderSpriteRange {
    descriptor_set: vk::DescriptorSet,
    first_vertex: u32,
    vertex_count: u32,
}

struct RenderSpriteBatch<'a> {
    projection: [f32; 16],
    ranges: &'a [RenderSpriteRange],
}

fn duration_ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

/// Resolves each batch range's texture id to its descriptor set, appending the
/// GPU-ready draw ranges to `out`. `out` is appended to (never cleared) so the
/// caller controls buffer reuse across frames.
fn resolve_draw_ranges(
    batch_ranges: &[SpriteBatchRange],
    out: &mut Vec<RenderSpriteRange>,
    test_descriptor_set: vk::DescriptorSet,
    font_descriptor_set: vk::DescriptorSet,
) -> anyhow::Result<()> {
    out.reserve(batch_ranges.len());
    for range in batch_ranges {
        let descriptor_set = if range.texture == TEST_TEXTURE_ID {
            test_descriptor_set
        } else if range.texture == FONT_TEXTURE_ID {
            font_descriptor_set
        } else {
            anyhow::bail!("unknown texture id {:?}", range.texture);
        };

        out.push(RenderSpriteRange {
            descriptor_set,
            first_vertex: range.first_vertex,
            vertex_count: range.vertex_count,
        });
    }

    Ok(())
}

/// Uploads `vertices` into this frame's dynamic vertex buffer, growing (and
/// reallocating) it only when the existing capacity is too small. Returns the
/// buffer handle to bind, or `None` when there is nothing to draw.
fn upload_sprite_vertices(
    device: &ash::Device,
    allocator: &mut gpu_allocator::vulkan::Allocator,
    buffer_slot: &mut Option<buffer::Buffer>,
    vertices: &[SpriteVertex],
) -> anyhow::Result<Option<vk::Buffer>> {
    if vertices.is_empty() {
        return Ok(None);
    }

    let required_bytes = std::mem::size_of_val(vertices) as vk::DeviceSize;
    let should_recreate = match buffer_slot {
        Some(buffer) => buffer.size < required_bytes,
        None => true,
    };

    if should_recreate {
        if let Some(old_buffer) = buffer_slot.take() {
            unsafe {
                old_buffer.destroy(device, allocator);
            }
        }

        let capacity = sprite_vertex_buffer_capacity(required_bytes);
        let buffer = buffer::Buffer::new(
            device,
            allocator,
            capacity,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            MemoryLocation::CpuToGpu,
            "dynamic sprite vertex buffer",
        )?;
        *buffer_slot = Some(buffer);
    }

    let buffer = buffer_slot
        .as_mut()
        .expect("dynamic sprite buffer exists after creation");
    buffer.copy_from_slice(vertices)?;
    Ok(Some(buffer.handle))
}

#[allow(clippy::too_many_arguments)]
unsafe fn record_sprite_commands(
    device: &ash::Device,
    cmd: vk::CommandBuffer,
    image: vk::Image,
    image_view: vk::ImageView,
    extent: vk::Extent2D,
    sprite_pipeline_layout: vk::PipelineLayout,
    sprite_pipeline: vk::Pipeline,
    sprite_vertex_buffer: Option<vk::Buffer>,
    render_batches: &[RenderSpriteBatch<'_>],
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
            float32: [0.02, 0.02, 0.04, 1.0],
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

        if let Some(sprite_vertex_buffer) = sprite_vertex_buffer {
            let vertex_buffers = [sprite_vertex_buffer];
            let offsets = [0_u64];
            device.cmd_bind_vertex_buffers(cmd, 0, &vertex_buffers, &offsets);

            for batch in render_batches {
                if batch.ranges.is_empty() {
                    continue;
                }

                device.cmd_push_constants(
                    cmd,
                    sprite_pipeline_layout,
                    vk::ShaderStageFlags::VERTEX,
                    0,
                    bytemuck::bytes_of(&batch.projection),
                );

                for range in batch.ranges {
                    let descriptor_sets = [range.descriptor_set];
                    device.cmd_bind_descriptor_sets(
                        cmd,
                        vk::PipelineBindPoint::GRAPHICS,
                        sprite_pipeline_layout,
                        0,
                        &descriptor_sets,
                        &[],
                    );
                    device.cmd_draw(cmd, range.vertex_count, 1, range.first_vertex, 0);
                }
            }
        }

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

#[allow(clippy::too_many_arguments)]
unsafe fn submit_frame(
    device: &ash::Device,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    swapchain_loader: &ash::khr::swapchain::Device,
    swapchain: vk::SwapchainKHR,
    image_available: vk::Semaphore,
    command_buffer: vk::CommandBuffer,
    in_flight: vk::Fence,
    render_finished: vk::Semaphore,
    image_index: u32,
) -> Result<bool, vk::Result> {
    let wait_info = vk::SemaphoreSubmitInfo::default()
        .semaphore(image_available)
        .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT);

    let cmd_info = vk::CommandBufferSubmitInfo::default().command_buffer(command_buffer);

    let signal_info = vk::SemaphoreSubmitInfo::default()
        .semaphore(render_finished)
        .stage_mask(vk::PipelineStageFlags2::ALL_GRAPHICS);

    let submit_info = vk::SubmitInfo2::default()
        .wait_semaphore_infos(std::slice::from_ref(&wait_info))
        .command_buffer_infos(std::slice::from_ref(&cmd_info))
        .signal_semaphore_infos(std::slice::from_ref(&signal_info));

    unsafe {
        device.reset_fences(std::slice::from_ref(&in_flight))?;
        device.queue_submit2(
            graphics_queue,
            std::slice::from_ref(&submit_info),
            in_flight,
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

fn asset_path(relative: &str) -> PathBuf {
    // Prefer assets shipped next to the executable (the installed/distributed
    // layout). Fall back to the crate manifest dir so `cargo run` works from a
    // source checkout where assets live in the repo root.
    if let Ok(exe) = std::env::current_exe()
        && let Some(exe_dir) = exe.parent()
    {
        let candidate = exe_dir.join(relative);
        if candidate.exists() {
            return candidate;
        }
    }

    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn sprite_vertex_buffer_capacity(required_bytes: vk::DeviceSize) -> vk::DeviceSize {
    let mut capacity = INITIAL_SPRITE_VERTEX_BUFFER_BYTES;
    while capacity < required_bytes {
        capacity *= 2;
    }
    capacity
}

fn ui_projection(extent: vk::Extent2D) -> glam::Mat4 {
    glam::Mat4::orthographic_rh(
        0.0,
        extent.width as f32,
        0.0,
        extent.height as f32,
        -1.0,
        1.0,
    )
}

fn validation_layer_available(entry: &Entry) -> anyhow::Result<bool> {
    let layers = unsafe { entry.enumerate_instance_layer_properties()? };
    Ok(layers.iter().any(|layer| {
        let name = unsafe { CStr::from_ptr(layer.layer_name.as_ptr()) };
        name == debug::VALIDATION_LAYER
    }))
}
