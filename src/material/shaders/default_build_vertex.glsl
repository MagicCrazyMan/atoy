/**
 * Standard Vertex Shader Build Vertex Source Code.
 */

uniform mat4 u_ModelMatrix;
uniform mat4 u_NormalMatrix;

atoy_OutputVertex atoy_build_vertex(atoy_InputVertex input_vertex) {
    vec4 position = u_ModelMatrix * input_vertex.position;
    vec4 normal = u_NormalMatrix * vec4(input_vertex.normal, 0.0);

    return atoy_OutputVertex(position, vec3(normal), input_vertex.tex_coord);
}
