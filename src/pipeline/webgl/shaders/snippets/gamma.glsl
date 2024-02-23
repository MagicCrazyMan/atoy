/**
 * Standard Gamma Correction Code Snippet.
 */

uniform float u_Gamma;

/**
 * Calculates gamma correction to a specified color.
 */
vec3 atoy_gamma_correction(vec3 color, float gamma) {
    if(gamma == 0.0) {
        return vec3(0.0);
    } else {
        return pow(color, vec3(1.0 / gamma));
    }
}

// /**
//  * Calculates gamma to a specified color.
//  */
// vec3 atoy_gamma(vec3 color, float gamma) {
//     return pow(color, vec3(gamma));
// }