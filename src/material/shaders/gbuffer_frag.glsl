/**
 * Standard Fragment Shader GBuffer Collector Source Code.
 *
 * Fucntion `atoy_Material atoy_build_material(atoy_Fragment)` MUST be filled.
 */

#ifdef GBUFFER
layout(location = 0) out vec4 o_PositionAndSpecularShininess;
layout(location = 1) out vec4 o_Normal;
layout(location = 2) out vec4 o_Albedo;

void main() {
    atoy_Fragment fragment = atoy_build_fragment();
    atoy_Material material = atoy_build_material(fragment);

    o_PositionAndSpecularShininess = vec4(fragment.position_ws, material.specular_shininess);
    o_Normal = vec4(fragment.normal_ws, 1.0);
    o_Albedo = vec4(material.albedo, 1.0);
}
#endif