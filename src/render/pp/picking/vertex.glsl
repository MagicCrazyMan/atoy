#version 300 es 

in vec4 a_Position;

uniform mat4 u_ModelMatrix;
uniform mat4 u_ViewProjMatrix;

void main() {
    gl_Position = u_ViewProjMatrix * u_ModelMatrix * a_Position;
}