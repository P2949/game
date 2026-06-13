use ash::vk;

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct FrameData {
    pub command_pool: vk::CommandPool,
    pub command_buffer: vk::CommandBuffer,
    pub image_available: vk::Semaphore,
    pub render_finished: vk::Semaphore,
    pub in_flight: vk::Fence,
}

impl FrameData {
    pub fn new(device: &ash::Device, graphics_queue_family: u32) -> anyhow::Result<Self> {
        let pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(graphics_queue_family)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        let command_pool = unsafe { device.create_command_pool(&pool_info, None)? };

        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffer = unsafe { device.allocate_command_buffers(&alloc_info)?[0] };

        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let image_available = unsafe { device.create_semaphore(&semaphore_info, None)? };
        let render_finished = unsafe { device.create_semaphore(&semaphore_info, None)? };

        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
        let in_flight = unsafe { device.create_fence(&fence_info, None)? };

        log::info!("created per-frame command and sync resources");

        Ok(Self {
            command_pool,
            command_buffer,
            image_available,
            render_finished,
            in_flight,
        })
    }

    pub unsafe fn destroy(&self, device: &ash::Device) {
        unsafe {
            device.destroy_fence(self.in_flight, None);
            device.destroy_semaphore(self.render_finished, None);
            device.destroy_semaphore(self.image_available, None);
            device.destroy_command_pool(self.command_pool, None);
        }
    }
}
