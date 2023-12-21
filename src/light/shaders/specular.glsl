/**
 * Includes `./attenuation.glsl` before incuding this file.
 */

/**
 * Specular light difinition.
 */
struct SpecularLight {
                        // base alignment
    vec3 color;         // 16
    vec3 position;      // 16
    vec3 attenuations;  // 12 (Merged)
    float shininess;    // 4  (Merged)
    bool enabled;       // 4
};

/**
 * Specular light usinig Phong shading.
 */
vec3 specular_light(
    in SpecularLight light,
    in vec3 reflection,
    in vec3 surface_normal,
    in vec3 surface_position,
    in vec3 receiver_position
) {
    if(light.enabled) {
        vec3 to_receiver = normalize(receiver_position - surface_position);
        vec3 from_light = normalize(surface_position - light.position);
        vec3 reflect = reflect(from_light, surface_normal);
        float power = max(dot(reflect, to_receiver), 0.0);
        power = pow(power, light.shininess);
        return light.color * reflection * power;
    } else {
        return vec3(0.0);
    }
}

/**
 * Specular light usinig Phong shading with distance attenuation.
 */
vec3 specular_light_attenuation(
    in SpecularLight light,
    in vec3 reflection,
    in vec3 surface_normal,
    in vec3 surface_position,
    in vec3 receiver_position
) {
    if(light.enabled) {
        float attenuation_distance = distance(light.position, surface_position) + distance(surface_position, receiver_position);
        float attenuation_power = attenuation_power(light.attenuations, attenuation_distance);
        vec3 specular = specular_light(light, reflection, surface_normal, surface_position, receiver_position);
        return specular * attenuation_power;
    } else {
        return vec3(0.0);
    }
}

layout(std140) uniform SpecularLights {
    SpecularLight u_SpecularLights[12]; // align to 64 bytes
};

/**
 * Applies all specular lights in `SpecularLights`.
 */
vec3 specular_lights(
    in vec3 reflection,
    in vec3 surface_normal,
    in vec3 surface_position,
    in vec3 receiver_position
) {
    vec3 color = vec3(0.0);
    for(int i = 0; i < 12; i++) {
        SpecularLight light = u_SpecularLights[i];
        color += specular_light_attenuation(light, reflection, surface_normal, surface_position, receiver_position);
    }
    return color;
}