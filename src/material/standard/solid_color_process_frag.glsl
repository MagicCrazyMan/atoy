/**
 * Sold Color Material Fragment Shader Process Function Source Code.
 */

uniform vec3 u_Color;

atoy_Material atoy_process_frag(atoy_Frag frag) {
    return atoy_Material(frag.transparency, u_Color, u_Color, u_Color);
}
