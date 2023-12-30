/**
 * Texture Material Fragment Shader Process Function Source Code.
 */

uniform sampler2D u_DiffuseSampler;

atoy_Material atoy_process_frag(atoy_Frag frag) {
    vec3 diffuse = texture(u_DiffuseSampler, frag.tex_coord).rgb;

    return atoy_Material(frag.transparency, diffuse, diffuse, diffuse);
}
