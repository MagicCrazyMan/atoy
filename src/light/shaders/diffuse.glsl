/**
 * Includes `./attenuation.glsl` before incuding this file.
 */

/**
 * Diffuse light difinition.
 */
struct DiffuseLight {
                       // base alignment
    vec3 color;        // 16
    vec3 position;     // 16
    vec3 attenuations; // 12(Merged)
    bool enabled;      // 4 (Merged)
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
    DiffuseLight u_DiffuseLights[12]; // align to 48 bytes
};

/**
 * Applies all diffuse lights in `DiffuseLights`.
 */
vec3 diffuse_lights(
    in vec3 reflection,
    in vec3 surface_normal,
    in vec3 surface_position,
    in vec3 receiver_position
) {
    vec3 color = vec3(0.0);
    for(int i = 0; i < 12; i++) {
        DiffuseLight light = u_DiffuseLights[i];
        color += diffuse_light_attenuation(light, reflection, surface_normal, surface_position, receiver_position);
    }
    return color;
}