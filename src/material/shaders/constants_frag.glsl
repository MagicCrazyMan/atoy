/**
 * Standard Fragment Shader Constants Source Code.
 */

in vec3 v_PositionWS;
in vec3 v_PositionES;
in vec3 v_PositionCS;
in vec3 v_NormalWS;
in vec2 v_TexCoord;

uniform float u_Transparency;

layout(location = 0) out vec4 o_Color;
#ifdef BLOOM
uniform vec3 u_BloomThreshold;
layout(location = 1) out vec4 o_BloomColor;
#endif

/**
 * Input fragment difinition.
 * 
 * - `transparency`: Fragment transparency.
 * - `position_ws`: Fragment position in WORLD space.
 * - `position_es`: Fragment position in EYE space.
 * - `position_cs`: Fragment position in CLIP space.
 * - `normal_ws`: Normal vector in WORLD space.
 * - `tex_coord`: Texture coordinate associated with.
 */
struct atoy_Frag {
    float transparency;
    vec3 position_ws;
    vec3 position_es;
    vec3 position_cs;
    vec3 normal_ws;
    vec2 tex_coord;
};

/**
 * Output material difinition.
 * 
 * - `transparency`: Fragment transparency.
 * - `ambient`: Ambient color of the material.
 * - `diffuse`: Diffuse color of the material.
 * - `specular`: Specular color of the material.
 */
struct atoy_Material {
    float transparency;
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};
