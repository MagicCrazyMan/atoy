#version 300 es

#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
precision highp sampler2D;
#else
precision mediump float;
precision mediump sampler2D;
#endif

in vec2 v_TexCoord;

uniform sampler2D u_HdrTexture;
uniform float u_HdrExposure;

out vec4 o_Color;

void main() {
    vec4 hdr_color = texture(u_HdrTexture, v_TexCoord);
    vec3 rgb = hdr_color.rgb;

    vec3 mapped = 1.0 - exp(-rgb * u_HdrExposure);
    
    o_Color = vec4(mapped, hdr_color.a);
}