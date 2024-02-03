/**
 * Standard Vertex Shader Constants Source Code.
 */

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
