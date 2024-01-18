/**
 * Standard Fragment Shader Entry Source Code.
 *
 * Fucntion `atoy_Material atoy_process_frag(atoy_Frag)` MUST be filled.
 */

void main() {
    vec3 normal_ws;
    if(gl_FrontFacing) {
        normal_ws = normalize(v_NormalWS);
    } else {
        normal_ws = normalize(-v_NormalWS);
    }
    atoy_Frag frag = atoy_Frag(u_Transparency, v_PositionWS, v_PositionES, v_PositionCS, normal_ws, v_TexCoord);
    atoy_Material material = atoy_process_frag(frag);

    #ifdef LIGHTING
    vec3 color = atoy_lighting(frag, material);
    #else
    vec3 color = material.diffuse;
    #endif

    o_Color = vec4(color, material.transparency);

    #ifdef BLOOM
    if(atoy_is_bloom_color(color, u_BloomThreshold)) {
        o_BloomColor = vec4(color, 1.0);
    } else {
        o_BloomColor = vec4(0.0, 0.0, 0.0, 1.0);
    }
    #endif
}
