use ash::vk;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteVertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl SpriteVertex {
    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Self>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }
    }

    pub fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 3] {
        [
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: std::mem::offset_of!(SpriteVertex, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32_SFLOAT,
                offset: std::mem::offset_of!(SpriteVertex, uv) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 2,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: std::mem::offset_of!(SpriteVertex, color) as u32,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::SpriteVertex;

    #[test]
    fn sprite_vertex_attribute_offsets_match_struct_layout() {
        let attributes = SpriteVertex::attribute_descriptions();

        assert_eq!(
            attributes[0].offset,
            std::mem::offset_of!(SpriteVertex, pos) as u32
        );
        assert_eq!(
            attributes[1].offset,
            std::mem::offset_of!(SpriteVertex, uv) as u32
        );
        assert_eq!(
            attributes[2].offset,
            std::mem::offset_of!(SpriteVertex, color) as u32
        );
    }
}
