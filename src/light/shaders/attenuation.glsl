/**
 * Calculates attenuation power.
 */
float attenuation_power(in vec3 components, in float distance) {
    float value = components.x + components.y * distance + components.z * pow(distance, 2.0);
    return value == 0.0 ? 1.0 : 1.0 / value;
}