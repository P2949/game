//! A registry mapping [`TextureId`]s to their GPU texture and descriptor set.
//!
//! Replaces hard-coded per-texture branching in the render path: textures are
//! registered once at startup (or later) and looked up by id while recording
//! draws, so adding a texture never means editing a `match`/`if` in the
//! command recorder. Ids are assigned sequentially in registration order; the
//! built-in [`crate::TEST_TEXTURE_ID`] / [`crate::FONT_TEXTURE_ID`]
//! are simply the first two registrations.

use ash::vk;
use gpu_allocator::vulkan::Allocator;

use crate::TextureId;
use crate::owned::OwnedDescriptorPool;
use crate::texture::{self, Texture};

trait Cleanup<T> {
    fn cleanup(&mut self, value: T);
}

struct CleanupGuard<T, C: Cleanup<T>> {
    value: Option<T>,
    cleanup: C,
}

impl<T, C: Cleanup<T>> CleanupGuard<T, C> {
    fn new(value: T, cleanup: C) -> Self {
        Self {
            value: Some(value),
            cleanup,
        }
    }

    fn value(&self) -> &T {
        self.value
            .as_ref()
            .expect("cleanup guard value exists until taken")
    }

    fn take(mut self) -> T {
        self.value
            .take()
            .expect("cleanup guard value exists until taken")
    }
}

impl<T, C: Cleanup<T>> Drop for CleanupGuard<T, C> {
    fn drop(&mut self) {
        if let Some(value) = self.value.take() {
            self.cleanup.cleanup(value);
        }
    }
}

struct PendingTextureCleanup<'a> {
    device: &'a ash::Device,
    allocator: &'a mut Allocator,
}

impl Cleanup<Texture> for PendingTextureCleanup<'_> {
    fn cleanup(&mut self, texture: Texture) {
        unsafe {
            texture.destroy(self.device, self.allocator);
        }
    }
}

struct PendingTexture<'a> {
    guard: CleanupGuard<Texture, PendingTextureCleanup<'a>>,
}

impl<'a> PendingTexture<'a> {
    fn new(texture: Texture, device: &'a ash::Device, allocator: &'a mut Allocator) -> Self {
        Self {
            guard: CleanupGuard::new(texture, PendingTextureCleanup { device, allocator }),
        }
    }

    fn texture(&self) -> &Texture {
        self.guard.value()
    }

    fn take(self) -> Texture {
        self.guard.take()
    }
}

struct TextureEntry {
    texture: Texture,
    // Owns the descriptor pool; dropping it destroys the pool (and frees
    // `descriptor_set`). Only read in `destroy`, where it is dropped before the
    // texture's image/view/sampler.
    descriptor_pool: OwnedDescriptorPool,
    descriptor_set: vk::DescriptorSet,
    #[allow(dead_code)]
    name: String,
}

/// Owns renderer textures and their descriptor pools/sets.
///
/// This type is not self-dropping because texture destruction requires access
/// to the Vulkan allocator. It must be destroyed explicitly with
/// [`TextureRegistry::destroy`], or held by [`TextureRegistryGuard`] while
/// `VulkanContext::new` is still fallible.
#[derive(Default)]
pub struct TextureRegistry {
    entries: Vec<TextureEntry>,
}

