#version 300 es

#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
precision highp int;
precision highp sampler2D;
#else
precision mediump float;
precision mediump int;
precision mediump sampler2D;
#endif

#include Defines
#include UniversalUniforms

#ifdef USE_LIGHTING
#include Lighting
#endif

in vec2 v_TexCoord;

uniform sampler2D u_PositionsAndSpecularShininessTexture;
uniform sampler2D u_NormalsTexture;
uniform sampler2D u_AlbedoTexture;

out vec4 o_Color;

void main() {
    vec4 position_and_specular_shininess = texture(u_PositionsAndSpecularShininessTexture, v_TexCoord);
    vec3 position = position_and_specular_shininess.xyz;
    float specular_shininess = position_and_specular_shininess.w;

    vec3 normal = texture(u_NormalsTexture, v_TexCoord).xyz;
    vec4 albedo_and_existence = texture(u_AlbedoTexture, v_TexCoord);
    if(albedo_and_existence.a == 0.0f) {
        discard;
    }
    vec3 albedo = albedo_and_existence.xyz;

    #ifdef USE_LIGHTING
    atoy_LightingMaterial lighting_material = atoy_LightingMaterial(position, normal, albedo, specular_shininess);
    vec3 color = atoy_lighting(u_CameraPosition, lighting_material);
    #else
    vec3 color = albedo;
    #endif

    o_Color = vec4(color, 1.0f);
}