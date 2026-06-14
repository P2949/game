use std::cell::Cell;

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

pub unsafe fn immediate_submit<F: FnOnce(vk::CommandBuffer) -> anyhow::Result<()>>(
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

    // Free the command buffer on every exit, including an unwind out of `f`.
    // Without this guard a panicking closure would skip the cleanup and leak the
    // buffer until the pool is reset or destroyed. The free is suppressed only in
    // the catastrophic device-lost case below, where the buffer may still be
    // referenced by an in-flight submission we can no longer drain.
    struct FreeOnDrop<'a> {
        device: &'a ash::Device,
        pool: vk::CommandPool,
        command_buffer: vk::CommandBuffer,
        should_free: Cell<bool>,
    }
    impl Drop for FreeOnDrop<'_> {
        fn drop(&mut self) {
            if !self.should_free.get() {
                return;
            }
            unsafe {
                self.device
                    .free_command_buffers(self.pool, std::slice::from_ref(&self.command_buffer));
            }
        }
    }
    let free_guard = FreeOnDrop {
        device,
        pool: command_pool,
        command_buffer,
        should_free: Cell::new(true),
    };

    (|| -> anyhow::Result<()> {
        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            device.begin_command_buffer(command_buffer, &begin_info)?;
        }
        f(command_buffer)?;
        unsafe {
            device.end_command_buffer(command_buffer)?;
        }

        let cmd_info = vk::CommandBufferSubmitInfo::default().command_buffer(command_buffer);
        let submit_info =
            vk::SubmitInfo2::default().command_buffer_infos(std::slice::from_ref(&cmd_info));

        unsafe {
            device.queue_submit2(queue, std::slice::from_ref(&submit_info), fence)?;
        }

        // The submission succeeded, so the GPU may reference the command buffer
        // until `fence` signals. If the wait itself fails (typically device-lost),
        // try to drain the device first so the `FreeOnDrop` guard can still safely
        // free the command buffer. If even `device_wait_idle` fails, the buffer may
        // remain part of an in-flight submission, so suppress the free entirely:
        // leaking one upload command buffer during fatal teardown is safer than
        // freeing a buffer the GPU might still read.
        if let Err(err) =
            unsafe { device.wait_for_fences(std::slice::from_ref(&fence), true, u64::MAX) }
        {
            if unsafe { device.device_wait_idle() }.is_err() {
                free_guard.should_free.set(false);
            }
            return Err(err.into());
        }

        Ok(())
    })()
}

/// Validates a requested buffer size before any Vulkan call. Vulkan forbids
/// zero-sized buffers, so reject that case up front with a named error.
fn validate_buffer_size(size: vk::DeviceSize, name: &str) -> anyhow::Result<()> {
    if size == 0 {
        anyhow::bail!("cannot create zero-sized Vulkan buffer '{name}'");
    }
    Ok(())
}

/// Frees an allocation, logging (never panicking) on failure. These frees run on
/// cleanup/teardown paths — including from `Drop`, possibly while another panic is
/// already unwinding — so a failed free must not become a second panic. A free
/// failure here means leaked GPU memory, which is logged loudly instead.
pub(crate) fn free_allocation(allocator: &mut Allocator, allocation: Allocation, name: &str) {
    if let Err(err) = allocator.free(allocation) {
        log::error!("failed to free allocation '{name}': {err}");
    }
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
        // Vulkan requires a nonzero buffer size; creating a zero-sized buffer is
        // a validation error. Reject it here with a clear message instead.
        validate_buffer_size(size, name)?;

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
            // Destroy the buffer before freeing its backing memory, matching
            // `Buffer::destroy`'s "object before its memory" teardown order.
            unsafe {
                device.destroy_buffer(handle, None);
            }
            free_allocation(allocator, allocation, name);
            return Err(err.into());
        }

        // `copy_from_bytes` writes CpuToGpu buffers via a plain mapped memcpy with
        // no flush, which is only correct on HOST_COHERENT memory. Enforce it at
        // creation so a non-coherent allocation fails fast here instead of later at
        // copy time (or, worse, silently presenting stale data on a future port).
        if location == MemoryLocation::CpuToGpu
            && !allocation
                .memory_properties()
                .contains(vk::MemoryPropertyFlags::HOST_COHERENT)
        {
            // Destroy the buffer before freeing its backing memory, matching
            // `Buffer::destroy`'s "object before its memory" teardown order.
            unsafe {
                device.destroy_buffer(handle, None);
            }
            free_allocation(allocator, allocation, name);
            anyhow::bail!(
                "buffer '{name}' requested CpuToGpu memory but the allocation is not \
                 HOST_COHERENT; mapped writes would require explicit flushing"
            );
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

        // This is a plain memcpy with no explicit flush, which is only correct on
        // HOST_COHERENT memory. gpu-allocator's CpuToGpu allocations are
        // host-visible and coherent on the desktop drivers this renderer targets;
        // enforce it so a future port to non-coherent memory fails loudly here
        // instead of silently presenting stale vertex data.
        if !allocation
            .memory_properties()
            .contains(vk::MemoryPropertyFlags::HOST_COHERENT)
        {
            anyhow::bail!(
                "buffer copy target is not HOST_COHERENT; mapped writes require an explicit \
                 vkFlushMappedMemoryRanges before GPU use"
            );
        }

        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), mapped.as_ptr() as *mut u8, bytes.len());
        }

        Ok(())
    }

    pub fn copy_from_slice<T: bytemuck::Pod>(&mut self, data: &[T]) -> anyhow::Result<()> {
        self.copy_from_bytes(bytemuck::cast_slice(data))
    }

    pub unsafe fn destroy(mut self, device: &ash::Device, allocator: &mut Allocator) {
        // Destroy the Vulkan buffer before freeing its backing memory, matching
        // `Texture::destroy` and the conventional "object before its memory"
        // teardown order. Freeing bound memory first is permitted by the spec only
        // under conditions that are awkward to reason about and confuses
        // validation/debug tooling, so destroy the handle first.
        unsafe {
            device.destroy_buffer(self.handle, None);
        }

        if let Some(allocation) = self.allocation.take() {
            free_allocation(allocator, allocation, "buffer");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::validate_buffer_size;

    #[test]
    fn zero_sized_buffer_is_rejected() {
        assert!(validate_buffer_size(0, "test buffer").is_err());
    }

    #[test]
    fn nonzero_sized_buffer_is_accepted() {
        assert!(validate_buffer_size(1, "test buffer").is_ok());
        assert!(validate_buffer_size(1024, "test buffer").is_ok());
    }
}
