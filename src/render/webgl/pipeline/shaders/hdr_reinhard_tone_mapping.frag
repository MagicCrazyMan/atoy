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
uniform float exposure;

out vec4 o_Color;

void main() {
    vec4 sampled = texture(u_Sampler, v_TexCoord);
    vec3 hdr_color = sampled.rgb;

    vec3 mapped = hdr_color / (hdr_color + 1.0f);

    o_Color = vec4(mapped, sampled.a);
}