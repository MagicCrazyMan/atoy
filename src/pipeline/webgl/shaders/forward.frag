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

in vec3 v_Position;

#ifdef USE_POSITION_EYE_SPACE
in vec3 v_PositionES;
#endif

#ifdef USE_NORMAL
in vec3 v_Normal;

    #ifdef USE_TBN
in mat3 v_TBN;

        #ifdef USE_TBN_INVERT
in mat3 v_TBNInvert;
        #endif
    #endif
#endif

#ifdef USE_TEXTURE_COORDINATE
in vec2 v_TexCoord;
#endif

layout(location = 0) out vec4 o_Color;

#ifdef USE_BLOOM
uniform vec3 u_BloomThreshold;
layout(location = 1) out vec4 o_BloomColor;
#endif

#include FragmentProcess

#ifdef USE_LIGHTING
#include Lighting
#endif

void main() {
    atoy_Fragment fragment = fragment_process();

    vec3 color;
    #ifdef USE_LIGHTING
    atoy_LightingMaterial lighting_material = atoy_LightingMaterial(fragment.position, fragment.normal, fragment.albedo, fragment.shininess);
    color = atoy_lighting(u_CameraPosition, lighting_material);
    #else
    color = fragment.albedo;
    #endif
    o_Color = vec4(color, fragment.transparency);

    #ifdef USE_BLOOM
    if(dot(color, u_BloomThreshold) > 1.0f) {
        o_BloomColor = vec4(color, 1.0f);
    } else {
        o_BloomColor = vec4(0.0f, 0.0f, 0.0f, 0.0f);
    }
    #endif
}