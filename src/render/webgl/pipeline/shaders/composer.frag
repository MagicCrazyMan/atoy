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

out vec4 o_FragColor;

void main() {
    o_FragColor = texture(u_Sampler, v_TexCoord, 0.0);
}
