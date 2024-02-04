/**
 * Standard Fragment Shader Draw Source Code.
 *
 * Fucntion `atoy_Material atoy_build_material(atoy_Fragment)` MUST be filled.
 */

#ifndef GBUFFER
layout(location = 0) out vec4 o_Color;

#ifdef BLOOM
uniform vec3 u_BloomThreshold;
layout(location = 1) out vec4 o_BloomColor;
#endif

#if defined(NORMAL_MAP) || defined(PARALLAX_MAP)
in mat3 v_TBN;
#endif

#ifdef NORMAL_MAP
uniform sampler2D u_NormalMap;
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


    #ifdef LIGHTING
    atoy_LightingFragment lighting_fragment = atoy_LightingFragment(fragment.position_ws, fragment.normal_ws);
    atoy_LightingMaterial lighting_material = atoy_LightingMaterial(material.ambient, material.diffuse, material.specular, material.specular_shininess);
    vec3 color = atoy_lighting(u_CameraPosition, lighting_fragment, lighting_material);
    #else
    vec3 color = material.diffuse;
    #endif

    #ifdef BLOOM
    if(dot(color, u_BloomThreshold) > 1.0f) {
        o_BloomColor = vec4(color, 1.0);
    } else {
        o_BloomColor = vec4(0.0, 0.0, 0.0, 0.0);
    }
    #endif

    o_Color = vec4(color, material.transparency);
}
#endif