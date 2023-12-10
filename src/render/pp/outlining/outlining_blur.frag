#version 300 es

#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif

in vec2 v_TexCoord;

uniform bool u_Horizontal;
uniform sampler2D u_ColorSampler;

// contribution weights of each pixel under gaussian blur
float weights[5] = float[](0.227027f, 0.1945946f, 0.1216216f, 0.054054f, 0.016216f);

out vec4 out_Color;

void main() {
    // calculates gussian blur
    vec2 tex_size = 1.0f / vec2(textureSize(u_ColorSampler, 0)); // units per texture pixel size
    // collect contribution of each pixel around current pixel
    vec4 color = texture(u_ColorSampler, v_TexCoord) * weights[0];
    if(u_Horizontal) {
    // in s direction
        for(int i = 1; i < weights.length(); i++) {
            float offset = tex_size.s * float(i);
            color += texture(u_ColorSampler, vec2(v_TexCoord.s - offset, v_TexCoord.t)) * weights[i];
            color += texture(u_ColorSampler, vec2(v_TexCoord.s + offset, v_TexCoord.t)) * weights[i];
        }
    } else {
    // in t direction
        for(int i = 1; i < weights.length(); i++) {
            float offset = tex_size.t * float(i);
            color += texture(u_ColorSampler, vec2(v_TexCoord.s, v_TexCoord.t - offset)) * weights[i];
            color += texture(u_ColorSampler, vec2(v_TexCoord.s, v_TexCoord.t + offset)) * weights[i];
        }
    }

    out_Color = color;
}