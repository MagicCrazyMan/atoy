use super::{
    attribute::WebGlAttributeValue,
    context::WebGlContext,
    program::WebGlShaderSource,
    uniform::{WebGlUniformBlockValue, WebGlUniformValue},
};

pub trait WebGlMaterial {
    /// Returns vertex shader source for this material.
    fn vertex_shader(&self) -> &dyn WebGlShaderSource;

    /// Returns fragment shader source for this material.
    fn fragment_shader(&self) -> &dyn WebGlShaderSource;

    /// Returns attribute value of a specified GLSL attribute variable name.
    fn attribute_value(&self, name: &str) -> Option<WebGlAttributeValue>;

    /// Returns uniform value of a specified GLSL uniform variable name.
    fn uniform_value(&self, name: &str) -> Option<WebGlUniformValue>;

    /// Returns uniform block value of a specified GLSL uniform block layout variable name.
    fn uniform_block_value(&self, name: &str) -> Option<WebGlUniformBlockValue>;

    /// A method invoked before rendering.
    /// Renderer always invokes this method no matter this material is ready or not.
    /// Developer should prepare the material (fetching data from remote, loading resources etc.) in this method.
    fn pre_render(&mut self, context: &mut WebGlContext);

    /// A method invoked after rendering.
    /// Renderer does not invoke this method if this material is not ready.
    fn post_render(&mut self, context: &mut WebGlContext);

    /// Returns `true` if  this renderable material is ready to render.
    /// [`WebGlMaterial::post_render`] never
    fn ready(&self) -> bool;
}
