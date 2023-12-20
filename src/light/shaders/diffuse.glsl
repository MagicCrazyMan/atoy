/**
 * Includes `./attenuation.glsl` before incuding this file.
 */

/**
 * Diffuse light difinition.
 */
struct DiffuseLight {
    bool enabled;
    vec3 color;
    vec3 position;
    vec3 attenuations;
};

/**
 * Calculates diffuse light color.
 */
vec3 diffuse_light(
    in DiffuseLight light,
    in vec3 reflection,
    in vec3 surface_normal,
    in vec3 surface_position,
    in vec3 receiver_position
) {
    if(light.enabled) {
        vec3 to_receiver = normalize(receiver_position - surface_position);
        float power = max(dot(to_receiver, surface_normal), 0.0);
        return light.color * reflection * power;
    } else {
        return vec3(0.0);
    }
}

/**
 * Calculates ambient light color with distance attenuation.
 */
// vec3 diffuse_light_distance_attenuation(
//     in DiffuseLight light,
//     in vec3 light_reflection,
//     in vec3 surface_normal,
//     in vec3 surface_position,
//     in vec3 receiver_position,
//     in vec3 attenuation_components,
//     in float attenuation_distance
// ) {
//     if (light.enabled) {
//         vec3 diffuse = diffuse_light(light, light_reflection, surface_normal, surface_position, receiver_position);
//         float attenuation_power = attenuation_power(attenuation_components, attenuation_distance);
//         return attenuation_power * diffuse;
//     } else {
//         return vec3(0.0);
//     }
// }

/**
 * Calculates ambient light color with distance attenuation.
 */
vec3 diffuse_light_attenuation(
    in DiffuseLight light,
    in vec3 reflection,
    in vec3 surface_normal,
    in vec3 surface_position,
    in vec3 receiver_position
) {
    if(light.enabled) {
        float attenuation_distance = distance(light.position, surface_position) + distance(surface_position, receiver_position);
        float attenuation_power = attenuation_power(light.attenuations, attenuation_distance);
        vec3 diffuse = diffuse_light(light, reflection, surface_normal, surface_position, receiver_position);
        return attenuation_power * diffuse;
    } else {
        return vec3(0.0);
    }
}

layout(std140) uniform DiffuseLights {
    int u_DiffuseLightCount;
    DiffuseLight u_Lights[12];
};

vec3 diffuse_lights(
    in vec3 reflection,
    in vec3 surface_normal,
    in vec3 surface_position,
    in vec3 receiver_position
) {
    vec3 color = vec3(0.0);
    for(int i = 0; i < min(u_DiffuseLightCount, 12); i++) {
        DiffuseLight light = u_Lights[i];
        color += diffuse_light_attenuation(light, reflection, surface_normal, surface_position, receiver_position);
    }
    return color;
}