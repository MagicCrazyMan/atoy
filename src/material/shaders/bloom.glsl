/**
 * Standard Shader Bloom Color Source Code.
 */

#ifdef BLOOM
/**
 * Checks wether input color is bloom color.
 */
bool atoy_is_bloom_color(vec3 color, vec3 bloom_threshold) {
    float brightness = dot(color, bloom_threshold);
    return brightness > 1.0f;
}
#endif