impl TextureRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a descriptor set for `texture` from `layout` and registers it,
    /// returning the assigned [`TextureId`].
    ///
    /// Private: the only public registration path is
    /// [`TextureRegistryGuard::create_and_register`], which fuses creation and
    /// registration so a freshly-built `Texture` is never held unguarded across a
    /// fallible step. This method takes an already-built `Texture` and is only
    /// safe to call from inside that fused path.
    fn register_texture(
        &mut self,
        device: &ash::Device,
        allocator: &mut Allocator,
        descriptor_set_layout: vk::DescriptorSetLayout,
        texture: Texture,
        name: impl Into<String>,
    ) -> anyhow::Result<TextureId> {
        let pending_texture = PendingTexture::new(texture, device, allocator);
        let (pool, descriptor_set) = texture::create_texture_descriptor_set(
            device,
            descriptor_set_layout,
            pending_texture.texture(),
        )?;
        let descriptor_pool = OwnedDescriptorPool::from_handle(device, pool);
        let texture = pending_texture.take();

        let id = TextureId(
            u32::try_from(self.entries.len())
                .map_err(|_| anyhow::anyhow!("texture registry exceeded u32::MAX entries"))?,
        );
        self.entries.push(TextureEntry {
            texture,
            descriptor_pool,
            descriptor_set,
            name: name.into(),
        });
        Ok(id)
    }

    /// Looks up the descriptor set bound for `id`, erroring on an unknown id.
    pub fn descriptor_set(&self, id: TextureId) -> anyhow::Result<vk::DescriptorSet> {
        self.entries
            .get(id.0 as usize)
            .map(|entry| entry.descriptor_set)
            .ok_or_else(|| anyhow::anyhow!("unknown texture id {id:?}"))
    }

    /// Replaces the image and descriptor resources behind an existing texture
    /// id. The id itself remains stable, so content-facing texture-handle
    /// mappings and already-built draw batches continue to resolve correctly.
    ///
    /// The caller must ensure the device is idle before calling this method.
    /// New GPU resources are fully created before the old entry is touched, so
    /// a decode/upload/descriptor failure leaves the current texture intact.
    pub fn replace_texture<F>(
        &mut self,
        device: &ash::Device,
        allocator: &mut Allocator,
        descriptor_set_layout: vk::DescriptorSetLayout,
        id: TextureId,
        name: impl Into<String>,
        make: F,
    ) -> anyhow::Result<()>
    where
        F: FnOnce(&ash::Device, &mut Allocator) -> anyhow::Result<Texture>,
    {
        let texture = make(device, allocator)?;
        let pending_texture = PendingTexture::new(texture, device, allocator);
        let (pool, descriptor_set) = texture::create_texture_descriptor_set(
            device,
            descriptor_set_layout,
            pending_texture.texture(),
        )?;
        let replacement = TextureEntry {
            texture: pending_texture.take(),
            descriptor_pool: OwnedDescriptorPool::from_handle(device, pool),
            descriptor_set,
            name: name.into(),
        };
        let entry = self
            .entries
            .get_mut(id.0 as usize)
            .ok_or_else(|| anyhow::anyhow!("unknown texture id {id:?}"))?;
        let old = std::mem::replace(entry, replacement);
        drop(old.descriptor_pool);
        unsafe {
            old.texture.destroy(device, allocator);
        }
        Ok(())
    }

    /// Destroys every registered texture and descriptor pool. Must be called
    /// while the logical device is still alive (the owned descriptor pools drop
    /// here, and texture teardown needs the device and allocator).
    ///
    /// # Safety
    ///
    /// `device` and `allocator` must outlive all registered textures and must
    /// match the objects used to create them. The device should be idle, or
    /// callers must otherwise guarantee no in-flight command buffer can access
    /// registered descriptor sets or texture resources.
    pub unsafe fn destroy(&mut self, device: &ash::Device, allocator: &mut Allocator) {
        for entry in self.entries.drain(..) {
            let TextureEntry {
                texture,
                descriptor_pool,
                ..
            } = entry;
            // Destroy the descriptor pool first so the descriptor set referencing
            // this texture's image view/sampler is gone before those resources
            // are. Teardown runs with the device idle, so the order isn't strictly
            // load-bearing, but it keeps the dependency direction clear.
            drop(descriptor_pool);
            unsafe {
                texture.destroy(device, allocator);
            }
        }
    }
}

