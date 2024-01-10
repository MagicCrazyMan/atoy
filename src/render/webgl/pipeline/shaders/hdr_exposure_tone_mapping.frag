#version 300 es

#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
precision highp sampler2D;
#else
precision mediump float;
precision mediump sampler2D;
#endif

in vec2 v_TexCoord;

uniform sampler2D u_Sampler;
uniform float u_Exposure;

out vec4 o_Color;

void main() {
    vec4 sampled = texture(u_Sampler, v_TexCoord);
    vec3 hdr_color = sampled.rgb;

    vec3 mapped = 1.0 - exp(-hdr_color * u_Exposure);
    
    o_Color = vec4(mapped, sampled.a);
}