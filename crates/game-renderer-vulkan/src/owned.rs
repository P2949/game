//! Small RAII wrappers around individual Vulkan device-child handles.
//!
//! Each wrapper owns a cloned [`ash::Device`] plus exactly one Vulkan object and
//! destroys it on `Drop`. Cloning `ash::Device` is cheap (it shares the loaded
//! function pointers), and giving each handle an owner makes two things fall out
//! for free:
//!
//! * **Failure-safe construction.** If a later `?` fails while building a larger
//!   object, any wrappers already created as locals are dropped automatically,
//!   so no half-built state leaks.
//! * **Failure-safe collections.** A `Vec<OwnedSemaphore>` that is only
//!   partially filled when an error occurs cleans up every element it does hold.
//!
//! These wrappers are intended only for handles whose destruction needs nothing
//! but the device. Resources whose teardown also needs the allocator (buffers,
//! textures) keep their explicit `destroy(device, allocator)` methods, because a
//! `Drop` impl cannot reach the externally-owned allocator.
//!
//! Ownership-ordering note: a [`crate::context::VulkanContext`] still
//! destroys the [`ash::Device`] itself, so any of these wrappers held by the
//! context must be dropped *before* that device is destroyed. The context's
//! `Drop` does this explicitly (clearing owned collections and taking owned
//! `Option` fields) before tearing the logical device down.

use ash::vk;

macro_rules! owned_handle {
    (
        $(#[$meta:meta])*
        $name:ident, $handle:ty, $destroy:ident
    ) => {
        $(#[$meta])*
        pub struct $name {
            device: ash::Device,
            handle: $handle,
        }

        impl $name {
            /// Adopts ownership of an already-created handle, cloning `device` so
            /// the handle is destroyed when this wrapper drops. Part of the
            /// reusable wrapper API; not every wrapper type uses it.
            #[allow(dead_code)]
            pub fn from_handle(device: &ash::Device, handle: $handle) -> Self {
                Self {
                    device: device.clone(),
                    handle,
                }
            }

            // Reusable accessor; a few wrapper types are only created, stored,
            // and dropped without their handle being read again.
            #[allow(dead_code)]
            pub fn handle(&self) -> $handle {
                self.handle
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                unsafe {
                    self.device.$destroy(self.handle, None);
                }
            }
        }
    };
}

owned_handle!(
    /// Owns a `vk::Semaphore`.
    OwnedSemaphore,
    vk::Semaphore,
    destroy_semaphore
);
owned_handle!(
    /// Owns a `vk::Fence`.
    OwnedFence,
    vk::Fence,
    destroy_fence
);
owned_handle!(
    /// Owns a `vk::CommandPool`. Destroying it also frees command buffers
    /// allocated from it.
    OwnedCommandPool,
    vk::CommandPool,
    destroy_command_pool
);
owned_handle!(
    /// Owns a `vk::DescriptorPool`. Destroying it also frees descriptor sets
    /// allocated from it.
    OwnedDescriptorPool,
    vk::DescriptorPool,
    destroy_descriptor_pool
);
owned_handle!(
    /// Owns a `vk::DescriptorSetLayout`.
    OwnedDescriptorSetLayout,
    vk::DescriptorSetLayout,
    destroy_descriptor_set_layout
);

impl OwnedSemaphore {
    pub fn new(device: &ash::Device) -> anyhow::Result<Self> {
        let info = vk::SemaphoreCreateInfo::default();
        let handle = unsafe { device.create_semaphore(&info, None)? };
        Ok(Self {
            device: device.clone(),
            handle,
        })
    }
}

impl OwnedFence {
    pub fn new(device: &ash::Device, signaled: bool) -> anyhow::Result<Self> {
        let flags = if signaled {
            vk::FenceCreateFlags::SIGNALED
        } else {
            vk::FenceCreateFlags::empty()
        };
        let info = vk::FenceCreateInfo::default().flags(flags);
        let handle = unsafe { device.create_fence(&info, None)? };
        Ok(Self {
            device: device.clone(),
            handle,
        })
    }
}

impl OwnedCommandPool {
    pub fn new(device: &ash::Device, info: &vk::CommandPoolCreateInfo) -> anyhow::Result<Self> {
        let handle = unsafe { device.create_command_pool(info, None)? };
        Ok(Self {
            device: device.clone(),
            handle,
        })
    }
}
