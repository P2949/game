//! Per-frame command recording, queue submission/presentation, and the
//! CPU-side draw-range / vertex-buffer preparation that feeds them. Split out of
//! `context.rs` so the context owns frame orchestration while the imperative
//! Vulkan command details live here.

use ash::vk;
use gpu_allocator::MemoryLocation;

use crate::renderer::sprite_batch::SpriteBatchRange;
use crate::renderer::texture_registry::TextureRegistry;
use crate::renderer::vertex::SpriteVertex;
use crate::renderer::{buffer, texture};

const INITIAL_SPRITE_VERTEX_BUFFER_BYTES: vk::DeviceSize = 1024 * 1024;

/// A run of vertices to draw with one descriptor set (one texture).
pub struct RenderSpriteRange {
    descriptor_set: vk::DescriptorSet,
    first_vertex: u32,
    vertex_count: u32,
}

/// A group of ranges sharing one projection (world space or UI space).
pub struct RenderSpriteBatch<'a> {
    pub projection: [f32; 16],
    pub ranges: &'a [RenderSpriteRange],
}

/// Resolves each batch range's texture id to its descriptor set via the registry,
/// appending the GPU-ready draw ranges to `out`. `out` is appended to (never
/// cleared) so the caller controls buffer reuse across frames.
pub fn resolve_draw_ranges(
    batch_ranges: &[SpriteBatchRange],
    out: &mut Vec<RenderSpriteRange>,
    textures: &TextureRegistry,
) -> anyhow::Result<()> {
    out.reserve(batch_ranges.len());
    for range in batch_ranges {
        out.push(RenderSpriteRange {
            descriptor_set: textures.descriptor_set(range.texture)?,
            first_vertex: range.first_vertex,
            vertex_count: range.vertex_count,
        });
    }

    Ok(())
}

/// Uploads `vertices` into this frame's dynamic vertex buffer, growing (and
/// reallocating) it only when the existing capacity is too small. Returns the
/// buffer handle to bind, or `None` when there is nothing to draw.
pub fn upload_sprite_vertices(
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
        Some(buffer) => buffer.size() < required_bytes,
        None => true,
    };

    if should_recreate {
        if let Some(old_buffer) = buffer_slot.take() {
            unsafe {
                old_buffer.destroy(device, allocator);
            }
        }

        let capacity = sprite_vertex_buffer_capacity(required_bytes)?;
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
    Ok(Some(buffer.handle()))
}

fn sprite_vertex_buffer_capacity(required_bytes: vk::DeviceSize) -> anyhow::Result<vk::DeviceSize> {
    let mut capacity = INITIAL_SPRITE_VERTEX_BUFFER_BYTES;
    while capacity < required_bytes {
        // Doubling can overflow for an absurd request. It cannot happen for the
        // current game (a frame's vertices are far below this), but check it so a
        // bad `required_bytes` errors out instead of wrapping to a tiny capacity.
        capacity = capacity.checked_mul(2).ok_or_else(|| {
            anyhow::anyhow!(
                "sprite vertex buffer capacity overflowed while growing to fit \
                 {required_bytes} bytes"
            )
        })?;
    }
    Ok(capacity)
}

pub fn ui_projection(extent: vk::Extent2D) -> glam::Mat4 {
    glam::Mat4::orthographic_rh(
        0.0,
        extent.width as f32,
        0.0,
        extent.height as f32,
        -1.0,
        1.0,
    )
}

#[allow(clippy::too_many_arguments)]
/// # Safety
///
/// `cmd` must be a valid primary command buffer allocated from `device`, `image`
/// and `image_view` must refer to the same swapchain image, and all pipeline,
/// layout, descriptor, and buffer handles referenced by `render_batches` must
/// remain valid for the duration of command recording and later submission.
pub unsafe fn record_sprite_commands(
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
        texture::transition_image(
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

        texture::transition_image(
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

/// Submits the recorded render command buffer for one frame.
///
/// # Safety
///
/// `image_available`, `render_finished`, `command_buffer`, and `in_flight` must
/// belong to `device`. The command buffer must have been recorded for the
/// swapchain image associated with the signaled acquire semaphore, and
/// `in_flight` must not already be associated with outstanding work.
///
/// If submission fails after the fence is reset, the fence may remain unsignaled.
/// Callers must treat that failure as fatal for the frame/device path and must
/// not attempt to wait on the same fence later as though no submission happened.
pub unsafe fn submit_frame(
    device: &ash::Device,
    graphics_queue: vk::Queue,
    image_available: vk::Semaphore,
    command_buffer: vk::CommandBuffer,
    in_flight: vk::Fence,
    render_finished: vk::Semaphore,
) -> Result<(), vk::Result> {
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

    Ok(())
}

/// Presents the rendered swapchain image after graphics submission.
///
/// # Safety
///
/// `render_finished` must be signaled by submitted work that wrote
/// `image_index` in `swapchain`; `swapchain` and `present_queue` must be valid
/// for `swapchain_loader`.
pub unsafe fn present_frame(
    swapchain_loader: &ash::khr::swapchain::Device,
    present_queue: vk::Queue,
    swapchain: vk::SwapchainKHR,
    render_finished: vk::Semaphore,
    image_index: u32,
) -> Result<bool, vk::Result> {
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

#[cfg(test)]
mod tests {
    use super::{INITIAL_SPRITE_VERTEX_BUFFER_BYTES, sprite_vertex_buffer_capacity};
    use ash::vk;

    #[test]
    fn capacity_keeps_initial_when_request_fits() {
        let cap = sprite_vertex_buffer_capacity(INITIAL_SPRITE_VERTEX_BUFFER_BYTES / 2).unwrap();
        assert_eq!(cap, INITIAL_SPRITE_VERTEX_BUFFER_BYTES);

        let cap = sprite_vertex_buffer_capacity(INITIAL_SPRITE_VERTEX_BUFFER_BYTES).unwrap();
        assert_eq!(cap, INITIAL_SPRITE_VERTEX_BUFFER_BYTES);
    }

    #[test]
    fn capacity_doubles_until_request_fits() {
        let cap = sprite_vertex_buffer_capacity(INITIAL_SPRITE_VERTEX_BUFFER_BYTES + 1).unwrap();
        assert_eq!(cap, INITIAL_SPRITE_VERTEX_BUFFER_BYTES * 2);

        let cap =
            sprite_vertex_buffer_capacity(INITIAL_SPRITE_VERTEX_BUFFER_BYTES * 4 + 1).unwrap();
        assert_eq!(cap, INITIAL_SPRITE_VERTEX_BUFFER_BYTES * 8);
    }

    #[test]
    fn capacity_overflow_is_rejected() {
        // A request near the top of the address space cannot be satisfied by
        // power-of-two growth, so it must error rather than wrap to a tiny size.
        assert!(sprite_vertex_buffer_capacity(vk::DeviceSize::MAX).is_err());
    }
}
