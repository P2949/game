# Vulkan Renderer Lifetime Map

This document tracks the Vulkan resources owned by the renderer and the intended
cleanup owner for each one. The project currently uses a mixed model:

- device-only handles use small RAII wrappers where practical;
- allocator-backed resources keep explicit `destroy(device, allocator)` methods;
- `VulkanContext::drop` coordinates shutdown order while the device and
  allocator are still alive.

The target direction is to keep all RAII owners intact during construction and
to use construction guards for explicitly-destroyed resources.

## Allocator/Device Lifetime Boundary

The renderer uses the roadmap's **Option A** for allocator-backed objects:

- Handles that need only an `ash::Device` to destroy themselves should own a
  cloned device and implement `Drop`.
- Resources that also need the `gpu_allocator::vulkan::Allocator` keep explicit
  `destroy(device, allocator)` methods.
- Any fallible construction path that owns allocator-backed resources must use a
  local cleanup guard or explicit error cleanup until ownership transfers into
  `VulkanContext`.

This keeps allocator ownership single-threaded and avoids adding a shared
allocator lock just to make `Drop` possible for textures and buffers.

| Resource type | Created in | Currently destroyed in | Depends on | Failure risk | Target owner |
| --- | --- | --- | --- | --- | --- |
| Vulkan entry | `instance::VulkanInstance::new` | `VulkanInstance::Drop` with the owning bundle | dynamic Vulkan loader | Low; Rust value only | `VulkanInstance` |
| Vulkan instance | `instance::VulkanInstance::new` | `VulkanInstance::Drop` via `instance.destroy_instance` | entry | Covered by RAII owner during and after context construction | `VulkanInstance::Drop` |
| Debug utils loader | `instance::VulkanInstance::new` | `VulkanInstance::Drop` with the owning bundle | entry, instance | Loader itself is Rust state; messenger depends on it | `VulkanInstance` |
| Debug messenger | `instance::VulkanInstance::new` | `VulkanInstance::Drop` via `destroy_debug_utils_messenger` | instance, debug utils loader | Covered by RAII owner during and after context construction | `VulkanInstance::Drop` |
| Surface loader | `Surface::new` | Stored in `Surface` | entry, instance | Rust state only | `Surface` |
| Surface handle | `Surface::new` | `Surface::Drop` via `destroy_surface` | instance | Covered by RAII owner during and after context construction | `Surface::Drop` |
| Logical device | `LogicalDevice::new` | Dropped from `VulkanContext::drop` after child resources | instance, physical device | Mostly safe once local variable exists; child handles must drop first | `LogicalDevice` plus explicit context drop order |
| Queues | `LogicalDevice::new` | Owned by logical device | logical device | No standalone destruction | `LogicalDevice` |
| Allocator | `buffer::create_allocator` | `VulkanContext::drop` after textures/buffers are destroyed | instance, device, physical device | Must outlive allocator-backed textures/buffers | `VulkanContext` explicit drop order |
| Upload command pool | `create_upload_command_pool`, adopted by `OwnedCommandPool` | `OwnedCommandPool::Drop`, forced before device teardown | logical device | Low after adoption; leak possible before adoption only | `OwnedCommandPool` |
| Upload fence | `create_upload_fence`, adopted by `OwnedFence` | `OwnedFence::Drop`, forced before device teardown | logical device | Low after adoption; leak possible before adoption only | `OwnedFence` |
| Texture descriptor set layout | `texture::create_texture_descriptor_set_layout`, adopted by `OwnedDescriptorSetLayout` | `OwnedDescriptorSetLayout::Drop`, forced before device teardown | logical device | Low after adoption | `OwnedDescriptorSetLayout` |
| Descriptor pools for textures | `texture::create_texture_descriptor_set`, adopted by `OwnedDescriptorPool` | `OwnedDescriptorPool::Drop` from `TextureRegistry::destroy` | logical device | Covered during context construction by `TextureRegistryGuard`; explicit destroy still required after context ownership transfer | `TextureRegistry` plus construction guard |
| Descriptor sets | allocated from texture descriptor pools | Freed when descriptor pool is destroyed | descriptor pool | No standalone destruction | `OwnedDescriptorPool` inside `TextureRegistry` |
| Texture images | `Texture::from_rgba8` / `Texture::from_path` | `Texture::destroy` called by `TextureRegistry::destroy` | logical device, allocator | `PendingTexture` cleans up registration failure; `TextureRegistryGuard` cleans up later construction failure | `TextureRegistry` with `PendingTexture` / registry construction guard |
| Texture image allocations | `Texture::from_rgba8` | `Texture::destroy` frees through allocator | allocator | Protected by pending/registry guards during construction; must be freed before allocator/device destruction | `TextureRegistry` with explicit destroy |
| Texture image views | `Texture::from_rgba8` | `Texture::destroy` | logical device | Protected by pending/registry guards during construction | `TextureRegistry` with explicit destroy |
| Texture samplers | `Texture::from_rgba8` | `Texture::destroy` | logical device | Protected by pending/registry guards during construction | `TextureRegistry` with explicit destroy |
| Staging buffers | `Texture::from_rgba8` | explicit `Buffer::destroy` in success and error paths | logical device, allocator | Local failure paths must destroy before returning | Local construction guards or explicit cleanup |
| Dynamic sprite buffers | `upload_sprite_vertices` / `Buffer::new` | `VulkanContext::drop` iterates and calls `Buffer::destroy` | logical device, allocator | Must be destroyed before allocator/device | `VulkanContext` explicit drop order |
| Swapchain | `Swapchain::new` | `Swapchain::Drop`; `VulkanContext` also calls idempotent `destroy()` before device teardown/replacement | instance, logical device, surface | Covered by RAII owner during construction; explicit early destroy preserves device drop order | `Swapchain::Drop` |
| Swapchain images | returned by swapchain | Owned by swapchain implementation | swapchain | No standalone destruction | `Swapchain` |
| Swapchain image views | `SwapchainImageViews::new` | `SwapchainImageViews::Drop`; `VulkanContext` also calls idempotent `destroy()` before device teardown/replacement | logical device, swapchain images | Covered by RAII owner during construction; explicit early destroy preserves image-view-before-swapchain order | `SwapchainImageViews::Drop` |
| Graphics pipeline layout | `GraphicsPipeline::new_sprite` | `GraphicsPipeline::Drop`; `VulkanContext` also calls idempotent `destroy()` before device teardown/replacement | logical device, descriptor set layout | Covered by RAII owner during construction; explicit early destroy preserves device drop order | `GraphicsPipeline::Drop` |
| Graphics pipeline | `GraphicsPipeline::new_sprite` | `GraphicsPipeline::Drop`; `VulkanContext` also calls idempotent `destroy()` before device teardown/replacement | logical device, pipeline layout, swapchain format | Covered by RAII owner during construction; explicit early destroy preserves pipeline-before-layout order | `GraphicsPipeline::Drop` |
| Shader modules | `create_sprite_shader_modules` | Destroyed immediately after pipeline creation attempt | logical device | Covered by local cleanup in constructor | Local construction cleanup |
| Per-image render-finished semaphores | `create_image_render_finished_semaphores` | `OwnedSemaphore::Drop`, vector cleared before device teardown | logical device | Low after adoption; partially-filled vectors clean themselves | `OwnedSemaphore` |
| Per-frame command pools | `FrameData::new`, adopted by `OwnedCommandPool` | `FrameData::Drop`, vector cleared before device teardown | logical device | Low after adoption | `FrameData` |
| Per-frame command buffers | allocated from per-frame command pool | Freed when command pool is destroyed | command pool | No standalone destruction | `OwnedCommandPool` inside `FrameData` |
| Per-frame image-available semaphores | `FrameData::new`, adopted by `OwnedSemaphore` | `FrameData::Drop` | logical device | Low after adoption | `FrameData` |
| Per-frame fences | `FrameData::new`, adopted by `OwnedFence` | `FrameData::Drop` | logical device | Low after adoption | `FrameData` |

## Required Drop Order

The safe destruction order is:

1. Wait for the logical device to become idle.
2. Destroy frame resources and per-image semaphores.
3. Destroy textures and dynamic buffers while the allocator is alive.
4. Destroy pipeline and pipeline layout.
5. Destroy swapchain image views.
6. Destroy swapchain.
7. Destroy upload command pool, upload fence, descriptor set layout, and texture descriptor pools.
8. Drop allocator.
9. Drop logical device.
10. Destroy surface.
11. Destroy debug messenger, then Vulkan instance.

`VulkanContext::drop` may remain the coordinator for resources requiring the
allocator, but resources that only need a device/instance should move toward
RAII `Drop` owners so constructor failure paths are automatically cleaned up.
