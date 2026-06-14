use ash::vk;
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator};
use std::path::Path;

use crate::renderer::buffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureColorSpace {
    SrgbColor,
    LinearData,
}

impl TextureColorSpace {
    fn format(self) -> vk::Format {
        match self {
            Self::SrgbColor => vk::Format::R8G8B8A8_SRGB,
            Self::LinearData => vk::Format::R8G8B8A8_UNORM,
        }
    }
}

/// Validates RGBA8 texture inputs before any Vulkan image is created. Rejects
/// zero extents, dimension arithmetic that would overflow `usize`, and pixel
/// buffers whose length does not match `width * height * 4`. Returns the
/// expected byte length on success.
fn validate_rgba8_texture(
    name: &str,
    width: u32,
    height: u32,
    pixels_len: usize,
) -> anyhow::Result<usize> {
    if width == 0 || height == 0 {
        anyhow::bail!("texture '{name}' has zero extent ({width}x{height})");
    }

    let expected_len = (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(|| {
            anyhow::anyhow!("texture '{name}' dimensions overflow ({width}x{height})")
        })?;

    if pixels_len != expected_len {
        anyhow::bail!("texture '{name}' has {pixels_len} bytes, expected {expected_len}");
    }

    Ok(expected_len)
}

pub struct Texture {
    pub image: vk::Image,
    pub allocation: Option<Allocation>,
    pub view: vk::ImageView,
    pub sampler: vk::Sampler,
}

pub struct TextureUpload<'a> {
    pub device: &'a ash::Device,
    pub allocator: &'a mut Allocator,
    pub queue: vk::Queue,
    pub upload_pool: vk::CommandPool,
    pub upload_fence: vk::Fence,
}

impl Texture {
    pub fn from_path(
        upload: &mut TextureUpload<'_>,
        path: impl AsRef<Path>,
        color_space: TextureColorSpace,
        name: &str,
    ) -> anyhow::Result<Self> {
        let rgba = image::open(path.as_ref())?.to_rgba8();
        let width = rgba.width();
        let height = rgba.height();
        let pixels = rgba.into_raw();
        Self::from_rgba8(upload, width, height, &pixels, color_space, name)
    }

    pub fn from_rgba8(
        upload: &mut TextureUpload<'_>,
        width: u32,
        height: u32,
        pixels: &[u8],
        color_space: TextureColorSpace,
        name: &str,
    ) -> anyhow::Result<Self> {
        validate_rgba8_texture(name, width, height, pixels.len())?;

        let image_size = pixels.len() as vk::DeviceSize;
        let format = color_space.format();

        let staging_name = format!("{name} texture staging");
        let mut staging = buffer::Buffer::new(
            upload.device,
            upload.allocator,
            image_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            MemoryLocation::CpuToGpu,
            &staging_name,
        )?;
        if let Err(err) = staging.copy_from_bytes(pixels) {
            unsafe {
                staging.destroy(upload.device, upload.allocator);
            }
            return Err(err);
        }

        let mut allocation = None;
        let mut view = vk::ImageView::null();
        let mut sampler = vk::Sampler::null();

        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
            .samples(vk::SampleCountFlags::TYPE_1)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let image = match unsafe { upload.device.create_image(&image_info, None) } {
            Ok(image) => image,
            Err(err) => {
                unsafe {
                    staging.destroy(upload.device, upload.allocator);
                }
                return Err(err.into());
            }
        };

        let result = (|| -> anyhow::Result<Self> {
            let requirements = unsafe { upload.device.get_image_memory_requirements(image) };

            allocation = Some(upload.allocator.allocate(&AllocationCreateDesc {
                name,
                requirements,
                location: MemoryLocation::GpuOnly,
                linear: false,
                allocation_scheme: AllocationScheme::GpuAllocatorManaged,
            })?);

            let image_allocation = allocation
                .as_ref()
                .expect("image allocation was just created");
            unsafe {
                upload.device.bind_image_memory(
                    image,
                    image_allocation.memory(),
                    image_allocation.offset(),
                )?;
            }

            unsafe {
                buffer::immediate_submit(
                    upload.device,
                    upload.queue,
                    upload.upload_pool,
                    upload.upload_fence,
                    |cmd| {
                        transition_image(
                            upload.device,
                            cmd,
                            image,
                            vk::ImageLayout::UNDEFINED,
                            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                            vk::PipelineStageFlags2::TOP_OF_PIPE,
                            vk::AccessFlags2::empty(),
                            vk::PipelineStageFlags2::TRANSFER,
                            vk::AccessFlags2::TRANSFER_WRITE,
                        );

                        let region = vk::BufferImageCopy::default()
                            .buffer_offset(0)
                            .buffer_row_length(0)
                            .buffer_image_height(0)
                            .image_subresource(
                                vk::ImageSubresourceLayers::default()
                                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                                    .mip_level(0)
                                    .base_array_layer(0)
                                    .layer_count(1),
                            )
                            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
                            .image_extent(vk::Extent3D {
                                width,
                                height,
                                depth: 1,
                            });

                        upload.device.cmd_copy_buffer_to_image(
                            cmd,
                            staging.handle,
                            image,
                            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                            std::slice::from_ref(&region),
                        );

                        transition_image(
                            upload.device,
                            cmd,
                            image,
                            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                            vk::PipelineStageFlags2::TRANSFER,
                            vk::AccessFlags2::TRANSFER_WRITE,
                            vk::PipelineStageFlags2::FRAGMENT_SHADER,
                            vk::AccessFlags2::SHADER_SAMPLED_READ,
                        );

                        Ok(())
                    },
                )?;
            }

            let view_info = vk::ImageViewCreateInfo::default()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format)
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1),
                );
            view = unsafe { upload.device.create_image_view(&view_info, None)? };

