/**
 * Standard Fragment Shader GBuffer Collector Source Code.
 *
 * Fucntion `atoy_Material atoy_build_material(atoy_Fragment)` MUST be filled.
 */

#ifdef GBUFFER
layout(location = 0) out vec4 o_PositionAndSpecularShininess;
layout(location = 1) out vec4 o_Normal;
layout(location = 2) out vec4 o_Albedo;

#ifdef NORMAL_MAP
uniform sampler2D u_NormalMap;
in mat3 v_TBN;
#endif

void main() {
    atoy_Fragment fragment = atoy_build_fragment();
    #ifdef NORMAL_MAP
        vec3 color = texture(u_NormalMap, fragment.tex_coord).xyz;
        color.xy = color.xy * 2.0f - 1.0f;
        color.z = (color.z - 0.5f) * 2.0f;
        vec3 normal = v_TBN * color;
        fragment.normal_ws = normalize(normal);
    #endif
    atoy_Material material = atoy_build_material(fragment);

    o_PositionAndSpecularShininess = vec4(fragment.position_ws, material.specular_shininess);
    o_Normal = vec4(fragment.normal_ws, 1.0f);
    o_Albedo = vec4(material.albedo, 1.0f);
}
#endif