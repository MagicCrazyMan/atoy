in vec2 v_TexCoord;

uniform sampler2D u_PositionsAndSpecularShininessTexture;
uniform sampler2D u_NormalsTexture;
uniform sampler2D u_AlbedoAndTransparencyTexture;

out vec4 o_Color;

void main() {
    vec4 position_and_specular_shininess = texture(u_PositionsAndSpecularShininessTexture, v_TexCoord);
    vec3 position = position_and_specular_shininess.xyz;
    float specular_shininess = position_and_specular_shininess.w;

    vec3 normal = texture(u_NormalsTexture, v_TexCoord).xyz;

    vec4 albedo_and_transparency = texture(u_AlbedoAndTransparencyTexture, v_TexCoord);
    vec3 albedo = albedo_and_transparency.xyz;
    float transparency = albedo_and_transparency.a;

    #ifdef LIGHTING
    atoy_LightingFragment lighting_fragment = atoy_LightingFragment(position, normal);
    atoy_LightingMaterial lighting_material = atoy_LightingMaterial(albedo, albedo, albedo, specular_shininess);
    vec3 color = atoy_lighting(u_CameraPosition, lighting_fragment, lighting_material);
    #else
    vec3 color = albedo;
    #endif

    o_Color = vec4(color, transparency);
}