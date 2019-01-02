#version 150 core

in vec2 u_pos;
out vec2 coord;

void main() {
	coord = u_pos;
	gl_Position = vec4(u_pos, 0.0, 1.0);
}

