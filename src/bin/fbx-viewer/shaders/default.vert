#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in uint material;

layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec3 v_material;

layout(set = 0, binding = 0) uniform Data {
	mat4 world;
	mat4 view;
	mat4 proj;
} uniforms;

void main() {
	mat4 worldview = uniforms.view * uniforms.world;
	v_normal = (normal + vec3(1.0)) / 2;
	v_material = vec3(float(material));
	gl_Position = uniforms.proj * worldview * vec4(position, 1.0);
}
