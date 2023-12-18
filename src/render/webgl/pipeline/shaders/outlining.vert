#version 300 es

in vec4 a_Position;
in vec2 a_TexCoord;

uniform int u_StageVertex;
uniform mat4 u_ModelMatrix;
uniform mat4 u_ViewProjMatrix;

out vec2 v_TexCoord;

void main() {
    if(u_StageVertex == 0 || u_StageVertex == 2) {
        gl_Position = u_ViewProjMatrix * u_ModelMatrix * a_Position;
    } else {
        gl_Position = a_Position;
        v_TexCoord = a_TexCoord;
    }
}