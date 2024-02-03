/**
 * Texture Material Fragment Shader Build Material Source Code.
 */

uniform float u_Transparency;
uniform float u_SpecularShininess;
uniform sampler2D u_AlbedoMap;

atoy_Material atoy_build_material(atoy_Fragment fragment) {
    vec3 albedo = texture(u_AlbedoMap, fragment.tex_coord).rgb;

    return atoy_Material(u_Transparency, albedo, albedo, albedo, albedo, u_SpecularShininess);
}
