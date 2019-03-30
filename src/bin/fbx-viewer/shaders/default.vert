#version 450

layout(location = 0) in vec3 position;

layout(set = 0, binding = 0) uniform Data {
	mat4 world;
	mat4 view;
	mat4 proj;
} uniforms;

void main() {
	mat4 worldview = uniforms.view * uniforms.world;
	gl_Position = uniforms.proj * worldview * vec4(position, 1.0);
}
