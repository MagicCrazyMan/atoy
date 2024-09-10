use super::{
    attribute::WebGlAttributeValue,
    context::WebGlContext,
    material::WebGlMaterial,
    uniform::{WebGlUniformBlockValue, WebGlUniformValue},
};

pub trait WebGlRenderable {
    /// Returns material for rendering.
    fn material(&self) -> &dyn WebGlMaterial;

    /// Returns attribute value of a specified GLSL attribute variable name.
    fn attribute_value(&self, name: &str) -> Option<WebGlAttributeValue>;

    /// Returns uniform value of a specified GLSL uniform variable name.
    fn uniform_value(&self, name: &str) -> Option<WebGlUniformValue>;

    /// Returns uniform block value of a specified GLSL uniform block layout variable name.
    fn uniform_block_value(&self, name: &str) -> Option<WebGlUniformBlockValue>;

    /// A method invoked before rendering.
    /// Renderer always invokes this method no matter this object is ready or not.
    /// Developer should prepare the object (fetching data from remote, loading resources etc.) in this method.
    fn pre_render(&mut self, context: &mut WebGlContext);

    /// A method invoked after rendering.
    /// Renderer does not invoke this method if this object is not ready.
    fn post_render(&mut self, context: &mut WebGlContext);

    /// Returns `true` if  this renderable object is ready to render.
    /// [`WebGlRenderable::post_render`] never
    fn ready(&self) -> bool;
}
