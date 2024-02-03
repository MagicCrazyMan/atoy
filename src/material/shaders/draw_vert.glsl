/**
 * Standard Vertex Shader Draw Source Code.
 *
 * Fucntion `atoy_OutputVertex atoy_build_vertex(atoy_InputVertex)` MUST be filled.
 */
 
in vec4 a_Position;
in vec3 a_Normal;
in vec2 a_TexCoord;

out vec3 v_PositionWS;
out vec3 v_PositionES;
out vec3 v_PositionCS;
out vec3 v_NormalWS;
out vec2 v_TexCoord;

void main() {
    atoy_InputVertex input_vertex = atoy_InputVertex(a_Position, a_Normal, a_TexCoord);
    atoy_OutputVertex output_vertex = atoy_build_vertex(input_vertex);

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
