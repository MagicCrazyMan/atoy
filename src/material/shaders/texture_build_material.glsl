/**
 * Texture Material Fragment Shader Build Material Source Code.
 */

uniform sampler2D u_DiffuseTexture;
uniform float u_Transparency;
uniform float u_SpecularShininess;

atoy_Material atoy_build_material(atoy_Fragment frag) {
    vec3 diffuse = texture(u_DiffuseTexture, frag.tex_coord).rgb;

    return atoy_Material(u_Transparency, diffuse, diffuse, diffuse, diffuse, u_SpecularShininess);
}
