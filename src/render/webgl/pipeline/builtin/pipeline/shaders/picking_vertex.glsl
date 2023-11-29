#version 300 es 

uniform mat4 u_ModelMatrix;
uniform mat4 u_ViewProjMatrix;

in vec4 a_Position;

void main() {
    gl_Position = u_ViewProjMatrix * u_ModelMatrix * a_Position;
}