/**
 * Solid Color Material Fragment Shader Build Material Source Code.
 */

uniform vec3 u_Color;
uniform float u_Transparency;
uniform float u_SpecularShininess;

atoy_Material atoy_build_material(atoy_Fragment fragment) {
    return atoy_Material(u_Transparency, u_Color, u_Color, u_Color, u_Color, u_SpecularShininess);
}
