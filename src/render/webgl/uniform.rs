use gl_matrix4rust::{mat4::AsMat4, vec3::AsVec3};
use log::warn;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGlUniformLocation};

use crate::{entity::Entity, geometry::Geometry, material::Material, render::pp::State};

use super::{
    buffer::BufferDescriptor,
    conversion::{GLintptr, GLsizeiptr, ToGlEnum},
    program::ProgramItem,
    shader::VariableDataType,
    texture::{TextureDescriptor, TextureParameter, TextureUnit},
};

/// Uniform Buffer Object mount point for `atoy_UniversalUniformsVert` and `atoy_UniversalUniformsFrag`.
pub const UBO_UNIVERSAL_UNIFORMS_BINDING: u32 = 0;
/// Uniform Buffer Object mount point for `atoy_Lights`.
pub const UBO_LIGHTS_BINDING: u32 = 1;
/// Uniform Buffer Object mount point for gaussian blur.
pub const UBO_GAUSSIAN_BLUR_BINDING: u32 = 2;

/// Uniform Buffer Object bytes length for `atoy_UniversalUniformsVert` and `atoy_UniversalUniformsFrag`.
pub const UBO_UNIVERSAL_UNIFORMS_BYTES_LENGTH: u32 = 16 + 16 + 64 + 64 + 64;
/// Uniform Buffer Object bytes length for `u_RenderTime`.
pub const UBO_UNIVERSAL_UNIFORMS_RENDER_TIME_BYTES_LENGTH: u32 = 4;
/// Uniform Buffer Object bytes length for `u_EnableLighting`.
pub const UBO_UNIVERSAL_UNIFORMS_ENABLE_LIGHTING_BYTES_LENGTH: u32 = 4;
/// Uniform Buffer Object bytes length for `u_GammaCorrection`.
pub const UBO_UNIVERSAL_UNIFORMS_GAMMA_CORRECTION_BYTES_LENGTH: u32 = 4;
/// Uniform Buffer Object bytes length for `u_GammaCorrectionInverse`.
pub const UBO_UNIVERSAL_UNIFORMS_GAMMA_CORRECTION_INVERSE_BYTES_LENGTH: u32 = 4;
/// Uniform Buffer Object bytes length for `u_CameraPosition`.
pub const UBO_UNIVERSAL_UNIFORMS_CAMERA_POSITION_BYTES_LENGTH: u32 = 12;
/// Uniform Buffer Object bytes length for `u_ViewMatrix`.
pub const UBO_UNIVERSAL_UNIFORMS_VIEW_MATRIX_BYTES_LENGTH: u32 = 64;
/// Uniform Buffer Object bytes length for `u_ProjMatrix`.
pub const UBO_UNIVERSAL_UNIFORMS_PROJ_MATRIX_BYTES_LENGTH: u32 = 64;
/// Uniform Buffer Object bytes length for `u_ViewProjMatrix`.
pub const UBO_UNIVERSAL_UNIFORMS_VIEW_PROJ_MATRIX_BYTES_LENGTH: u32 = 64;

/// Uniform Buffer Object bytes offset for `u_RenderTime`.
pub const UBO_UNIVERSAL_UNIFORMS_RENDER_TIME_BYTES_OFFSET: u32 = 0;
/// Uniform Buffer Object bytes offset for `u_EnableLighting`.
pub const UBO_UNIVERSAL_UNIFORMS_ENABLE_LIGHTING_BYTES_OFFSET: u32 = 4;
/// Uniform Buffer Object bytes offset for `u_GammaCorrection`.
pub const UBO_UNIVERSAL_UNIFORMS_GAMMA_CORRECTION_BYTES_OFFSET: u32 = 8;
/// Uniform Buffer Object bytes offset for `u_GammaCorrectionInverse`.
pub const UBO_UNIVERSAL_UNIFORMS_GAMMA_CORRECTION_INVERSE_BYTES_OFFSET: u32 = 12;
/// Uniform Buffer Object bytes offset for `u_CameraPosition`.
pub const UBO_UNIVERSAL_UNIFORMS_CAMERA_POSITION_BYTES_OFFSET: u32 = 16;
/// Uniform Buffer Object bytes offset for `u_ViewMatrix`.
pub const UBO_UNIVERSAL_UNIFORMS_VIEW_MATRIX_BYTES_OFFSET: u32 = 32;
/// Uniform Buffer Object bytes offset for `u_ProjMatrix`.
pub const UBO_UNIVERSAL_UNIFORMS_PROJ_MATRIX_BYTES_OFFSET: u32 = 96;
/// Uniform Buffer Object bytes offset for `u_ViewProjMatrix`.
pub const UBO_UNIVERSAL_UNIFORMS_VIEW_PROJ_MATRIX_BYTES_OFFSET: u32 = 160;

