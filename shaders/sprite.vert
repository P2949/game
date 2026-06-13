#version 450

layout(location = 0) in vec2 in_pos;
layout(location = 1) in vec2 in_uv;
layout(location = 2) in vec4 in_color;

layout(location = 0) out vec2 frag_uv;
layout(location = 1) out vec4 frag_color;

layout(push_constant) uniform PushConstants {
    mat4 view_proj;
} pc;

void main() {
    frag_uv = in_uv;
    frag_color = in_color;
    gl_Position = pc.view_proj * vec4(in_pos, 0.0, 1.0);
}
