#version 300 es 

in vec4 a_Position;

uniform mat4 u_ModelMatrix;
uniform mat4 u_ViewProjMatrix;

out vec3 v_Position;

void main() {
    vec4 world_position = u_ModelMatrix * a_Position;
    v_Position = vec3(world_position);
    gl_Position = u_ViewProjMatrix * world_position;
}