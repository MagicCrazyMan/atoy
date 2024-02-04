/**
 * Standard Vertex Shader Build Vertex Source Code.
 */

uniform mat4 u_ModelMatrix;
uniform mat4 u_NormalMatrix;

#if defined(NORMAL_MAP) || defined(PARALLAX_MAP)
in vec3 a_Tangent;
in vec3 a_Bitangent;
out mat3 v_TBN;
#endif

#ifdef PARALLAX_MAP 
out mat3 v_TBNInv;
#endif

atoy_OutputVertex atoy_build_vertex(atoy_InputVertex input_vertex) {
    vec4 position = u_ModelMatrix * input_vertex.position;
    vec4 normal = u_NormalMatrix * vec4(input_vertex.normal, 0.0);

    #if defined(NORMAL_MAP) || defined(PARALLAX_MAP)
        vec3 T = normalize(vec3(u_ModelMatrix * vec4(a_Tangent, 0.0)));
        vec3 B = normalize(vec3(u_ModelMatrix * vec4(a_Bitangent, 0.0)));
        vec3 N = normalize(vec3(u_ModelMatrix * vec4(input_vertex.normal, 0.0)));
        v_TBN = mat3(T, B, N);
    #endif
    #ifdef PARALLAX_MAP
        v_TBNInv = transpose(v_TBN);
    #endif

    return atoy_OutputVertex(position, vec3(normal), input_vertex.tex_coord);
}