impl Drop for TextureRegistry {
    fn drop(&mut self) {
        // The registry is not self-cleaning (texture teardown needs the external
        // allocator), so reaching `Drop` with live entries means an owner forgot
        // to call `destroy`/hold a `TextureRegistryGuard`. That would leak GPU
        // memory and descriptor pools; catch it in debug builds. After a correct
        // `destroy` (or guard cleanup) the entry vector is drained, so this holds.
        debug_assert!(
            self.entries.is_empty(),
            "TextureRegistry dropped without explicit destroy(): {} live texture(s) leaked",
            self.entries.len()
        );
    }
}

struct RegistryCleanup<'a> {
    device: &'a ash::Device,
    allocator: &'a mut Allocator,
}

impl Cleanup<TextureRegistry> for RegistryCleanup<'_> {
    fn cleanup(&mut self, mut registry: TextureRegistry) {
        unsafe {
            registry.destroy(self.device, self.allocator);
        }
    }
}

/// Owns a [`TextureRegistry`] while `VulkanContext::new` is still fallible.
///
/// `TextureRegistry` is not self-dropping because texture destruction needs the
/// external Vulkan allocator. This guard guarantees that registered textures are
/// explicitly destroyed if a later startup step fails before the registry is
/// moved into the finished context.
pub struct TextureRegistryGuard<'a> {
    registry: CleanupGuard<TextureRegistry, RegistryCleanup<'a>>,
}

impl<'a> TextureRegistryGuard<'a> {
    pub fn new(device: &'a ash::Device, allocator: &'a mut Allocator) -> Self {
        Self {
            registry: CleanupGuard::new(
                TextureRegistry::new(),
                RegistryCleanup { device, allocator },
            ),
        }
    }

    /// Creates a texture via `make` (which receives the guard's device and
    /// allocator) and registers it in one step. The texture is never observable
    /// as a plain value outside this call, so neither a creation failure nor a
    /// descriptor-set failure can leak it, and any texture already registered is
    /// owned by the guard (cleaned up on drop). This is the only registration
    /// entry point, which keeps every built-in texture failure-safe.
    pub fn create_and_register<F>(
        &mut self,
        descriptor_set_layout: vk::DescriptorSetLayout,
        name: impl Into<String>,
        make: F,
    ) -> anyhow::Result<TextureId>
    where
        F: FnOnce(&ash::Device, &mut Allocator) -> anyhow::Result<Texture>,
    {
        let CleanupGuard { value, cleanup } = &mut self.registry;
        let texture = make(cleanup.device, cleanup.allocator)?;
        value
            .as_mut()
            .expect("registry exists until finish")
            .register_texture(
                cleanup.device,
                cleanup.allocator,
                descriptor_set_layout,
                texture,
                name,
            )
    }

    pub fn finish(self) -> TextureRegistry {
        self.registry.take()
    }
}

#[cfg(test)]
mod tests {
    use super::{Cleanup, CleanupGuard, TextureRegistry};
    use crate::TextureId;
    use std::cell::Cell;
    use std::rc::Rc;

    struct CountCleanup {
        count: Rc<Cell<usize>>,
    }

    impl Cleanup<&'static str> for CountCleanup {
        fn cleanup(&mut self, _value: &'static str) {
            self.count.set(self.count.get() + 1);
        }
    }

    #[test]
    fn unknown_id_lookup_errors() {
        let registry = TextureRegistry::new();
        assert!(registry.descriptor_set(TextureId(0)).is_err());
        assert!(registry.descriptor_set(TextureId(7)).is_err());
    }

    #[test]
    fn cleanup_guard_runs_cleanup_when_unfinished() {
        let count = Rc::new(Cell::new(0));
        {
            let _guard = CleanupGuard::new(
                "pending",
                CountCleanup {
                    count: Rc::clone(&count),
                },
            );
        }

        assert_eq!(count.get(), 1);
    }

    #[test]
    fn cleanup_guard_take_prevents_cleanup() {
        let count = Rc::new(Cell::new(0));
        let value = {
            let guard = CleanupGuard::new(
                "finished",
                CountCleanup {
                    count: Rc::clone(&count),
                },
            );
            guard.take()
        };

        assert_eq!(value, "finished");
        assert_eq!(count.get(), 0);
    }
}
