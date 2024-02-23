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

in vec3 v_Position;

#ifdef USE_POSITION_EYE_SPACE
in vec3 v_PositionES;
#endif

#ifdef USE_NORMAL
in vec3 v_Normal;

    #ifdef USE_TBN
in mat3 v_TBN;

        #ifdef USE_TBN_INVERT
in mat3 v_TBNInvert;
        #endif
    #endif
#endif

#ifdef USE_TEXTURE_COORDINATE
in vec2 v_TexCoord;
#endif