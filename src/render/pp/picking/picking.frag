#version 300 es

#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif

uniform uint u_Index;

out uint out_Index;

void main() {
    out_Index = u_Index;
}