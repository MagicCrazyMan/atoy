#version 300 es

#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif

in vec2 v_TexCoord;

uniform sampler2D u_Sampler;

out vec4 o_FragColor;

void main() {
    o_FragColor = texture(u_Sampler, v_TexCoord, 0.0);
}