/// Uniform Buffer Object bytes length for `atoy_Lights`.
pub const UBO_LIGHTS_BYTES_LENGTH: u32 = 16 + 16 + 64 * 12 + 64 * 12 + 80 * 12 + 112 * 12;
/// Uniform Buffer Object bytes length for `u_Attenuations`.
pub const UBO_LIGHTS_ATTENUATIONS_BYTES_LENGTH: u32 = 12;
/// Uniform Buffer Object bytes length for `u_AmbientLight`.
pub const UBO_LIGHTS_AMBIENT_LIGHT_BYTES_LENGTH: u32 = 16;
/// Uniform Buffer Object bytes length for `u_DirectionalLights`.
pub const UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTES_LENGTH: u32 = 64;
/// Uniform Buffer Object bytes length for `u_PointLights`.
pub const UBO_LIGHTS_POINT_LIGHTS_BYTES_LENGTH: u32 = 64;
/// Uniform Buffer Object bytes length for `u_SpotLights`.
pub const UBO_LIGHTS_SPOT_LIGHTS_BYTES_LENGTH: u32 = 80;
/// Uniform Buffer Object bytes length for `u_AreaLights`.
pub const UBO_LIGHTS_AREA_LIGHTS_BYTES_LENGTH: u32 = 112;

/// Uniform Buffer Object bytes offset for `u_Attenuations`.
pub const UBO_LIGHTS_ATTENUATIONS_BYTES_OFFSET: u32 = 0;
/// Uniform Buffer Object bytes offset for `u_AmbientLight`.
pub const UBO_LIGHTS_AMBIENT_LIGHT_BYTES_OFFSET: u32 = 16;
/// Uniform Buffer Object bytes offset for `u_DirectionalLights`.
pub const UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTES_OFFSET: u32 = 32;
/// Uniform Buffer Object bytes offset for `u_PointLights`.
pub const UBO_LIGHTS_POINT_LIGHTS_BYTES_OFFSET: u32 = 800;
/// Uniform Buffer Object bytes offset for `u_SpotLights`.
pub const UBO_LIGHTS_SPOT_LIGHTS_BYTES_OFFSET: u32 = 1568;
/// Uniform Buffer Object bytes offset for `u_AreaLights`.
pub const UBO_LIGHTS_AREA_LIGHTS_BYTES_OFFSET: u32 = 2528;

/// Available uniform values.
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
        unit: TextureUnit,
    },
}

/// Available uniform block values.
#[derive(Clone)]
pub enum UniformBlockValue {
    BufferBase {
        descriptor: BufferDescriptor,
        binding: u32,
    },
    BufferRange {
        descriptor: BufferDescriptor,
        offset: GLintptr,
        size: GLsizeiptr,
        binding: u32,
    },
}

