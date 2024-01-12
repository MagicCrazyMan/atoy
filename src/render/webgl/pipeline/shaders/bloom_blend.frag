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
uniform sampler2D u_SamplerBloomBlur;

out vec4 o_Color;

void main(){
    vec4 base_color = texture(u_Sampler, v_TexCoord);
    vec4 bloom_blur_color = texture(u_SamplerBloomBlur, v_TexCoord);
    o_Color = vec4(base_color.rgb + bloom_blur_color.rgb, base_color.a);
}