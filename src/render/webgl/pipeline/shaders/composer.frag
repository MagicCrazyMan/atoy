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

out vec4 o_Color;

/**
 * Standard Fragment Shader Gamma Correction Source Code.
 */

/**
 * Calculates gamma correction to a specified color.
 */
vec3 atoy_gamma_correction(vec3 color, float gamma) {
    return pow(color, vec3(1.0f / gamma));
}

/**
 * Calculates gamma to a specified color.
 */
vec3 atoy_gamma(vec3 color, float gamma) {
    return pow(color, vec3(gamma));
}

void main() {
    vec4 color = texture(u_Sampler, v_TexCoord);
    vec3 rgb = atoy_gamma_correction(color.rgb, 2.2f);
    o_Color = vec4(rgb, color.a);
}
