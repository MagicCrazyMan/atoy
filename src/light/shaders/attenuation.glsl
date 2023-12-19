/**
 * Calculates attenuation power.
 */
float attenuation_power(in vec3 attenuation_components, in float attenuation_distance) {
    float value = attenuation_components.x + attenuation_components.y * attenuation_distance + attenuation_components.z * pow(attenuation_distance, 2.0);
    if(value == 0.0) {
        return 0.0;
    } else {
        return 1.0 / value;
    }
}