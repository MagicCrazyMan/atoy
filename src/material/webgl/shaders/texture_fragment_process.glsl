/**
 * Texture Frgament Process Snippet.
 */

uniform sampler2D u_AlbedoMap;
uniform float u_Transparency;
uniform float u_SpecularShininess;

#ifdef USE_NORMAL_MAP
uniform sampler2D u_NormalMap;
#endif

#ifdef USE_PARALLAX_MAP
uniform sampler2D u_ParallaxMap;
uniform float u_ParallaxHeightScale;
#endif

atoy_Fragment fragment_process() {
    vec2 tex_coord;
    vec3 normal;

    #ifdef USE_PARALLAX_MAP
    vec3 to_camera = normalize(v_TBNInvert * u_CameraPosition - v_TBNInvert * v_Position);
    float height = texture(u_ParallaxMap, v_TexCoord).r * u_ParallaxHeightScale;
    vec2 offset = to_camera.xy / to_camera.z * height;
    tex_coord = v_TexCoord - offset;
    if(tex_coord.x > 1.0f || tex_coord.y > 1.0f || tex_coord.x < 0.0f || tex_coord.y < 0.0f)
        discard;
    #else
    tex_coord = v_TexCoord;
    #endif

    #ifdef USE_NORMAL_MAP
    normal = texture(u_NormalMap, tex_coord).xyz;
    normal.xy = normal.xy * 2.0f - 1.0f;
    normal.z = (normal.z - 0.5f) * 2.0f;
    normal = normalize(v_TBN * normal);
    #else
        #ifdef USE_NORMAL
        normal = v_Normal;
        #else
        normal = vec3(0.0f);
        #endif
    #endif

    vec3 albedo = texture(u_AlbedoMap, tex_coord).xyz;

    return atoy_Fragment(v_Position, normal, albedo, u_SpecularShininess, u_Transparency);
}