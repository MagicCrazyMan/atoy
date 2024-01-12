/**
 * Standard Vertex Shader Constants Source Code.
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
