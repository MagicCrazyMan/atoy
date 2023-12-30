#version 300 es

out vec2 v_TexCoord;

void main() {
    switch(gl_VertexID) {
        case 0: {
            gl_Position = vec4(1.0f, -1.0f, 0.0f, 1.0f);
            v_TexCoord = vec2(1.0f, 0.0f);
            break;
        }
        case 1: {
            gl_Position = vec4(1.0f, 1.0f, 0.0f, 1.0f);
            v_TexCoord = vec2(1.0f, 1.0f);
            break;
        }
        case 2: {
            gl_Position = vec4(-1.0f, 1.0f, 0.0f, 1.0f);
            v_TexCoord = vec2(0.0f, 1.0f);
            break;
        }
        default: {
            gl_Position = vec4(-1.0f, -1.0f, 0.0f, 1.0f);
            v_TexCoord = vec2(0.0f, 0.0f);
            break;
        }
    }
}
