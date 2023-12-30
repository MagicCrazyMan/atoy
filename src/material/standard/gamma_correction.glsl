/**
 * Standard Fragment Shader Gamma Correction Source Code.
 */

vec3 GAMMA_CORRECTION = vec3(0.454545f);

/**
 * Calculates gamma correction to a specified color.
 */
vec3 atoy_gamma_correction(vec3 color) {
    return pow(color, GAMMA_CORRECTION);
}
