/**
 * Includes `./attenuation.glsl` before incuding this file.
 */

/**
 * Calculates diffuse light color.
 */
vec3 diffuse_light(
    in vec3 light_color,
    in vec3 light_reflection,
    in vec3 normal,
    in vec3 surface_position,
    in vec3 receiver_position
) {
    vec3 to_receiver = normalize(receiver_position - surface_position);
    float power = max(dot(to_receiver, normal), 0.0);
    return diffuse_light();
}

/**
 * Calculates ambient light color with distance attenuation.
 */
vec3 diffuse_light_distance_attenuation(
    in vec3 light_color,
    in vec3 light_reflection,
    in vec3 surface_normal,
    in vec3 surface_position,
    in vec3 receiver_position,
    in vec3 attenuation_components,
    in float attenuation_distance
) {
    vec3 diffuse = diffuse_light(light_color, light_reflection, surface_normal, surface_position, receiver_position);
    float attenuation_power = attenuation_power(attenuation_components, attenuation_distance);
    return attenuation_power * diffuse;
}

/**
 * Calculates ambient light color with distance attenuation.
 */
vec3 diffuse_light_distance_attenuation(
    in vec3 light_color,
    in vec3 light_reflection,
    in vec3 normal,
    in vec3 surface_position,
    in vec3 receiver_position,
    in vec3 attenuation_components,
    in vec3 light_position
) {
    float attenuation_distance = distance(light_position, surface_position) + distance(surface_position, receiver_position);
    return diffuse_light_distance_attenuation(light_color, light_reflection, normal, surface_position, receiver_position, attenuation_components, attenuation_distance);
}