#version 450

layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec2 v_uv;

layout(location = 0) out vec4 f_color;

layout(set = 1, binding = 0) uniform sampler2D diffuse;

layout(set = 2, binding = 0) uniform Material {
	vec3 ambient;
	vec3 diffuse;
	vec3 emissive;
	bool enabled;
} material;

void main() {
    vec4 diffuse = material.enabled ?
		vec4(material.diffuse, 1.0) :
		texture(diffuse, v_uv);
	f_color = diffuse;
}
