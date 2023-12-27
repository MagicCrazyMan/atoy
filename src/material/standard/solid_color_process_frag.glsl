/**
 * Sold Color Material Fragment Shader Process Function Source Code.
 */

uniform vec3 u_Color;

atoy_OutputMaterial atoy_process_frag(atoy_InputFrag input_frag) {
    return atoy_OutputMaterial(input_frag.transparency, u_Color, u_Color, u_Color);
}