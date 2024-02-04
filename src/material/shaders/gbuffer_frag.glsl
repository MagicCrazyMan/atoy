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

#ifdef PARALLAX_MAP
uniform sampler2D u_ParallaxMap;
uniform float u_ParallaxHeightScale;
in mat3 v_TBNInv;
#endif

void main() {
    atoy_Fragment fragment = atoy_build_fragment();
    atoy_Material material = atoy_build_material(fragment);
    #ifdef PARALLAX_MAP
        vec3 to_camera = normalize(v_TBNInv * u_CameraPosition - v_TBNInv * fragment.position_ws);
        float height = texture(u_ParallaxMap, fragment.tex_coord).r * u_ParallaxHeightScale;
        vec2 offset = to_camera.xy /  to_camera.z * height;
        fragment.tex_coord = fragment.tex_coord - offset;
        if(fragment.tex_coord.x > 1.0 || fragment.tex_coord.y > 1.0 || fragment.tex_coord.x < 0.0 || fragment.tex_coord.y < 0.0)
            discard;
    #endif
    #ifdef NORMAL_MAP
        vec3 normal_color = texture(u_NormalMap, fragment.tex_coord).xyz;
        normal_color.xy = normal_color.xy * 2.0f - 1.0f;
        normal_color.z = (normal_color.z - 0.5f) * 2.0f;
        vec3 normal = v_TBN * normal_color;
        fragment.normal_ws = normalize(normal);
    #endif

    o_PositionAndSpecularShininess = vec4(fragment.position_ws, material.specular_shininess);
    o_Normal = vec4(fragment.normal_ws, 1.0f);
    o_Albedo = vec4(material.albedo, 1.0f);
}
#endif