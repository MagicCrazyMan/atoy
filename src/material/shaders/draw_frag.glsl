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

void main() {
    atoy_Fragment fragment = atoy_build_fragment();
    atoy_Material material = atoy_build_material(fragment);

    #ifdef LIGHTING
    vec3 color = atoy_lighting(fragment, material);
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