            let sampler_info = vk::SamplerCreateInfo::default()
                .mag_filter(vk::Filter::NEAREST)
                .min_filter(vk::Filter::NEAREST)
                .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
                .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                // Images are created with a single mip level, so clamp LOD to 0.0
                // (sampling mip 0 only). A nonzero max_lod would only matter with
                // a real mip chain, which these textures do not have.
                .max_lod(0.0);
            sampler = unsafe { upload.device.create_sampler(&sampler_info, None)? };

            log::info!("created texture '{name}' ({width}x{height}, {color_space:?}, {format:?})");

            Ok(Self {
                image,
                allocation: allocation.take(),
                view,
                sampler,
            })
        })();

        unsafe {
            staging.destroy(upload.device, upload.allocator);
        }

        match result {
            Ok(texture) => Ok(texture),
            Err(err) => {
                unsafe {
                    if sampler != vk::Sampler::null() {
                        upload.device.destroy_sampler(sampler, None);
                    }
                    if view != vk::ImageView::null() {
                        upload.device.destroy_image_view(view, None);
                    }
                    upload.device.destroy_image(image, None);
                }

                if let Some(allocation) = allocation.take() {
                    buffer::free_allocation(upload.allocator, allocation, name);
                }

                Err(err)
            }
        }
    }

    pub unsafe fn destroy(mut self, device: &ash::Device, allocator: &mut Allocator) {
        unsafe {
            device.destroy_sampler(self.sampler, None);
            device.destroy_image_view(self.view, None);
            device.destroy_image(self.image, None);
        }

        if let Some(allocation) = self.allocation.take() {
            buffer::free_allocation(allocator, allocation, "texture");
        }
    }
}

pub fn create_texture_descriptor_set_layout(
    device: &ash::Device,
) -> anyhow::Result<vk::DescriptorSetLayout> {
    let sampler_binding = vk::DescriptorSetLayoutBinding::default()
        .binding(0)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT);

    let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
        .bindings(std::slice::from_ref(&sampler_binding));
    let descriptor_set_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };

    log::info!("created texture descriptor set layout");

    Ok(descriptor_set_layout)
}

pub fn create_texture_descriptor_set(
    device: &ash::Device,
    descriptor_set_layout: vk::DescriptorSetLayout,
    texture: &Texture,
) -> anyhow::Result<(vk::DescriptorPool, vk::DescriptorSet)> {
    let pool_size = vk::DescriptorPoolSize::default()
        .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1);
    let pool_info = vk::DescriptorPoolCreateInfo::default()
        .pool_sizes(std::slice::from_ref(&pool_size))
        .max_sets(1);
    let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };

    let set_layouts = [descriptor_set_layout];
    let set_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(descriptor_pool)
        .set_layouts(&set_layouts);
    let descriptor_set = match unsafe { device.allocate_descriptor_sets(&set_info) } {
        Ok(sets) => sets[0],
        Err(err) => {
            unsafe {
                device.destroy_descriptor_pool(descriptor_pool, None);
            }
            return Err(err.into());
        }
    };

    let image_info = vk::DescriptorImageInfo::default()
        .sampler(texture.sampler)
        .image_view(texture.view)
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

    let write = vk::WriteDescriptorSet::default()
        .dst_set(descriptor_set)
        .dst_binding(0)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .image_info(std::slice::from_ref(&image_info));

    unsafe {
        device.update_descriptor_sets(std::slice::from_ref(&write), &[]);
    }

    log::info!("created texture descriptor set");

    Ok((descriptor_pool, descriptor_set))
}

#[allow(clippy::too_many_arguments)]
pub unsafe fn transition_image(
    device: &ash::Device,
    cmd: vk::CommandBuffer,
    image: vk::Image,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    src_stage: vk::PipelineStageFlags2,
    src_access: vk::AccessFlags2,
    dst_stage: vk::PipelineStageFlags2,
    dst_access: vk::AccessFlags2,
) {
    let barrier = vk::ImageMemoryBarrier2::default()
        .src_stage_mask(src_stage)
        .src_access_mask(src_access)
        .dst_stage_mask(dst_stage)
        .dst_access_mask(dst_access)
        .old_layout(old_layout)
        .new_layout(new_layout)
        .image(image)
        .subresource_range(
            vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1),
        );

    let dependency =
        vk::DependencyInfo::default().image_memory_barriers(std::slice::from_ref(&barrier));

    unsafe {
        device.cmd_pipeline_barrier2(cmd, &dependency);
    }
}

#[cfg(test)]
mod tests {
    use super::validate_rgba8_texture;

    #[test]
    fn zero_extent_is_rejected() {
        assert!(validate_rgba8_texture("t", 0, 1, 0).is_err());
        assert!(validate_rgba8_texture("t", 1, 0, 0).is_err());
        assert!(validate_rgba8_texture("t", 0, 0, 0).is_err());
    }

    #[test]
    fn overflowing_dimensions_are_rejected() {
        // width * height * 4 cannot fit in usize.
        assert!(validate_rgba8_texture("t", u32::MAX, u32::MAX, 0).is_err());
    }

    #[test]
    fn mismatched_pixel_length_is_rejected() {
        // 1x1 RGBA8 needs exactly 4 bytes.
        assert!(validate_rgba8_texture("t", 1, 1, 3).is_err());
        assert!(validate_rgba8_texture("t", 1, 1, 5).is_err());
    }

    #[test]
    fn valid_textures_return_expected_byte_length() {
        assert_eq!(validate_rgba8_texture("t", 1, 1, 4).unwrap(), 4);
        assert_eq!(validate_rgba8_texture("t", 2, 2, 16).unwrap(), 16);
        assert_eq!(validate_rgba8_texture("t", 4, 3, 48).unwrap(), 48);
    }
}
