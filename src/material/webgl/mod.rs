pub mod solid_color;
pub mod texture;

use std::{
    any::Any,
    borrow::Cow,
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::{
    entity::Entity,
    readonly::Readonly,
    renderer::webgl::{
        attribute::{AttributeBinding, AttributeValue},
        program::Define,
        state::FrameState,
        uniform::{UniformBinding, UniformBlockBinding, UniformBlockValue, UniformValue},
    },
};

use super::Transparency;

/// Standard material preparation procedure finishing callback.
/// Developer should invoke [`StandardMaterialPreparationCallback::finish`]
/// after [`StandardMaterial::prepare`] finished.
#[derive(Debug, Clone)]
pub struct StandardMaterialPreparationCallback(Weak<RefCell<*mut Entity>>);

impl StandardMaterialPreparationCallback {
    /// Marks preparation procedure is finished.
    pub fn finish(&self) {
        unsafe {
            let Some(entity) = self.0.upgrade() else {
                return;
            };

            let mut entity = entity.borrow_mut();
            if (*entity).is_null() {
                return;
            }

            (**entity).mark_material_dirty();
        }
    }
}

impl Entity {
    pub(crate) fn material_callback(&self) -> StandardMaterialPreparationCallback {
        StandardMaterialPreparationCallback(Rc::downgrade(self.me()))
    }
}

pub trait StandardMaterial {
    /// Returns a material name.
    fn name(&self) -> Cow<'_, str>;

    /// Returns `true` if material is ready for drawing.
    /// Drawer skips entity drawing if material is not ready.
    fn ready(&self) -> bool;

    /// Prepares material.
    fn prepare(&mut self, state: &mut FrameState, callback: StandardMaterialPreparationCallback);

    /// Returns transparency of this material.
    fn transparency(&self) -> Transparency;

    /// Returns a custom attribute value by an attribute variable name.
    fn attribute_value(&self, name: &str) -> Option<Readonly<'_, AttributeValue>>;

    /// Returns a custom uniform value by an uniform variable name.
    fn uniform_value(&self, name: &str) -> Option<Readonly<'_, UniformValue>>;

    /// Returns a custom uniform block buffer binding value by an uniform block name.
    fn uniform_block_value(&self, name: &str) -> Option<Readonly<'_, UniformBlockValue>>;

    // /// Returns GLSL code snippet with processing function for vertex shader.
    // fn vertex_process(&self) -> Cow<'_, str> {
    //     Cow::Borrowed(include_str!("./shaders/vertex_process.glsl"))
    // }

    /// Returns GLSL code snippet with processing function for fragment shader.
    fn fragment_process(&self) -> Cow<'_, str>;

    /// Returns custom defines macros for vertex shader.
    fn vertex_defines(&self) -> &[Define<'_>];

    /// Returns custom defines macros for fragment shader.
    ///
    /// Returns [`StandardMaterial::vertex_defines`] as default.
    fn fragment_defines(&self) -> &[Define<'_>] {
        self.vertex_defines()
    }

    /// Returns custom self-associated GLSL code snippet by name.
    fn snippet(&self, name: &str) -> Option<Cow<'_, str>>;

    /// Returns attribute bindings requirements.
    fn attribute_bindings(&self) -> &[AttributeBinding];

    /// Returns uniform bindings requirements.
    fn uniform_bindings(&self) -> &[UniformBinding];

    /// Returns uniform block bindings requirements.
    fn uniform_block_bindings(&self) -> &[UniformBlockBinding];

    /// Returns `true` if vertex shader should output position on Eye Space.
    /// `vec3 v_PositionES` is available in fragment shader when enabled.
    fn use_position_eye_space(&self) -> bool;

    /// Returns `true` if vertex shader should output normal.
    /// `vec3 v_Normal` is available in fragment shader when enabled.
    ///
    /// [`Geometry::normals`](crate::geometry::Geometry::normals) is required.
    fn use_normal(&self) -> bool;

    /// Returns `true` if vertex shader should output texture coordinate.
    /// `vec2 v_TexCoord` is available in fragment shader when enabled.
    ///
    /// [`Geometry::texture_coordinates`](crate::geometry::Geometry::texture_coordinates) is required.
    fn use_texture_coordinate(&self) -> bool;

    /// Returns `true` if vertex shader should output TBN matrix.
    /// `mat3 v_TBN` is available in fragment shader when enabled.
    ///
    /// Useless unless [`StandardMaterial::use_normal`] is `true`;
    fn use_tbn(&self) -> bool;

    /// Returns `true` if vertex shader should output inverted TBN matrix.
    /// `mat3 v_TBNInvert` is available in fragment shader when enabled.
    ///
    /// Useless unless [`StandardMaterial::use_tbn`] is `true`;
    ///
    /// Returns `true` as default when [`StandardMaterial::use_tbn`] is `true`,
    /// overrides this method if you really don't want to calculate this or have some idea else.
    fn use_tbn_invert(&self) -> bool {
        self.use_tbn()
    }

    /// Returns `true` if vertex shader should calculate bitangets from tangents and normals,
    /// instead using bitangents from [`Geometry::bitangents`](crate::geometry::Geometry::bitangents).
    ///
    /// Useless unless [`StandardMaterial::use_tbn`] is `true`;
    fn use_calculated_bitangent(&self) -> bool;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}
