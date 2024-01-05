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

    vec3 color;
    if(u_EnableLighting) {
        color = atoy_lighting(frag, material);
    } else {
        color = material.diffuse;
    }

    color = atoy_gamma_correction(color, u_GammaCorrection);
    
    // vec3 a = vec3(0.0);
    // float period = 2000.0;
    // float radius = 10.0 * (mod(u_RenderTime, period) / period);
    // float width = 2.0;
    // float dist = length(frag.position_ws);
    // if(dist < radius) {
    //     a.x = smoothstep(1.0, 0.0, (radius - dist) / width);
    // }

    o_Color = vec4(color, material.transparency);

}
