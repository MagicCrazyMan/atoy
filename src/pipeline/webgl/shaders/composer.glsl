in vec2 v_TexCoord;

uniform sampler2D u_Texture;

out vec4 o_Color;

void main() {
    vec4 color = texture(u_Texture, v_TexCoord);
    if(color == vec4(0.0f)) {
        discard;
    }

    #ifdef GAMMA_CORRECTION
    vec3 rgb = atoy_gamma_correction(color.rgb, u_Gamma);
    #else
    vec3 rgb = color.rgb;
    #endif

    o_Color = vec4(rgb, color.a);
}
