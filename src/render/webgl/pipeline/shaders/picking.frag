#version 300 es

#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif

layout(location = 0) out uint out_Index;
layout(location = 1) out uvec3 out_Position;

in vec3 v_Position;

uniform uint u_Index;

void main() {
    out_Index = u_Index;
    out_Position = uvec3(floatBitsToUint(v_Position.x), floatBitsToUint(v_Position.y), floatBitsToUint(v_Position.z));
}
