use ash::vk;
use std::io::Cursor;

use crate::renderer::vertex::SpriteVertex;

pub const SPRITE_VERT_SPV: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/sprite.vert.spv"));
pub const SPRITE_FRAG_SPV: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/sprite.frag.spv"));

pub fn create_shader_module(
    device: &ash::Device,
    bytes: &[u8],
) -> anyhow::Result<vk::ShaderModule> {
    let words = ash::util::read_spv(&mut Cursor::new(bytes))?;
    let info = vk::ShaderModuleCreateInfo::default().code(&words);
    let module = unsafe { device.create_shader_module(&info, None)? };
    Ok(module)
}

pub fn create_sprite_shader_modules(
    device: &ash::Device,
) -> anyhow::Result<(vk::ShaderModule, vk::ShaderModule)> {
    let vert = create_shader_module(device, SPRITE_VERT_SPV)?;
    let frag = match create_shader_module(device, SPRITE_FRAG_SPV) {
        Ok(frag) => frag,
        Err(err) => {
            unsafe {
                device.destroy_shader_module(vert, None);
            }
            return Err(err);
        }
    };
    Ok((vert, frag))
}

pub struct GraphicsPipeline {
    device: ash::Device,
    layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
}

impl GraphicsPipeline {
    pub fn new_sprite(
        device: &ash::Device,
        swapchain_format: vk::Format,
        descriptor_set_layout: vk::DescriptorSetLayout,
    ) -> anyhow::Result<Self> {
        let (vert_module, frag_module) = create_sprite_shader_modules(device)?;
        let result = create_sprite_graphics_pipeline(
            device,
            swapchain_format,
            descriptor_set_layout,
            vert_module,
            frag_module,
        );

        unsafe {
            device.destroy_shader_module(frag_module, None);
            device.destroy_shader_module(vert_module, None);
        }

        result
    }

    pub fn layout(&self) -> vk::PipelineLayout {
        self.layout
    }

    pub fn pipeline(&self) -> vk::Pipeline {
        self.pipeline
    }

    pub fn destroy(&mut self) {
        unsafe {
            if self.pipeline != vk::Pipeline::null() {
                self.device.destroy_pipeline(self.pipeline, None);
                self.pipeline = vk::Pipeline::null();
            }

            if self.layout != vk::PipelineLayout::null() {
                self.device.destroy_pipeline_layout(self.layout, None);
                self.layout = vk::PipelineLayout::null();
            }
        }
    }
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        self.destroy();
    }
}

fn create_sprite_graphics_pipeline(
    device: &ash::Device,
    swapchain_format: vk::Format,
    descriptor_set_layout: vk::DescriptorSetLayout,
    vert_module: vk::ShaderModule,
    frag_module: vk::ShaderModule,
) -> anyhow::Result<GraphicsPipeline> {
    let set_layouts = [descriptor_set_layout];
    let push_constant_range = vk::PushConstantRange::default()
        .stage_flags(vk::ShaderStageFlags::VERTEX)
        .offset(0)
        .size(std::mem::size_of::<glam::Mat4>() as u32);
    let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
        .set_layouts(&set_layouts)
        .push_constant_ranges(std::slice::from_ref(&push_constant_range));
    let pipeline_layout = unsafe { device.create_pipeline_layout(&pipeline_layout_info, None)? };

    let color_formats = [swapchain_format];
    let mut rendering_info =
        vk::PipelineRenderingCreateInfo::default().color_attachment_formats(&color_formats);

    let main = c"main";

    let shader_stages = [
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_module)
            .name(main),
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_module)
            .name(main),
    ];

    let binding = SpriteVertex::binding_description();
    let attributes = SpriteVertex::attribute_descriptions();

    let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(std::slice::from_ref(&binding))
        .vertex_attribute_descriptions(&attributes);

    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

    let viewport_state = vk::PipelineViewportStateCreateInfo::default()
        .viewport_count(1)
        .scissor_count(1);

    let rasterization = vk::PipelineRasterizationStateCreateInfo::default()
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::NONE)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .line_width(1.0);

    let multisample = vk::PipelineMultisampleStateCreateInfo::default()
        .rasterization_samples(vk::SampleCountFlags::TYPE_1);

    let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
        .blend_enable(true)
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .alpha_blend_op(vk::BlendOp::ADD)
        .color_write_mask(
            vk::ColorComponentFlags::R
                | vk::ColorComponentFlags::G
                | vk::ColorComponentFlags::B
                | vk::ColorComponentFlags::A,
        );

    let color_blend = vk::PipelineColorBlendStateCreateInfo::default()
        .attachments(std::slice::from_ref(&color_blend_attachment));

    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state =
        vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

    let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
        .push_next(&mut rendering_info)
        .stages(&shader_stages)
        .vertex_input_state(&vertex_input)
        .input_assembly_state(&input_assembly)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterization)
        .multisample_state(&multisample)
        .color_blend_state(&color_blend)
        .dynamic_state(&dynamic_state)
        .layout(pipeline_layout);

    let pipeline_result = unsafe {
        device.create_graphics_pipelines(
            vk::PipelineCache::null(),
            std::slice::from_ref(&pipeline_info),
            None,
        )
    };

    let pipeline = match pipeline_result {
        Ok(pipelines) => pipelines[0],
        Err((pipelines, err)) => {
            unsafe {
                for pipeline in pipelines {
                    device.destroy_pipeline(pipeline, None);
                }
                device.destroy_pipeline_layout(pipeline_layout, None);
            }
            return Err(err.into());
        }
    };

    log::info!(
        "created alpha-blended sprite pipeline for {:?}",
        swapchain_format
    );

    Ok(GraphicsPipeline {
        device: device.clone(),
        layout: pipeline_layout,
        pipeline,
    })
}
