#version 300 es

#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
precision highp sampler2D;
#else
precision mediump float;
precision mediump sampler2D;
#endif

#define KERNEL_SIZE 81
#define KERNEL_WIDTH 9
#define KERNEL_HEIGHT 9
#define KERNEL_HALF_WIDTH 4
#define KERNEL_HALF_HEIGHT 4

in vec2 v_TexCoord;

uniform sampler2D u_Sampler;

layout(std140) uniform Kernel {
    float u_Kernel[KERNEL_SIZE];
};

out vec4 o_Color;

void main() {
    // maps v_TexCoord to pixel coordinate
    ivec2 center = ivec2(v_TexCoord * vec2(textureSize(u_Sampler, 0)));

    vec3 color = vec3(0.0f);
    for(int t = 0; t < KERNEL_HEIGHT; t++) {
        for(int s = 0; s < KERNEL_WIDTH; s++) {
            ivec2 pixel = ivec2(center.s + s - KERNEL_HALF_WIDTH, center.t + t - KERNEL_HALF_HEIGHT) ;
            vec3 rgb = texelFetch(u_Sampler, pixel, 0).rgb;
            color += rgb * u_Kernel[t * KERNEL_WIDTH + s];
        }
    }

    o_Color = vec4(color, 1.0);
}
