#version 300 es

#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif

in vec2 v_TexCoord;

uniform int u_StageFrag;
uniform int u_OutlineWidth;
uniform vec4 u_OutlineColor;
uniform sampler2D u_OutlineSampler;

out vec4 o_Color;

void main() {
    if(u_StageFrag == 0) {
        // stage 1, draw entity normally with outline color
        o_Color = u_OutlineColor;
    } else if(u_StageFrag == 1) {
        // stage 2, draw outline with convolution kernel with size u_OutlineWidth * u_OutlineWidth and weights 1
        ivec2 center = ivec2(v_TexCoord * vec2(textureSize(u_OutlineSampler, 0)));

        // In practic, detects the edge ring is enough
        for(int t = 0; t < u_OutlineWidth * 2 + 1; t++) {
            for(int s = 0; s < u_OutlineWidth * 2 + 1; s++) {
                if (t == 0 || t == u_OutlineWidth * 2) {
                    if (s > 0 && s < u_OutlineWidth * 2) {
                        continue;
                    }
                }

                ivec2 tex_coord = ivec2(center.s + s - u_OutlineWidth, center.t + t - u_OutlineWidth);
                o_Color = texelFetch(u_OutlineSampler, tex_coord, 0);
                if(o_Color != vec4(0.0f, 0.0f, 0.0f, 0.0f)) {
                    return;
                }
            }
        }
    } else if(u_StageFrag == 2) {
        // stage 3, clear color draw on stage 1
        o_Color = vec4(0.0f, 0.0f, 0.0f, 0.0f);
    } else {
        discard;
    }
}