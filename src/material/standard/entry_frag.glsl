/**
 * Standard Fragment Shader Entry Source Code.
 *
 * Fucntion `atoy_OutputMaterial atoy_process_frag(atoy_InputFrag)` MUST be filled.
 */

void main() {
    atoy_InputFrag input_frag = atoy_InputFrag(u_Transparency, v_PositionWS, v_PositionES, v_PositionCS, normalize(v_NormalWS), v_TexCoord);
    atoy_OutputMaterial output_material = atoy_process_frag(input_frag);

    vec3 color;
    if(u_EnableLighting) {
        color = atoy_lighting(input_frag, output_material);
    } else {
        color = output_material.diffuse;
    }

    color = atoy_gamma_correction(color);
    o_Color = vec4(color, output_material.transparency);
}
