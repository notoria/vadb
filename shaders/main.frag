#version 150 core

precision highp float;

#define PI 3.14159265359

uniform sampler2D tex0;

in vec2 coord;
out vec4 target;

void main() {
	vec2 st = coord.xy;
	vec2 uv = (st + vec2(1.0)) / 2.0;
	uv.y = 1. - uv.y;
	target = texture(tex0, uv);
}

