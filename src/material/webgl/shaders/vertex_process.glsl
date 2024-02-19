/**
 * Standard Vertex Process Code Snippet.
 */

#include UniversalUniforms

in vec4 a_Position;
out vec3 v_Position;
uniform mat4 u_ModelMatrix;

#ifdef USE_POSITION_EYE_SPACE
out vec3 v_PositionES;
#endif

#ifdef USE_NORMAL
in vec3 a_Normal;
out vec3 v_Normal;
uniform mat3 u_NormalMatrix;

    #ifdef USE_TBN
    in vec3 a_Tangent;

    #ifndef USE_CALCULATED_BITANGENT
    in vec3 a_Bitangent;
    #endif

    out mat3 v_TBN;

    #ifdef USE_TBN_INVERT
    out mat3 v_TBNInvert;
    #endif
    #endif
#endif

#ifdef USE_TEXTURE_COORDINATE
in vec2 a_TexCoord;
out vec2 v_TexCoord;
#endif

void vertex_process() {
    v_Position = u_ViewProjMatrix * u_ModelMatrix * a_Position;
    gl_Position = v_Position;
    
    #ifdef USE_NORMAL
    v_Normal = u_NormalMatrix * a_Normal;

        #ifdef USE_TBN
        vec3 T = normalize(u_NormalMatrix * a_Tangent);
        vec3 N = normalize(v_Normal);
        
        #ifndef USE_CALCULATED_BITANGENT
        vec3 B = cross(N, T);
        #elif
        vec3 B = normalize(u_NormalMatrix * a_Bitangent);
        #endif

        v_TBN = mat3(T, B, N);
        v_TBNInvert = transpose(v_TBN);
        #endif
    #endif

    #ifdef USE_TEXTURE_COORDINATE
    v_TexCoord = a_TexCoord;
    #endif

    #ifdef USE_POSITION_EYE_SPACE
    v_PositionES = u_ViewMatrix * u_ModelMatrix * a_Position;
    #endif
}