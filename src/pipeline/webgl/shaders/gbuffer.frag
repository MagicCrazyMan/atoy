#version 300 es

#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
precision highp int;
precision highp sampler2D;
precision highp sampler2DArray;
#else
precision mediump float;
precision mediump int;
precision mediump sampler2D;
precision mediump sampler2DArray;
#endif

#include Defines
#include UniversalUniforms
#include FragmentConstants

layout(location = 0) out vec4 o_PositionAndSpecularShininess;
layout(location = 1) out vec4 o_Normal;
layout(location = 2) out vec4 o_Albedo;

#include FragmentProcess

void main() {
    atoy_Fragment fragment = fragment_process();
    o_PositionAndSpecularShininess = vec4(fragment.position, fragment.shininess);
    o_Normal = vec4(fragment.normal, 1.0f);
    o_Albedo = vec4(fragment.albedo, 1.0f);
}