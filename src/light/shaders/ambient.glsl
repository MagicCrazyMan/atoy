/**
 * Calculates ambient light.
 */
vec3 ambient_light(in vec3 light_color, in vec3 light_reflection) {
    return light_color * light_reflection;
}
