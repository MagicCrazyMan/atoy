use gl_matrix4rust::{mat4::AsMat4, vec3::AsVec3};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::console_log;
use web_sys::HtmlCanvasElement;

use crate::{
    entity::BorrowedMut,
    geometry::Geometry,
    material::Material,
    render::pp::{State, Stuff},
};

use super::{
    buffer::{BufferDescriptor, BufferTarget},
    conversion::{GLintptr, GLsizeiptr, ToGlEnum},
    program::ProgramItem,
    texture::{TextureDescriptor, TextureParameter, TextureUnit},
};

#[derive(Clone)]
pub enum UniformValue {
    Bool(bool),
    Float1(f32),
    Float2(f32, f32),
    Float3(f32, f32, f32),
    Float4(f32, f32, f32, f32),
    UnsignedInteger1(u32),
    UnsignedInteger2(u32, u32),
    UnsignedInteger3(u32, u32, u32),
    UnsignedInteger4(u32, u32, u32, u32),
    Integer1(i32),
    Integer2(i32, i32),
    Integer3(i32, i32, i32),
    Integer4(i32, i32, i32, i32),
    FloatVector1([f32; 1]),
    FloatVector2([f32; 2]),
    FloatVector3([f32; 3]),
    FloatVector4([f32; 4]),
    IntegerVector1([i32; 1]),
    IntegerVector2([i32; 2]),
    IntegerVector3([i32; 3]),
    IntegerVector4([i32; 4]),
    UnsignedIntegerVector1([u32; 1]),
    UnsignedIntegerVector2([u32; 2]),
    UnsignedIntegerVector3([u32; 3]),
    UnsignedIntegerVector4([u32; 4]),
    Matrix2 {
        data: [f32; 4],
        transpose: bool,
    },
    Matrix3 {
        data: [f32; 9],
        transpose: bool,
    },
    Matrix4 {
        data: [f32; 16],
        transpose: bool,
    },
    Texture {
        descriptor: TextureDescriptor,
        params: Vec<TextureParameter>,
        texture_unit: TextureUnit,
    },
}

