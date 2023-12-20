/**
 * Ambient light difinition.
 */
struct AmbientLight {
    bool enabled;
    vec3 color;
};

/**
 * Calculates ambient light.
 */
vec3 ambient_light(in AmbientLight light, in vec3 reflection) {
    if (light.enabled) {
        return light.color * reflection;
    } else {
        return vec3(0.0);
    }
}
