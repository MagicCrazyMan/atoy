/**
 * Standard Fragment Shader Prelude Source Code.
 */

in vec3 v_PositionWS;
in vec3 v_PositionES;
in vec3 v_PositionCS;
in vec3 v_NormalWS;
in vec2 v_TexCoord;

/**
 * Transparency of this material.
 */
uniform float u_Transparency;

out vec4 o_Color;

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

/**
 * Uniform block containing scene universal uniforms.
 * 
 * - `u_RenderTime`: Render time of current frame.
 * - `u_EnableLighting`: Is lighting enabled.
 * - `u_CameraPosition`: Camera position in WORLD space.
 * - `u_ViewMatrix`: View matrix.
 * - `u_ProjMatrix`: Proj matrix.
 * - `u_ViewProjMatrix`: View-Proj matrix.
 */
layout(std140) uniform atoy_UniversalUniforms {
                                // base alignment (bytes) // offset alignment (bytes)
    bool u_RenderTime;          // (merged)               // 0
    bool u_EnableLighting;      // 16                     // 4
    vec3 u_CameraPosition;      // 16                     // 16
    mat4 u_ViewMatrix;          // 64                     // 32
    mat4 u_ProjMatrix;          // 64                     // 96
    mat4 u_ViewProjMatrix;      // 64                     // 160
};
