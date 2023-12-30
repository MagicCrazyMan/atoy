/**
 * Standard Fragment Shader Gamma Correction Source Code.
 */

/**
 * Calculates gamma correction to a specified color.
 */
vec3 atoy_gamma_correction(vec3 color, float gamma_correction) {
    return pow(color, vec3(gamma_correction));
}

/**
 * Calculates gamma correction to a specified color.
 */
vec3 atoy_gamma_correction_inverse(vec3 color, float gamma_correction_inverse) {
    return pow(color, vec3(gamma_correction_inverse));
}
