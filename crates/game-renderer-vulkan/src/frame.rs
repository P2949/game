use ash::vk;

use crate::owned::{OwnedCommandPool, OwnedFence, OwnedSemaphore};

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

/// Per-frame command and synchronization resources. Every owned field cleans
/// itself up on `Drop`, so construction is failure-safe (a failed `?` drops the
/// resources already created) and a partially-built `Vec<FrameData>` releases
/// every frame it managed to create.
pub struct FrameData {
    // Field order matters for Drop: the command pool is destroyed last among
    // these so its command buffer is not freed out from under anything, though
    // in practice all per-frame work is idle before a frame is dropped.
    command_buffer: vk::CommandBuffer,
    image_available: OwnedSemaphore,
    in_flight: OwnedFence,
    // Held only to own the pool: dropping it destroys the pool (and frees the
    // command buffer above). Never read after construction.
    #[allow(dead_code)]
    command_pool: OwnedCommandPool,
}

impl FrameData {
    pub fn new(device: &ash::Device, graphics_queue_family: u32) -> anyhow::Result<Self> {
        let pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(graphics_queue_family)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let command_pool = OwnedCommandPool::new(device, &pool_info)?;

        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool.handle())
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        // If any step below fails, `command_pool` (and any earlier wrapper) drops
        // here, freeing the pool and its command buffer.
        let command_buffer = unsafe { device.allocate_command_buffers(&alloc_info)?[0] };

        let image_available = OwnedSemaphore::new(device)?;
        let in_flight = OwnedFence::new(device, true)?;

        log::info!("created per-frame command and sync resources");

        Ok(Self {
            command_buffer,
            image_available,
            in_flight,
            command_pool,
        })
    }

    pub fn command_buffer(&self) -> vk::CommandBuffer {
        self.command_buffer
    }

    pub fn image_available(&self) -> vk::Semaphore {
        self.image_available.handle()
    }

    pub fn in_flight(&self) -> vk::Fence {
        self.in_flight.handle()
    }
}
