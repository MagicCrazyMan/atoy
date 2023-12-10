#version 300 es

#ifdef GL_FRAGMENT_PRECISION_HIGH
    precision highp float;
#else
    precision mediump float;
#endif

in vec2 v_TexCoord;

uniform sampler2D u_ColorSampler;

out vec4 out_Color;

void main() {
    vec4 color = texture(u_ColorSampler, v_TexCoord);
    if (color.a == 0.0) {
        discard;
    }
    out_Color = color;
}