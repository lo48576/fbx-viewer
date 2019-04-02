#version 450

layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec3 v_material;
layout(location = 0) out vec4 f_color;

void main() {
    //f_color = vec4(v_normal, 1.0);
    f_color = vec4(v_material, 1.0);
}
