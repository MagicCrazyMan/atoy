#version 300 es

#include Defines
#include UniversalUniforms

in vec3 a_Position;
out vec3 v_Position;
uniform mat4 u_ModelMatrix;

#ifdef USE_POSITION_EYE_SPACE
out vec3 v_PositionES;
#endif

#ifdef USE_NORMAL
in vec3 a_Normal;
out vec3 v_Normal;
uniform mat4 u_NormalMatrix;

    #ifdef USE_TBN
    in vec3 a_Tangent;
    out mat3 v_TBN;

        #ifndef USE_CALCULATED_BITANGENT
        in vec3 a_Bitangent;
        #endif

        #ifdef USE_TBN_INVERT
        out mat3 v_TBNInvert;
        #endif
    #endif
#endif

#ifdef USE_TEXTURE_COORDINATE
in vec2 a_TexCoord;
out vec2 v_TexCoord;
#endif

void main() {
    vec4 position = u_ModelMatrix * vec4(a_Position, 1.0f);
    v_Position = vec3(position);
    gl_Position = u_ViewProjMatrix * position;
    
    #ifdef USE_NORMAL
    v_Normal = vec3(u_NormalMatrix * vec4(a_Normal, 0.0f));

        #ifdef USE_TBN
        vec3 T = normalize(vec3(u_NormalMatrix * vec4(a_Tangent, 0.0f)));
        vec3 N = normalize(v_Normal);
        vec3 B;
        
        #ifdef USE_CALCULATED_BITANGENT
        B = cross(N, T);
        #else
        B = normalize(vec3(u_NormalMatrix * vec4(a_Bitangent, 0.0f)));
        #endif

        v_TBN = mat3(T, B, N);
        v_TBNInvert = transpose(v_TBN);
        #endif
    #endif

    #ifdef USE_TEXTURE_COORDINATE
    v_TexCoord = a_TexCoord;
    #endif

    #ifdef USE_POSITION_EYE_SPACE
    vec4 position_es = u_ViewMatrix * u_ModelMatrix * a_Position;
    v_PositionES = vec3(position_es / position_es.z);
    #endif
}