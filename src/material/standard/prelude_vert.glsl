/**
 * Standard Vertex Shader Prelude Source Code.
 */

in vec4 a_Position;
in vec3 a_Normal;
in vec2 a_TexCoord;

uniform mat4 u_ModelMatrix;
uniform mat4 u_NormalMatrix;

out vec3 v_PositionWS;
out vec3 v_PositionES;
out vec3 v_PositionCS;
out vec3 v_NormalWS;
out vec2 v_TexCoord;

/**
 * Input vertex difinition.
 * 
 * - `position`: Vertex position in LOCAL space.
 * - `normal`: Normal vector in LOCAL space.
 * - `tex_coord`: Texture coordinate associated with.
 */
struct atoy_InputVertex {
    vec4 position;
    vec3 normal;
    vec2 tex_coord;
};

/**
 * Output vertex difinition.
 * 
 * - `position`: Vertex position in WORLD space.
 * - `normal`: Normal vector in WORLD space.
 * - `tex_coord`: Texture coordinate associated with.
 */
struct atoy_OutputVertex {
    vec4 position;
    vec3 normal;
    vec2 tex_coord;
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
