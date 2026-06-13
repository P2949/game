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

pub struct Buffer {
    pub handle: vk::Buffer,
    pub allocation: Option<Allocation>,
    #[allow(dead_code)]
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

        let allocation = allocator.allocate(&AllocationCreateDesc {
            name,
            requirements,
            location,
            linear: true,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        })?;

        unsafe {
            device.bind_buffer_memory(handle, allocation.memory(), allocation.offset())?;
        }

        log::info!("created buffer '{name}' ({size} bytes)");

        Ok(Self {
            handle,
            allocation: Some(allocation),
            size,
        })
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
