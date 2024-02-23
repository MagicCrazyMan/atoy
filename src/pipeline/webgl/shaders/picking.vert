#version 300 es 

in vec4 a_Position;

uniform mat4 u_ModelMatrix;
uniform mat4 u_ViewProjMatrix;

out vec3 v_Position;

void main() {
    vec4 position = u_ModelMatrix * a_Position;
    v_Position = vec3(position);
    gl_Position = u_ViewProjMatrix * position;
}
