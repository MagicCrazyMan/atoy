/**
 * Standard Fragment Shader Lighting Source Code.
 */

#ifdef LIGHTING
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
    float cos_theta = max(dot(r, to_camera), 0.0);
    float power = pow(cos_theta, shininess);
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
    float cos_theta = max(dot(h, frag_normal), 0.0);
    float power = pow(cos_theta, shininess);
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
 * Lighting fragment difinition.
 * 
 * - `position`: Fragment position in WORLD space.
 * - `normal`: Normal vector in WORLD space.
 */
struct atoy_LightingFragment {
    vec3 position;
    vec3 normal;
};

/**
 * Lighting material difinition.
 * 
 * - `ambient`: Ambient color of the material.
 * - `diffuse`: Diffuse color of the material.
 * - `specular`: Specular color of the material.
 * - `specular_shininess`: Specular shininess of the material.
 */
struct atoy_LightingMaterial {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    float specular_shininess;
};

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
 */
struct atoy_PointLight {
    vec3 position;
    bool enabled;
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

/**
 * Directional light definition.
 * 
 * - `direction`: Light direction.
 * - `enabled`: Is light enabled.
 * - `ambient`: Light ambient color.
 * - `diffuse`: Light diffuse color.
 * - `specular`: Light specular color.
 */
struct atoy_DirectionalLight {
    vec3 direction;
    bool enabled;
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

/**
 * Spot light definition.
 * 
 * - `direction`: Light direction.
 * - `enabled`: Is light enabled.
 * - `position`: Light position.
 * - `ambient`: Light ambient color.
 * - `inner_cutoff`: Inner cutoff in cosine value for smooth lighting.
 * - `diffuse`: Light diffuse color.
 * - `outer_cutoff`: Outer cutoff in cosine value for smooth lighting.
 * - `specular`: Light specular color.
 */
struct atoy_SpotLight {
    vec3 direction;
    bool enabled;
    vec3 position;
    vec3 ambient;
    float inner_cutoff;
    vec3 diffuse;
    float outer_cutoff;
    vec3 specular;
};

/**
 * Area light definition.
 * 
 * - `direction`: Light direction.
 * - `enabled`: Is light enabled.
 * - `up`: Light upward direction.
 * - `inner_width`: Light inner width for smooth lighting.
 * - `right`: Light rightward direction.
 * - `inner_height`: Light inner height for smooth lighting.
 * - `position`: Light position.
 * - `offset`: Light offset.
 * - `ambient`: Light ambient color.
 * - `outer_width`: Light outer width for smooth lighting.
 * - `diffuse`: Light diffuse color.
 * - `outer_height`: Light outer height for smooth lighting.
 * - `specular`: Light specular color.
 */
struct atoy_AreaLight {
    vec3 direction;
    bool enabled;
    vec3 up;
    float inner_width;
    vec3 right;
    float inner_height;
    vec3 position;
    float offset;
    vec3 ambient;
    float outer_width;
    vec3 diffuse;
    float outer_height;
    vec3 specular;
};

/**
 * Uniform block providing global lights.
 */
layout(std140) uniform atoy_Lights {
    vec3 u_Attenuations;
    atoy_AmbientLight u_AmbientLight;
    atoy_DirectionalLight u_DirectionalLights[DIRECTIONAL_LIGHTS_COUNT];
    atoy_PointLight u_PointLights[POINT_LIGHTS_COUNT];
    atoy_SpotLight u_SpotLights[SPOT_LIGHTS_COUNT];
    atoy_AreaLight u_AreaLights[AREA_LIGHTS_COUNT];
};

/**
 * Applies `atoy_AmbientLight` to a lighting result.
 */
void atoy_ambient_lighting(atoy_AmbientLight light, atoy_LightingMaterial material, inout vec3 lighting) {
    if(!u_AmbientLight.enabled) {
        return;
    }

    lighting += atoy_ambient(light.color, material.ambient);
}

/**
 * Applies `atoy_DirectionalLight` to a lighting result.
 */
void atoy_directional_lighting(atoy_DirectionalLight light, atoy_LightingFragment fragment, atoy_LightingMaterial material, vec3 to_camera, inout vec3 lighting) {
    if(!light.enabled) {
        return;
    }

    vec3 n = vec3(0.0, 1.0, 0.0);

    vec3 color = vec3(0.0);
    color += atoy_ambient(light.ambient, material.ambient);
    color += atoy_diffuse(light.diffuse, material.diffuse, fragment.normal, to_camera);
    // for directional light, skip specular lighting if incident of lighting is perpendicular with surface normal
    if(max(dot(-light.direction, fragment.normal), 0.0) != 0.0) {
        color += atoy_specular_phong(light.specular, material.specular_shininess, material.specular, fragment.normal, -light.direction, to_camera);
    }

    lighting += color;
}

/**
 * Applies `atoy_PointLight` to a lighting result.
 */
void atoy_point_lighting(atoy_PointLight light, atoy_LightingFragment fragment, atoy_LightingMaterial material, vec3 to_camera, inout vec3 lighting) {
    if(!light.enabled) {
        return;
    }

    vec3 to_light = light.position - fragment.position;
    float light_distance = length(to_light);
    to_light = normalize(to_light);

    vec3 color = vec3(0.0);
    color += atoy_ambient(light.ambient, material.ambient);
    color += atoy_diffuse(light.diffuse, material.diffuse, fragment.normal, to_camera);
    color += atoy_specular_phong(light.specular, material.specular_shininess, material.specular, fragment.normal, to_light, to_camera);

    float attenuation = atoy_attenuation_power(u_Attenuations.x, u_Attenuations.y, u_Attenuations.z, light_distance);
    color *= attenuation;

    lighting += color;
}

/**
 * Applies `atoy_SpotLight` to a lighting result.
 */
void atoy_spot_lighting(atoy_SpotLight light, atoy_LightingFragment fragment, atoy_LightingMaterial material, vec3 to_camera, inout vec3 lighting) {
    if(!light.enabled) {
        return;
    }

    vec3 to_light = light.position - fragment.position;
    float light_distance = length(to_light);
    to_light = normalize(to_light);

    // skips out of outer cutoff
    float cos_theta = dot(-to_light, light.direction);
    if(cos_theta < light.outer_cutoff) {
        return;
    }

    vec3 color = vec3(0.0);
    color += atoy_ambient(light.ambient, material.ambient);
    color += atoy_diffuse(light.diffuse, material.diffuse, fragment.normal, to_camera);
    color += atoy_specular_phong(light.specular, material.specular_shininess, material.specular, fragment.normal, to_light, to_camera);

    float attenuation = atoy_attenuation_power(u_Attenuations.x, u_Attenuations.y, u_Attenuations.z, light_distance);
    color *= attenuation;

    // applies smooth lighting by clamping inner and outer cutoff
    if(cos_theta < light.inner_cutoff) {
        float intensity = clamp((light.outer_cutoff - cos_theta) / (light.outer_cutoff - light.inner_cutoff), 0.0, 1.0);
        color *= intensity;
    }

    lighting += color;
}

/**
 * Applies `atoy_AreaLight` to a lighting result.
 */
void atoy_area_lighting(atoy_AreaLight light, atoy_LightingFragment fragment, atoy_LightingMaterial material, vec3 to_camera, inout vec3 lighting) {
    if(!light.enabled) {
        return;
    }

    vec3 to_light = light.position - fragment.position;
    float light_distance = length(to_light);
    to_light = normalize(to_light);
    vec3 from_light = -to_light;

    float cos_theta = dot(light.direction, from_light);
    if(cos_theta < 0.0) {
        return;
    }

    vec3 pop = light.position + light.direction * light.offset;
    float how = light.outer_width / 2.0;
    float hoh = light.outer_height / 2.0;

    float h = light.offset / cos_theta;
    float d = light_distance - h;
    vec3 p = fragment.position + d * to_light;

    vec3 v = p - pop;
    float x = abs(dot(v, light.right));
    float y = abs(dot(v, light.up));
    if(x > how || y > hoh) {
        return;
    }

    vec3 color = vec3(0.0);
    color += atoy_ambient(light.ambient, material.ambient);
    color += atoy_diffuse(light.diffuse, material.diffuse, fragment.normal, to_camera);
    color += atoy_specular_phong(light.specular, material.specular_shininess, material.specular, fragment.normal, to_light, to_camera);

    float attenuation = atoy_attenuation_power(u_Attenuations.x, u_Attenuations.y, u_Attenuations.z, light_distance);
    color *= attenuation;

    float intensity = 1.0;
    float hiw = light.inner_width / 2.0;
    float hih = light.inner_height / 2.0;
    if(x > hiw) {
        float ix = clamp((how - x) / (how - hiw), 0.0, 1.0);
        intensity = min(ix, intensity);
    }
    if(y > hih) {
        float iy = clamp((hoh - y) / (hoh - hih), 0.0, 1.0);
        intensity = min(iy, intensity);
    }
    color *= intensity;

    lighting += color;
}

/**
 * Calculates scene mixed lighting.
 */
vec3 atoy_lighting(vec3 camera_position, atoy_LightingFragment fragment, atoy_LightingMaterial material) {
    vec3 to_camera = normalize(camera_position - fragment.position);

    vec3 lighting = vec3(0.0);

    // ambient light
    atoy_ambient_lighting(u_AmbientLight, material, lighting);

    // directional lights
    for(int i = 0; i < DIRECTIONAL_LIGHTS_COUNT; i++) {
        atoy_directional_lighting(u_DirectionalLights[i], fragment, material, to_camera, lighting);
    }

    // point lights
    for(int i = 0; i < POINT_LIGHTS_COUNT; i++) {
        atoy_point_lighting(u_PointLights[i], fragment, material, to_camera, lighting);
    }

    // spot lights 
    for(int i = 0; i < SPOT_LIGHTS_COUNT; i++) {
        atoy_spot_lighting(u_SpotLights[i], fragment, material, to_camera, lighting);
    }

    // area lights 
    for(int i = 0; i < AREA_LIGHTS_COUNT; i++) {
        atoy_area_lighting(u_AreaLights[i], fragment, material, to_camera, lighting);
    }

    return lighting;
}
#endif