/**
 * Standard Fragment Shader Constants Source Code.
 */

in vec3 v_PositionWS;
in vec3 v_PositionES;
in vec3 v_PositionCS;
in vec3 v_NormalWS;
in vec2 v_TexCoord;

/**
 * Input fragment difinition.
 * 
 * - `transparency`: Fragment transparency.
 * - `position_ws`: Fragment position in WORLD space.
 * - `position_es`: Fragment position in EYE space.
 * - `position_cs`: Fragment position in CLIP space.
 * - `normal_ws`: Normal vector in WORLD space.
 * - `tex_coord`: Texture coordinate associated with.
 */
struct atoy_Fragment {
    vec3 position_ws;
    vec3 position_es;
    vec3 position_cs;
    vec3 normal_ws;
    vec2 tex_coord;
};

/**
 * Output material difinition.
 * 
 * - `transparency`: Transparency of the material.
 * - `albedo`: Albedo color of the material.
 * - `ambient`: Ambient color of the material.
 * - `diffuse`: Diffuse color of the material.
 * - `specular`: Specular color of the material.
 * - `specular_shininess`: Specular shininess of the material.
 */
struct atoy_Material {
    float transparency;
    vec3 albedo;
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    float specular_shininess;
};
/**
 * Build atoy_Fragment from varyings and uniforms.
 */
atoy_Fragment atoy_build_fragment() {
    vec3 normal_ws;
    if(gl_FrontFacing) {
        normal_ws = normalize(v_NormalWS);
    } else {
        normal_ws = normalize(-v_NormalWS);
    }
    return atoy_Fragment(v_PositionWS, v_PositionES, v_PositionCS, normal_ws, v_TexCoord);
}