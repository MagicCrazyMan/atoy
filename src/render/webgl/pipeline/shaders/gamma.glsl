#ifdef GAMMA_CORRECTION
uniform float u_Gamma;

/**
 * Calculates gamma correction to a specified color.
 */
vec3 atoy_gamma_correction(vec3 color, float gamma) {
    if(gamma == 0.0f) {
        return vec3(0.0f);
    } else {
        return pow(color, vec3(1.0f / gamma));
    }
}

// /**
//  * Calculates gamma to a specified color.
//  */
// vec3 atoy_gamma(vec3 color, float gamma) {
//     return pow(color, vec3(gamma));
// }
#endif