/**
 * Ambient light difinition.
 */
struct AmbientLight {
    bool enabled;
    vec3 color;
};

/**
 * Calculates ambient light.
 * If light disabled, returns reflection itself.
 */
vec3 ambient_light(in AmbientLight light, in vec3 reflection) {
    if (light.enabled) {
        return light.color * reflection;
    } else {
        return reflection;
    }
}
