#version 300 es

in vec4 a_Position;

uniform bool u_ShouldScale;
uniform uint u_OutlineWidth;
uniform uvec2 u_CanvasSize;

uniform mat4 u_ModelMatrix;
uniform mat4 u_ViewProjMatrix;

void main() {
    mat4 mvp = u_ViewProjMatrix * u_ModelMatrix;
    vec4 position = mvp * a_Position;
    if (u_ShouldScale) {
        float outline_width = float(u_OutlineWidth);
        float canvas_width = float(u_CanvasSize.x);

        float w = position.w;
        float sx = 1.0 + w * outline_width / (canvas_width / 2.0);
        mat4 scale = mat4(
            sx, 0.0, 0.0, 0.0,
            0.0, sx, 0.0, 0.0,
            0.0, 0.0, sx, 0.0,
            0.0, 0.0, 0.0, 1.0
        );
        
        // move entity back to origin and then scale
        gl_Position = mvp * scale * a_Position;
    } else {
        gl_Position = position;
    }
}