#version 300 es

#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
precision highp sampler2D;
#else
precision mediump float;
precision mediump sampler2D;
#endif

in vec2 v_TexCoord;

uniform sampler2D u_BaseTexture;
uniform vec3 u_BloomThreshold;

out vec4 o_Color;

void main() {
    vec4 color = texture(u_BaseTexture, v_TexCoord);
    vec3 rgb = color.rgb;
    float brightness = dot(rgb, u_BloomThreshold);
    if(brightness > 1.0f) {
        o_Color = vec4(rgb, 1.0);
    } else {
        o_Color = vec4(0.0f, 0.0f, 0.0f, 1.0f);
    }
}