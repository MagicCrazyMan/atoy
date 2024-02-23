/**
 * Solid Color Frgament Process Snippet.
 */

uniform vec3 u_Color;
uniform float u_Transparency;
uniform float u_SpecularShininess;

atoy_Fragment fragment_process() {
    #ifdef USE_NORMAL 
    return atoy_Fragment(v_Position, v_Normal, u_Color, u_SpecularShininess, u_Transparency);
    #else
    return atoy_Fragment(v_Position, vec3(0.0f), u_Color, u_SpecularShininess, u_Transparency);
    #endif
}