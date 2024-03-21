/**
 * Solid Color Frgament Process Snippet.
 */

uniform vec3 u_Material_Color;
uniform float u_Material_Transparency;
uniform float u_Material_SpecularShininess;

atoy_Fragment fragment_process() {
    #ifdef USE_NORMAL 
    return atoy_Fragment(v_Position, v_Normal, u_Material_Color, u_Material_SpecularShininess, u_Material_Transparency);
    #else
    return atoy_Fragment(v_Position, vec3(0.0f), u_Material_Color, u_Material_SpecularShininess, u_Material_Transparency);
    #endif
}