#[derive(Clone)]
pub enum UniformBlockValue {
    BufferBase {
        descriptor: BufferDescriptor,
        target: BufferTarget,
        binding: u32,
    },
    BufferRange {
        descriptor: BufferDescriptor,
        target: BufferTarget,
        offset: GLintptr,
        size: GLsizeiptr,
        binding: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UniformBinding {
    CanvasSize,
    ModelMatrix,
    NormalMatrix,
    ViewMatrix,
    ProjMatrix,
    ViewProjMatrix,
    ActiveCameraPosition,
    ActiveCameraCenter,
    FromGeometry(&'static str),
    FromMaterial(&'static str),
    FromEntity(&'static str),
}

impl UniformBinding {
    pub fn variable_name(&self) -> &str {
        match self {
            UniformBinding::CanvasSize => "u_CanvasSize",
            UniformBinding::ModelMatrix => "u_ModelMatrix",
            UniformBinding::NormalMatrix => "u_NormalMatrix",
            UniformBinding::ViewMatrix => "u_ViewMatrix",
            UniformBinding::ProjMatrix => "u_ProjMatrix",
            UniformBinding::ViewProjMatrix => "u_ViewProjMatrix",
            UniformBinding::ActiveCameraPosition => "u_ActiveCameraPosition",
            UniformBinding::ActiveCameraCenter => "u_ActiveCameraDirection",
            UniformBinding::FromGeometry(name)
            | UniformBinding::FromMaterial(name)
            | UniformBinding::FromEntity(name) => name,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UniformBlockBinding {
    FromGeometry(&'static str),
    FromMaterial(&'static str),
    FromEntity(&'static str),
}

impl UniformBlockBinding {
    pub fn variable_name(&self) -> &str {
        match self {
            UniformBlockBinding::FromGeometry(name)
            | UniformBlockBinding::FromMaterial(name)
            | UniformBlockBinding::FromEntity(name) => name,
        }
    }
}

/// Binds uniform data of the entity.
pub(crate) fn bind_uniforms(
    state: &mut State,
    stuff: &dyn Stuff,
    entity: &BorrowedMut,
    geometry: &dyn Geometry,
    material: &dyn Material,
    program: &ProgramItem,
) {
    for (binding, location) in program.uniform_locations() {
        let value = match binding {
            UniformBinding::FromGeometry(name) => (*geometry).uniform_value(name, entity),
            UniformBinding::FromMaterial(name) => (*material).uniform_value(name, entity),
            UniformBinding::FromEntity(name) => entity.uniform_values().get(*name).cloned(),
            UniformBinding::ModelMatrix
            | UniformBinding::ViewMatrix
            | UniformBinding::ProjMatrix
            | UniformBinding::NormalMatrix
            | UniformBinding::ViewProjMatrix => {
                let mat = match binding {
                    UniformBinding::ModelMatrix => entity.model_matrix().to_gl(),
                    UniformBinding::NormalMatrix => entity.normal_matrix().to_gl(),
                    UniformBinding::ViewMatrix => stuff.camera().view_matrix().to_gl(),
                    UniformBinding::ProjMatrix => stuff.camera().proj_matrix().to_gl(),
                    UniformBinding::ViewProjMatrix => stuff.camera().view_proj_matrix().to_gl(),
                    _ => unreachable!(),
                };

                Some(UniformValue::Matrix4 {
                    data: mat,
                    transpose: false,
                })
            }
            UniformBinding::ActiveCameraPosition | UniformBinding::ActiveCameraCenter => {
                let vec = match binding {
                    UniformBinding::ActiveCameraPosition => stuff.camera().position().to_gl(),
                    UniformBinding::ActiveCameraCenter => stuff.camera().center().to_gl(),
                    _ => unreachable!(),
                };

                Some(UniformValue::FloatVector3(vec))
            }
            UniformBinding::CanvasSize => state
                .gl()
                .canvas()
                .and_then(|canvas| canvas.dyn_into::<HtmlCanvasElement>().ok())
                .map(|canvas| {
                    UniformValue::UnsignedIntegerVector2([canvas.width(), canvas.height()])
                }),
        };
        let Some(value) = value else {
            // should log warning
            continue;
        };

        match value {
            UniformValue::Bool(v) => {
                if v {
                    state.gl().uniform1i(Some(location), 1)
                } else {
                    state.gl().uniform1i(Some(location), 0)
                }
            }
            UniformValue::UnsignedInteger1(x) => state.gl().uniform1ui(Some(location), x),
            UniformValue::UnsignedInteger2(x, y) => state.gl().uniform2ui(Some(location), x, y),
            UniformValue::UnsignedInteger3(x, y, z) => {
                state.gl().uniform3ui(Some(location), x, y, z)
            }
            UniformValue::UnsignedInteger4(x, y, z, w) => {
                state.gl().uniform4ui(Some(location), x, y, z, w)
            }
            UniformValue::Float1(x) => state.gl().uniform1f(Some(location), x),
            UniformValue::Float2(x, y) => state.gl().uniform2f(Some(location), x, y),
            UniformValue::Float3(x, y, z) => state.gl().uniform3f(Some(location), x, y, z),
            UniformValue::Float4(x, y, z, w) => state.gl().uniform4f(Some(location), x, y, z, w),
            UniformValue::Integer1(x) => state.gl().uniform1i(Some(location), x),
            UniformValue::Integer2(x, y) => state.gl().uniform2i(Some(location), x, y),
            UniformValue::Integer3(x, y, z) => state.gl().uniform3i(Some(location), x, y, z),
            UniformValue::Integer4(x, y, z, w) => state.gl().uniform4i(Some(location), x, y, z, w),
            UniformValue::FloatVector1(data) => {
                state.gl().uniform1fv_with_f32_array(Some(location), &data)
            }
            UniformValue::FloatVector2(data) => {
                state.gl().uniform2fv_with_f32_array(Some(location), &data)
            }
            UniformValue::FloatVector3(data) => {
                state.gl().uniform3fv_with_f32_array(Some(location), &data)
            }
            UniformValue::FloatVector4(data) => {
                state.gl().uniform4fv_with_f32_array(Some(location), &data)
            }
            UniformValue::IntegerVector1(data) => {
                state.gl().uniform1iv_with_i32_array(Some(location), &data)
            }
            UniformValue::IntegerVector2(data) => {
                state.gl().uniform2iv_with_i32_array(Some(location), &data)
            }
            UniformValue::IntegerVector3(data) => {
                state.gl().uniform3iv_with_i32_array(Some(location), &data)
            }
            UniformValue::IntegerVector4(data) => {
                state.gl().uniform4iv_with_i32_array(Some(location), &data)
            }
            UniformValue::UnsignedIntegerVector1(data) => {
                state.gl().uniform1uiv_with_u32_array(Some(location), &data)
            }
            UniformValue::UnsignedIntegerVector2(data) => {
                state.gl().uniform2uiv_with_u32_array(Some(location), &data)
            }
            UniformValue::UnsignedIntegerVector3(data) => {
                state.gl().uniform3uiv_with_u32_array(Some(location), &data)
            }
            UniformValue::UnsignedIntegerVector4(data) => {
                state.gl().uniform4uiv_with_u32_array(Some(location), &data)
            }
            UniformValue::Matrix2 { data, transpose } => state
                .gl()
                .uniform_matrix2fv_with_f32_array(Some(location), transpose, &data),
            UniformValue::Matrix3 { data, transpose } => state
                .gl()
                .uniform_matrix3fv_with_f32_array(Some(location), transpose, &data),
            UniformValue::Matrix4 { data, transpose } => state
                .gl()
                .uniform_matrix4fv_with_f32_array(Some(location), transpose, &data),
            UniformValue::Texture {
                descriptor,
                params,
                texture_unit,
            } => {
                // active texture
                state.gl().active_texture(texture_unit.gl_enum());

                let (target, texture) = match state.texture_store_mut().use_texture(&descriptor) {
                    Ok(texture) => texture,
                    Err(err) => {
                        // should log warning
                        console_log!("{}", err);
                        continue;
                    }
                };
                let texture = texture.clone();

                // binds texture
                state.gl().bind_texture(target, Some(&texture));
                // setups sampler parameters
                params
                    .iter()
                    .for_each(|param| param.tex_parameteri(state.gl(), target));
                // binds to shader
                state
                    .gl()
                    .uniform1i(Some(location), texture_unit.unit_index());
            }
        };
    }

    for (binding, index) in program.uniform_block_indices() {
        let value = match binding {
            UniformBlockBinding::FromGeometry(name) => geometry.uniform_block_value(name, entity),
            UniformBlockBinding::FromMaterial(name) => material.uniform_block_value(name, entity),
            UniformBlockBinding::FromEntity(name) => {
                entity.uniform_block_values().get(*name).cloned()
            }
        };
        let Some(value) = value else {
            continue;
        };

        match value {
            UniformBlockValue::BufferBase {
                descriptor,
                target,
                binding,
            } => {
                let buffer_item = match state.buffer_store_mut().use_buffer(descriptor, target) {
                    Ok(buffer) => buffer,
                    Err(err) => {
                        // should log error
                        console_log!("{}", err);
                        continue;
                    }
                };

                state
                    .gl()
                    .uniform_block_binding(program.gl_program(), *index, binding);
                state.gl().bind_buffer_base(
                    target.gl_enum(),
                    *index,
                    Some(&buffer_item.gl_buffer()),
                );
            }
            UniformBlockValue::BufferRange {
                descriptor,
                target,
                offset,
                size,
                binding,
            } => {
                let buffer_item = match state.buffer_store_mut().use_buffer(descriptor, target) {
                    Ok(buffer) => buffer,
                    Err(err) => {
                        // should log error
                        console_log!("{}", err);
                        continue;
                    }
                };

                state
                    .gl()
                    .uniform_block_binding(program.gl_program(), *index, binding);
                state.gl().bind_buffer_range_with_i32_and_i32(
                    target.gl_enum(),
                    *index,
                    Some(&buffer_item.gl_buffer()),
                    offset,
                    size,
                );
            }
        }
    }
}
