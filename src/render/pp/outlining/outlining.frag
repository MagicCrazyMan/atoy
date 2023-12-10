#version 300 es

#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif

uniform vec4 u_Color;

out vec4 out_Color;

void main() {
    out_Color = u_Color;
}