/// Uniform binding sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UniformBinding {
    RenderTime,
    CanvasSize,
    DrawingBufferSize,
    ModelMatrix,
    NormalMatrix,
    ViewMatrix,
    ProjMatrix,
    ViewProjMatrix,
    CameraPosition,
    Transparency,
    FromGeometry(&'static str),
    FromMaterial(&'static str),
    FromEntity(&'static str),
}

impl UniformBinding {
    /// Returns variable name.
    pub fn variable_name(&self) -> &str {
        match self {
            UniformBinding::RenderTime => "u_RenderTime",
            UniformBinding::CanvasSize => "u_CanvasSize",
            UniformBinding::DrawingBufferSize => "u_DrawingBufferSize",
            UniformBinding::ModelMatrix => "u_ModelMatrix",
            UniformBinding::NormalMatrix => "u_NormalMatrix",
            UniformBinding::ViewMatrix => "u_ViewMatrix",
            UniformBinding::ProjMatrix => "u_ProjMatrix",
            UniformBinding::ViewProjMatrix => "u_ViewProjMatrix",
            UniformBinding::CameraPosition => "u_CameraPosition",
            UniformBinding::Transparency => "u_Transparency",
            UniformBinding::FromGeometry(name)
            | UniformBinding::FromMaterial(name)
            | UniformBinding::FromEntity(name) => name,
        }
    }

    /// Returns [`VariableDataType`] for non-custom bindings.
    pub fn data_type(&self) -> Option<VariableDataType> {
        match self {
            UniformBinding::RenderTime => Some(VariableDataType::Float),
            UniformBinding::CanvasSize => Some(VariableDataType::FloatVec2),
            UniformBinding::DrawingBufferSize => Some(VariableDataType::FloatVec2),
            UniformBinding::ModelMatrix => Some(VariableDataType::Mat4),
            UniformBinding::NormalMatrix => Some(VariableDataType::Mat4),
            UniformBinding::ViewMatrix => Some(VariableDataType::Mat4),
            UniformBinding::ProjMatrix => Some(VariableDataType::Mat4),
            UniformBinding::ViewProjMatrix => Some(VariableDataType::Mat4),
            UniformBinding::CameraPosition => Some(VariableDataType::FloatVec3),
            UniformBinding::Transparency => Some(VariableDataType::Float),
            UniformBinding::FromGeometry(_)
            | UniformBinding::FromMaterial(_)
            | UniformBinding::FromEntity(_) => None,
        }
    }
}

/// Uniform block binding sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UniformBlockBinding {
    StandardUniversalUniforms,
    StandardLights,
    FromGeometry(&'static str),
    FromMaterial(&'static str),
    FromEntity(&'static str),
}

impl UniformBlockBinding {
    /// Returns uniform block interface name.
    pub fn block_name(&self) -> &str {
        match self {
            UniformBlockBinding::StandardUniversalUniforms => "atoy_UniversalUniforms",
            UniformBlockBinding::StandardLights => "atoy_Lights",
            UniformBlockBinding::FromGeometry(name)
            | UniformBlockBinding::FromMaterial(name)
            | UniformBlockBinding::FromEntity(name) => name,
        }
    }
}

/// Structural uniform binding sources.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UniformStructuralBinding {
    FromGeometry {
        variable_name: &'static str,
        fields: Vec<&'static str>,
        array_len: Option<usize>,
    },
    FromMaterial {
        variable_name: &'static str,
        fields: Vec<&'static str>,
        array_len: Option<usize>,
    },
    FromEntity {
        variable_name: &'static str,
        fields: Vec<&'static str>,
        array_len: Option<usize>,
    },
}

impl UniformStructuralBinding {
    /// Returns variable name.
    pub fn variable_name(&self) -> &str {
        match self {
            UniformStructuralBinding::FromGeometry { variable_name, .. }
            | UniformStructuralBinding::FromMaterial { variable_name, .. }
            | UniformStructuralBinding::FromEntity { variable_name, .. } => variable_name,
        }
    }

    /// Returns fields.
    pub fn fields(&self) -> &[&str] {
        match self {
            UniformStructuralBinding::FromGeometry { fields, .. }
            | UniformStructuralBinding::FromMaterial { fields, .. }
            | UniformStructuralBinding::FromEntity { fields, .. } => &fields,
        }
    }

    /// Returns array len. Returns `None` if not a array.
    pub fn array_len(&self) -> Option<usize> {
        match self {
            UniformStructuralBinding::FromGeometry { array_len, .. }
            | UniformStructuralBinding::FromMaterial { array_len, .. }
            | UniformStructuralBinding::FromEntity { array_len, .. } => array_len.clone(),
        }
    }

    /// Returns [`VariableDataType`] for non-custom bindings.
    pub fn data_type(&self) -> Option<VariableDataType> {
        match self {
            UniformStructuralBinding::FromGeometry { .. }
            | UniformStructuralBinding::FromMaterial { .. }
            | UniformStructuralBinding::FromEntity { .. } => None,
        }
    }
}

/// Uniforms buffer bound to WebGL runtime.
pub struct BoundUniform {
    descriptor: BufferDescriptor,
}

/// Binds uniform data from a entity.
pub fn bind_uniforms(
    state: &mut State,
    entity: &Entity,
    geometry: &dyn Geometry,
    material: &dyn Material,
    program_item: &ProgramItem,
) -> Vec<BoundUniform> {
    // binds simple uniforms
    for (binding, location) in program_item.uniform_locations() {
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
                    UniformBinding::ViewMatrix => state.camera().view_matrix().to_gl(),
                    UniformBinding::ProjMatrix => state.camera().proj_matrix().to_gl(),
                    UniformBinding::ViewProjMatrix => state.camera().view_proj_matrix().to_gl(),
                    _ => unreachable!(),
                };

                Some(UniformValue::Matrix4 {
                    data: mat,
                    transpose: false,
                })
            }
            UniformBinding::CameraPosition => Some(UniformValue::FloatVector3(
                state.camera().position().to_gl(),
            )),
            UniformBinding::RenderTime => Some(UniformValue::Float1(state.timestamp() as f32)),
            UniformBinding::Transparency => {
                Some(UniformValue::Float1(material.transparency().alpha()))
            }
            UniformBinding::CanvasSize => state
                .gl()
                .canvas()
                .and_then(|canvas| canvas.dyn_into::<HtmlCanvasElement>().ok())
                .map(|canvas| {
                    UniformValue::UnsignedIntegerVector2([canvas.width(), canvas.height()])
                }),
            UniformBinding::DrawingBufferSize => Some(UniformValue::IntegerVector2([
                state.gl().drawing_buffer_width(),
                state.gl().drawing_buffer_width(),
            ])),
        };
        let Some(value) = value else {
            warn!(
                target: "BindUniforms",
                "no value specified for uniform {}",
                binding.variable_name()
            );
            continue;
        };

        bind_uniform_value(state, location, value);
    }

    // binds structural uniform, converts it to simple uniform bindings
    for (binding, fields) in program_item.uniform_structural_locations() {
        let mut values = Vec::with_capacity(fields.len());
        match binding {
            UniformStructuralBinding::FromGeometry { .. } => {
                for (field, location) in fields {
                    let value = geometry.uniform_value(field, entity);
                    if let Some(value) = value {
                        values.push((location, value));
                    }
                }
            }
            UniformStructuralBinding::FromMaterial { .. } => {
                for (field, location) in fields {
                    let value = material.uniform_value(field, entity);
                    if let Some(value) = value {
                        values.push((location, value));
                    }
                }
            }
            UniformStructuralBinding::FromEntity { .. } => {
                for (field, location) in fields {
                    let value = entity.uniform_values().get(field).cloned();
                    if let Some(value) = value {
                        values.push((location, value));
                    }
                }
            }
        };

        for (location, value) in values {
            bind_uniform_value(state, location, value);
        }
    }

    // binds uniform blocks
    let mut bounds = Vec::with_capacity(program_item.uniform_block_indices().len());
    for (binding, uniform_block_index) in program_item.uniform_block_indices() {
        let value = match binding {
            UniformBlockBinding::StandardUniversalUniforms => Some(UniformBlockValue::BufferBase {
                descriptor: state.universal_ubo(),
                binding: UBO_UNIVERSAL_UNIFORMS_BINDING,
            }),
            UniformBlockBinding::StandardLights => Some(UniformBlockValue::BufferBase {
                descriptor: state.lights_ubo(),
                binding: UBO_LIGHTS_BINDING,
            }),
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
                binding,
            } => {
                if let Err(err) = state
                    .buffer_store_mut()
                    .bind_uniform_buffer_object(&descriptor, binding)
                {
                    warn!(
                        target: "BindUniforms",
                        "bind uniform buffer object failed: {}",
                        err
                    );
                    continue;
                };

                state.gl().uniform_block_binding(
                    program_item.gl_program(),
                    *uniform_block_index,
                    binding,
                );

                bounds.push(BoundUniform { descriptor });
            }
            UniformBlockValue::BufferRange {
                descriptor,
                offset,
                size,
                binding,
            } => {
                if let Err(err) = state.buffer_store_mut().bind_uniform_buffer_object_range(
                    &descriptor,
                    offset,
                    size,
                    binding,
                ) {
                    warn!(
                        target: "BindUniforms",
                        "bind uniform buffer object failed: {}",
                        err
                    );
                    continue;
                };

                state.gl().uniform_block_binding(
                    program_item.gl_program(),
                    *uniform_block_index,
                    binding,
                );

                bounds.push(BoundUniform { descriptor });
            }
        }
    }

    bounds
}

