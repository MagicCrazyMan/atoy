/**
 * Standard Fragment Shader Entry Source Code.
 *
 * Fucntion `atoy_Material atoy_process_frag(atoy_Frag)` MUST be filled.
 */

void main() {
    vec3 normal_ws;
    if(gl_FrontFacing) {
        normal_ws = normalize(v_NormalWS);
    } else {
        normal_ws = normalize(-v_NormalWS);
    }
    atoy_Frag frag = atoy_Frag(u_Transparency, v_PositionWS, v_PositionES, v_PositionCS, normal_ws, v_TexCoord);
    atoy_Material material = atoy_process_frag(frag);

    vec3 color;
    if(u_EnableLighting) {
        color = atoy_lighting(frag, material);
    } else {
        color = material.diffuse;
    }

    o_Color = vec4(color, material.transparency);
    o_Index = u_Index;
    o_Position = uvec3(floatBitsToUint(v_PositionWS.x), floatBitsToUint(v_PositionWS.y), floatBitsToUint(v_PositionWS.z));;
}
