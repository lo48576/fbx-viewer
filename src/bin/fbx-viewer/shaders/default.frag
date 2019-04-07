#version 450

layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec2 v_uv;

layout(location = 0) out vec4 f_color;

layout(set = 1, binding = 0) uniform sampler2D diffuse;

void main() {
    f_color = texture(diffuse, v_uv);
}