/// Binds a [`UniformValue`] to a [`WebGlUniformLocation`]
pub fn bind_uniform_value(state: &mut State, location: &WebGlUniformLocation, value: UniformValue) {
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
        UniformValue::UnsignedInteger3(x, y, z) => state.gl().uniform3ui(Some(location), x, y, z),
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
        UniformValue::Matrix2 { data, transpose } => {
            state
                .gl()
                .uniform_matrix2fv_with_f32_array(Some(location), transpose, &data)
        }
        UniformValue::Matrix3 { data, transpose } => {
            state
                .gl()
                .uniform_matrix3fv_with_f32_array(Some(location), transpose, &data)
        }
        UniformValue::Matrix4 { data, transpose } => {
            state
                .gl()
                .uniform_matrix4fv_with_f32_array(Some(location), transpose, &data)
        }
        UniformValue::Texture {
            descriptor,
            params,
            unit,
        } => {
            // active texture
            state.gl().active_texture(unit.gl_enum());

            let (target, texture) = match state.texture_store_mut().use_texture(&descriptor) {
                Ok(texture) => texture,
                Err(err) => {
                    warn!(
                        target: "BindUniforms",
                        "use texture store error: {}",
                        err
                    );
                    return;
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
            state.gl().uniform1i(Some(location), unit.unit_index());
        }
    };
}

/// Unbinds all uniforms after draw calls.
///
/// If you bind buffer attributes ever,
/// remember to unbind them by yourself or use this function.
pub fn unbind_uniforms(state: &mut State, bounds: Vec<BoundUniform>) {
    for BoundUniform { descriptor } in bounds {
        state.buffer_store_mut().unuse_buffer(&descriptor);
    }
}
