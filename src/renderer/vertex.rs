use ash::vk;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex2D {
    pub pos: [f32; 2],
    pub color: [f32; 3],
}

impl Vertex2D {
    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Self>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }
    }

    pub fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        [
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: 0,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: std::mem::size_of::<[f32; 2]>() as u32,
            },
        ]
    }
}

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
                offset: 0,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32_SFLOAT,
                offset: std::mem::size_of::<[f32; 2]>() as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 2,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: std::mem::size_of::<[f32; 4]>() as u32,
            },
        ]
    }
}

pub fn quad_vertices(x: f32, y: f32, w: f32, h: f32) -> [SpriteVertex; 6] {
    let white = [1.0, 1.0, 1.0, 1.0];

    [
        SpriteVertex {
            pos: [x, y],
            uv: [0.0, 0.0],
            color: white,
        },
        SpriteVertex {
            pos: [x + w, y],
            uv: [1.0, 0.0],
            color: white,
        },
        SpriteVertex {
            pos: [x + w, y + h],
            uv: [1.0, 1.0],
            color: white,
        },
        SpriteVertex {
            pos: [x, y],
            uv: [0.0, 0.0],
            color: white,
        },
        SpriteVertex {
            pos: [x + w, y + h],
            uv: [1.0, 1.0],
            color: white,
        },
        SpriteVertex {
            pos: [x, y + h],
            uv: [0.0, 1.0],
            color: white,
        },
    ]
}
