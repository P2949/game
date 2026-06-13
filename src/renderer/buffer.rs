use ash::vk;
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{
    Allocation, AllocationCreateDesc, AllocationScheme, Allocator, AllocatorCreateDesc,
};

pub fn create_allocator(
    instance: ash::Instance,
    device: ash::Device,
    physical_device: ash::vk::PhysicalDevice,
) -> anyhow::Result<Allocator> {
    Ok(Allocator::new(&AllocatorCreateDesc {
        instance,
        device,
        physical_device,
        debug_settings: Default::default(),
        buffer_device_address: false,
        allocation_sizes: Default::default(),
    })?)
}

pub unsafe fn immediate_submit<F: FnOnce(vk::CommandBuffer)>(
    device: &ash::Device,
    queue: vk::Queue,
    command_pool: vk::CommandPool,
    fence: vk::Fence,
    f: F,
) -> anyhow::Result<()> {
    unsafe {
        device.reset_fences(std::slice::from_ref(&fence))?;
        device.reset_command_pool(command_pool, vk::CommandPoolResetFlags::empty())?;
    }

    let alloc_info = vk::CommandBufferAllocateInfo::default()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);

    let command_buffer = unsafe { device.allocate_command_buffers(&alloc_info)?[0] };
    let result = (|| -> anyhow::Result<()> {
        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            device.begin_command_buffer(command_buffer, &begin_info)?;
        }
        f(command_buffer);
        unsafe {
            device.end_command_buffer(command_buffer)?;
        }

        let cmd_info = vk::CommandBufferSubmitInfo::default().command_buffer(command_buffer);
        let submit_info =
            vk::SubmitInfo2::default().command_buffer_infos(std::slice::from_ref(&cmd_info));

        unsafe {
            device.queue_submit2(queue, std::slice::from_ref(&submit_info), fence)?;
            device.wait_for_fences(std::slice::from_ref(&fence), true, u64::MAX)?;
        }

        Ok(())
    })();

    unsafe {
        device.free_command_buffers(command_pool, std::slice::from_ref(&command_buffer));
    }

    result
}

pub struct Buffer {
    pub handle: vk::Buffer,
    pub allocation: Option<Allocation>,
    pub size: vk::DeviceSize,
}

impl Buffer {
    pub fn new(
        device: &ash::Device,
        allocator: &mut Allocator,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        location: MemoryLocation,
        name: &str,
    ) -> anyhow::Result<Self> {
        let info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let handle = unsafe { device.create_buffer(&info, None)? };
        let requirements = unsafe { device.get_buffer_memory_requirements(handle) };

        let allocation = match allocator.allocate(&AllocationCreateDesc {
            name,
            requirements,
            location,
            linear: true,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        }) {
            Ok(allocation) => allocation,
            Err(err) => {
                unsafe {
                    device.destroy_buffer(handle, None);
                }
                return Err(err.into());
            }
        };

        if let Err(err) =
            unsafe { device.bind_buffer_memory(handle, allocation.memory(), allocation.offset()) }
        {
            allocator
                .free(allocation)
                .expect("free unbound buffer allocation");
            unsafe {
                device.destroy_buffer(handle, None);
            }
            return Err(err.into());
        }

        log::info!("created buffer '{name}' ({size} bytes)");

        Ok(Self {
            handle,
            allocation: Some(allocation),
            size,
        })
    }

    pub fn copy_from_bytes(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
        let byte_count = bytes.len() as vk::DeviceSize;
        if byte_count > self.size {
            anyhow::bail!(
                "buffer copy is {byte_count} bytes, but buffer capacity is {} bytes",
                self.size
            );
        }

        let allocation = self
            .allocation
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("buffer allocation has already been taken"))?;
        let mapped = allocation
            .mapped_ptr()
            .ok_or_else(|| anyhow::anyhow!("buffer allocation is not CPU mapped"))?;

        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), mapped.as_ptr() as *mut u8, bytes.len());
        }

        Ok(())
    }

    pub fn copy_from_slice<T: bytemuck::Pod>(&mut self, data: &[T]) -> anyhow::Result<()> {
        self.copy_from_bytes(bytemuck::cast_slice(data))
    }

    pub unsafe fn destroy(mut self, device: &ash::Device, allocator: &mut Allocator) {
        if let Some(allocation) = self.allocation.take() {
            allocator.free(allocation).expect("free buffer allocation");
        }

        unsafe {
            device.destroy_buffer(self.handle, None);
        }
    }
}
