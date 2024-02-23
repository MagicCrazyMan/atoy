/**
 * Frgament difinition for fragment.
 * 
 * - `position`: Position in WORLD space of this fragment.
 * - `normal`: Normal of this position in WORLD space of this fragment.
 * - `albedo`: Albedo of this fragment.
 * - `shininess`: Specular shininess of this fragment.
 * - `transparency`: Transparency of this fragment.
 */
struct atoy_Fragment {
    vec3 position;
    vec3 normal;
    vec3 albedo;
    float shininess;
    float transparency;
};