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

void main() {
    vec4 color = texture(u_Sampler, v_TexCoord);
    vec3 rgb = pow(color.rgb, vec3(1.0f / 2.2f));
    o_Color = vec4(rgb, color.a);
}
