use ash::vk;
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator};
use std::path::Path;

use crate::renderer::buffer;

pub struct Texture {
    pub image: vk::Image,
    pub allocation: Option<Allocation>,
    pub view: vk::ImageView,
    pub sampler: vk::Sampler,
    #[allow(dead_code)]
    pub width: u32,
    #[allow(dead_code)]
    pub height: u32,
}

impl Texture {
    pub fn from_path(
        device: &ash::Device,
        allocator: &mut Allocator,
        queue: vk::Queue,
        upload_pool: vk::CommandPool,
        upload_fence: vk::Fence,
        path: impl AsRef<Path>,
        name: &str,
    ) -> anyhow::Result<Self> {
        let rgba = image::open(path.as_ref())?.to_rgba8();
        let width = rgba.width();
        let height = rgba.height();
        let pixels = rgba.into_raw();
        Self::from_rgba8(
            device,
            allocator,
            queue,
            upload_pool,
            upload_fence,
            width,
            height,
            &pixels,
            name,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_rgba8(
        device: &ash::Device,
        allocator: &mut Allocator,
        queue: vk::Queue,
        upload_pool: vk::CommandPool,
        upload_fence: vk::Fence,
        width: u32,
        height: u32,
        pixels: &[u8],
        name: &str,
    ) -> anyhow::Result<Self> {
        let expected_len = width as usize * height as usize * 4;
        if pixels.len() != expected_len {
            anyhow::bail!(
                "texture '{name}' has {} bytes, expected {expected_len}",
                pixels.len()
            );
        }

        let image_size = pixels.len() as vk::DeviceSize;

        let staging_name = format!("{name} texture staging");
        let mut staging = buffer::Buffer::new(
            device,
            allocator,
            image_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            MemoryLocation::CpuToGpu,
            &staging_name,
        )?;

        let allocation = staging.allocation.as_mut().unwrap();
        let mapped = allocation
            .mapped_ptr()
            .expect("CpuToGpu allocation should be mapped");

        unsafe {
            std::ptr::copy_nonoverlapping(
                pixels.as_ptr(),
                mapped.as_ptr() as *mut u8,
                pixels.len(),
            );
        }

        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .format(vk::Format::R8G8B8A8_UNORM)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
            .samples(vk::SampleCountFlags::TYPE_1)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let image = unsafe { device.create_image(&image_info, None)? };
        let requirements = unsafe { device.get_image_memory_requirements(image) };

        let allocation = allocator.allocate(&AllocationCreateDesc {
            name,
            requirements,
            location: MemoryLocation::GpuOnly,
            linear: false,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        })?;

        unsafe {
            device.bind_image_memory(image, allocation.memory(), allocation.offset())?;
        }

        unsafe {
            buffer::immediate_submit(device, queue, upload_pool, upload_fence, |cmd| {
                transition_image(
                    device,
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

                device.cmd_copy_buffer_to_image(
                    cmd,
                    staging.handle,
                    image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    std::slice::from_ref(&region),
                );

                transition_image(
                    device,
                    cmd,
                    image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    vk::PipelineStageFlags2::TRANSFER,
                    vk::AccessFlags2::TRANSFER_WRITE,
                    vk::PipelineStageFlags2::FRAGMENT_SHADER,
                    vk::AccessFlags2::SHADER_SAMPLED_READ,
                );
            })?;

            staging.destroy(device, allocator);
        }

        let view_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::R8G8B8A8_UNORM)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1),
            );
        let view = unsafe { device.create_image_view(&view_info, None)? };

        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::NEAREST)
            .min_filter(vk::Filter::NEAREST)
            .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .max_lod(1.0);
        let sampler = unsafe { device.create_sampler(&sampler_info, None)? };

        log::info!("created texture '{name}' ({width}x{height})");

        Ok(Self {
            image,
            allocation: Some(allocation),
            view,
            sampler,
            width,
            height,
        })
    }

    pub unsafe fn destroy(mut self, device: &ash::Device, allocator: &mut Allocator) {
        unsafe {
            device.destroy_sampler(self.sampler, None);
            device.destroy_image_view(self.view, None);
            device.destroy_image(self.image, None);
        }

        if let Some(allocation) = self.allocation.take() {
            allocator.free(allocation).expect("free texture allocation");
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
    let descriptor_set = unsafe { device.allocate_descriptor_sets(&set_info)?[0] };

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
