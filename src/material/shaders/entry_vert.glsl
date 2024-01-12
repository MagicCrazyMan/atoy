/**
 * Standard Vertex Shader Entry Source Code.
 */


void main() {
    atoy_InputVertex input_vertex = atoy_InputVertex(a_Position, a_Normal, a_TexCoord);
    atoy_OutputVertex output_vertex = atoy_process_vertex(input_vertex);

    vec4 position_ws = output_vertex.position;
    vec4 position_es = u_ViewMatrix * position_ws;
    vec4 position_cs = u_ProjMatrix * position_es;

    v_PositionWS = vec3(position_ws);
    v_PositionES = vec3(position_es);
    v_PositionCS = vec3(position_cs / position_cs.w);
    v_NormalWS = output_vertex.normal;
    v_TexCoord = output_vertex.tex_coord;
    gl_Position = position_cs;
}
