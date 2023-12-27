/**
 * Standard Fragment Shader Lighting Source Code.
 */

/**
 * Calculates ambient light effect.
 */
vec3 atoy_ambient(vec3 color, vec3 ambient) {
    return color * ambient;
}

/**
 * Calculates diffuse light effect.
 * 
 * `frag_normal` and `to_camera` should be normalized.
 */
vec3 atoy_diffuse(
    vec3 color,
    vec3 diffuse,
    vec3 frag_normal,
    vec3 to_camera
) {
    float power = dot(frag_normal, to_camera);
    power = max(power, 0.0);
    return power * color * diffuse;
}

/**
 * Calculates Phong Shading specular light effect.
 * 
 * `frag_normal`, `to_light` and `to_camera` should be normalized.
 */
vec3 atoy_specular_phong(
    vec3 color,
    float shininess,
    vec3 specular,
    vec3 frag_normal,
    vec3 to_light,
    vec3 to_camera
) {
    vec3 r = reflect(-to_light, frag_normal);
    float theta = max(dot(r, to_camera), 0.0);
    float power = pow(theta, shininess);
    return power * color * specular;
}

/**
 * Calculates Blinn-Phong Shading specular light effect.
 * 
 * `frag_normal`, `from_light` and `to_camera` should be normalized.
 */
vec3 atoy_specular_blinn_phong(
    vec3 color,
    float shininess,
    vec3 specular,
    vec3 frag_normal,
    vec3 to_light,
    vec3 to_camera
) {
    vec3 h = normalize(to_light + to_camera); // halfway vector
    float theta = max(dot(h, frag_normal), 0.0);
    float power = pow(theta, shininess);
    return power * color * specular;
}

/**
 * Calculates attenuation power by given a distance.
 */
float atoy_attenuation_power(
    float a,
    float b,
    float c,
    float distance
) {
    float denominator = a + b * distance + c * pow(distance, 2.0);
    return denominator == 0.0 ? 1.0 : 1.0 / denominator;
}

/**
 * Calculates attenuation power by given light position, surface position and receiver position.
 */
float atoy_attenuation_power(
    float a,
    float b,
    float c,
    vec3 light_position,
    vec3 surface_position,
    vec3 receiver_position
) {
    float total_distance = distance(light_position, surface_position) + distance(receiver_position, surface_position);
    return atoy_attenuation_power(a, b, c, total_distance);
}

/**
 * Ambient light definition.
 * 
 * - `color`: Light color.
 * - `enabled`: Is light enabled.
 */
struct atoy_AmbientLight {
    vec3 color;
    bool enabled;
};

/**
 * Point light definition.
 * 
 * - `position`: Light position.
 * - `enabled`: Is light enabled.
 * - `ambient`: Light ambient color.
 * - `diffuse`: Light diffuse color.
 * - `specular`: Light specular color.
 * - `specular_shininess`: Specular light shininess.
 */
struct atoy_PointLight {
    vec3 position;
    bool enabled;
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    float specular_shininess;
};

/**
 * Directional light definition.
 * 
 * - `direction`: Light direction.
 * - `enabled`: Is light enabled.
 * - `ambient`: Light ambient color.
 * - `diffuse`: Light diffuse color.
 * - `specular`: Light specular color.
 * - `specular_shininess`: Specular light shininess.
 */
struct atoy_DirectionalLight {
    vec3 direction;
    bool enabled;
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    float specular_shininess;
};

/**
 * Spot light definition.
 * 
 * - `position`: Light position.
 * - `enabled`: Is light enabled.
 * - `direction`: Light direction.
 * - `ambient`: Light ambient color.
 * - `inner_cutoff`: Inner cutoff for smooth lighting.
 * - `diffuse`: Light diffuse color.
 * - `outer_cutoff`: Outer cutoff for smooth lighting.
 * - `specular`: Light specular color.
 * - `specular_shininess`: Specular light shininess.
 */
struct atoy_SpotLight {
    vec3 direction;
    vec3 position;
    bool enabled;
    vec3 ambient;
    float inner_cutoff;
    vec3 diffuse;
    float outer_cutoff;
    vec3 specular;
    float specular_shininess;
};

/**
 * Uniform block providing global lights.
 */
layout(std140) uniform atoy_Lights {
    vec3 u_Attenuations;
    atoy_AmbientLight u_AmbientLight;
    atoy_DirectionalLight u_DirectionalLights[12];
    atoy_PointLight u_PointLights[12];
    atoy_SpotLight u_SpotLights[12];
};

/**
 * Calculates scene mixed lighting.
 */
vec3 atoy_lighting(atoy_InputFrag frag, atoy_OutputMaterial material) {
    vec3 to_camera = normalize(u_CameraPosition - frag.position_ws);

    vec3 lighting = vec3(0.0);

    // ambient light
    if(u_AmbientLight.enabled) {
        lighting += atoy_ambient(u_AmbientLight.color, material.ambient);
    }

    // directional lights
    for(int i = 0; i < 12; i++) {
        atoy_DirectionalLight light = u_DirectionalLights[i];
        if(light.enabled) {
            vec3 to_light = -light.direction;

            vec3 color = vec3(0.0);
            color += atoy_ambient(light.ambient, material.ambient);
            color += atoy_diffuse(light.diffuse, material.diffuse, frag.normal_ws, to_camera);
            color += atoy_specular_phong(light.specular, light.specular_shininess, material.specular, frag.normal_ws, to_light, to_camera);

            lighting += color;
        }
    }

    // point lights
    for(int i = 0; i < 12; i++) {
        atoy_PointLight light = u_PointLights[i];
        if(light.enabled) {
            vec3 to_light = light.position - frag.position_ws;
            float to_light_distance = length(to_light);
            to_light = normalize(to_light);

            vec3 color = vec3(0.0);
            color += atoy_ambient(light.ambient, material.ambient);
            color += atoy_diffuse(light.diffuse, material.diffuse, frag.normal_ws, to_camera);
            color += atoy_specular_phong(light.specular, light.specular_shininess, material.specular, frag.normal_ws, to_light, to_camera);

            float attenuation = atoy_attenuation_power(u_Attenuations.x, u_Attenuations.y, u_Attenuations.z, to_light_distance);
            color *= attenuation;

            lighting += color;
        }
    }

    // spot loghts 
    // todo!

    return lighting;
}