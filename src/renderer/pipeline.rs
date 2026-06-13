#![allow(dead_code)]

use ash::vk;
use std::io::Cursor;

pub const TRIANGLE_VERT_SPV: &[u8] = include_bytes!("../../shaders/triangle.vert.spv");
pub const TRIANGLE_FRAG_SPV: &[u8] = include_bytes!("../../shaders/triangle.frag.spv");
pub const SPRITE_VERT_SPV: &[u8] = include_bytes!("../../shaders/sprite.vert.spv");
pub const SPRITE_FRAG_SPV: &[u8] = include_bytes!("../../shaders/sprite.frag.spv");

pub fn create_shader_module(
    device: &ash::Device,
    bytes: &[u8],
) -> anyhow::Result<vk::ShaderModule> {
    let words = ash::util::read_spv(&mut Cursor::new(bytes))?;
    let info = vk::ShaderModuleCreateInfo::default().code(&words);
    let module = unsafe { device.create_shader_module(&info, None)? };
    Ok(module)
}

pub fn create_triangle_shader_modules(
    device: &ash::Device,
) -> anyhow::Result<(vk::ShaderModule, vk::ShaderModule)> {
    let vert = create_shader_module(device, TRIANGLE_VERT_SPV)?;
    let frag = create_shader_module(device, TRIANGLE_FRAG_SPV)?;
    Ok((vert, frag))
}

pub fn create_sprite_shader_modules(
    device: &ash::Device,
) -> anyhow::Result<(vk::ShaderModule, vk::ShaderModule)> {
    let vert = create_shader_module(device, SPRITE_VERT_SPV)?;
    let frag = create_shader_module(device, SPRITE_FRAG_SPV)?;
    Ok((vert, frag))
}
