vec3 ambient_light(in vec3 ambient_reflection, in vec3 ambient_light_color) {
    return ambient_reflection * ambient_light_color;
}