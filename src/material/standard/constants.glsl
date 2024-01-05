/**
 * Standard Shader Constants Source Code.
 */

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
    float u_RenderTime;             // (merged)               // 0
    bool u_EnableLighting;          // (merged)               // 4
    float u_GammaCorrection;        // (merged)               // 8
    float u_GammaCOrrectionInverse; // 16                     // 12
    vec3 u_CameraPosition;          // 16                     // 16
    mat4 u_ViewMatrix;              // 64                     // 32
    mat4 u_ProjMatrix;              // 64                     // 96
    mat4 u_ViewProjMatrix;          // 64                     // 160
};
