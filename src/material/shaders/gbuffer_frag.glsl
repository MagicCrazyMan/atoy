/**
 * Standard Fragment Shader GBuffer Collector Source Code.
 *
 * Fucntion `atoy_Material atoy_build_material(atoy_Fragment)` MUST be filled.
 */

#ifdef GBUFFER
layout(location = 0) out vec4 o_Position;
layout(location = 1) out vec4 o_Normal;
layout(location = 2) out vec4 o_AlbedoAndSpecularShininess;

void main() {
    atoy_Fragment fragment = atoy_build_fragment();
    atoy_Material material = atoy_build_material(fragment);

    o_Position = vec4(fragment.position_ws, 1.0);
    o_Normal = vec4(fragment.normal_ws, 0.0);
    o_AlbedoAndSpecularShininess = vec4(material.albedo, material.specular_shininess);
}
#endif