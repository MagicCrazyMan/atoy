use gl_matrix4rust::{mat4::AsMat4, vec3::AsVec3, vec4::AsVec4};
use log::warn;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGlUniformLocation};

use crate::{
    entity::BorrowedMut,
    geometry::Geometry,
    material::Material,
    render::pp::{State, Stuff},
};

use super::{
    buffer::{BufferDescriptor, BufferItem, BufferTarget},
    conversion::{GLintptr, GLsizeiptr, ToGlEnum},
    program::ProgramItem,
    shader::VariableDataType,
    texture::{TextureDescriptor, TextureParameter, TextureUnit},
};

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
        texture_unit: TextureUnit,
    },
}

/// Available uniform block values.
#[derive(Clone)]
pub enum UniformBlockValue {
    BufferBase {
        descriptor: BufferDescriptor,
        target: BufferTarget,
        uniform_block_binding: u32,
    },
    BufferRange {
        descriptor: BufferDescriptor,
        target: BufferTarget,
        offset: GLintptr,
        size: GLsizeiptr,
        uniform_block_binding: u32,
    },
}

/// Uniform binding sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UniformBinding {
    CanvasSize,
    DrawingBufferSize,
    ModelMatrix,
    NormalMatrix,
    ViewMatrix,
    ProjMatrix,
    ViewProjMatrix,
    ActiveCameraPosition,
    EnableLighting,
    AmbientReflection,
    DiffuseReflection,
    FromGeometry(&'static str),
    FromMaterial(&'static str),
    FromEntity(&'static str),
}

impl UniformBinding {
    /// Returns variable name.
    pub fn variable_name(&self) -> &str {
        match self {
            UniformBinding::CanvasSize => "u_CanvasSize",
            UniformBinding::DrawingBufferSize => "u_DrawingBufferSize",
            UniformBinding::ModelMatrix => "u_ModelMatrix",
            UniformBinding::NormalMatrix => "u_NormalMatrix",
            UniformBinding::ViewMatrix => "u_ViewMatrix",
            UniformBinding::ProjMatrix => "u_ProjMatrix",
            UniformBinding::ViewProjMatrix => "u_ViewProjMatrix",
            UniformBinding::ActiveCameraPosition => "u_ActiveCameraPosition",
            UniformBinding::EnableLighting => "u_EnableLighting",
            UniformBinding::AmbientReflection => "u_AmbientReflection",
            UniformBinding::DiffuseReflection => "u_DiffuseReflection",
            UniformBinding::FromGeometry(name)
            | UniformBinding::FromMaterial(name)
            | UniformBinding::FromEntity(name) => name,
        }
    }

    /// Returns [`VariableDataType`] for non-custom bindings.
    pub fn data_type(&self) -> Option<VariableDataType> {
        match self {
            UniformBinding::CanvasSize => Some(VariableDataType::FloatVec2),
            UniformBinding::DrawingBufferSize => Some(VariableDataType::FloatVec2),
            UniformBinding::ModelMatrix => Some(VariableDataType::Mat4),
            UniformBinding::NormalMatrix => Some(VariableDataType::Mat4),
            UniformBinding::ViewMatrix => Some(VariableDataType::Mat4),
            UniformBinding::ProjMatrix => Some(VariableDataType::Mat4),
            UniformBinding::ViewProjMatrix => Some(VariableDataType::Mat4),
            UniformBinding::ActiveCameraPosition => Some(VariableDataType::FloatVec3),
            UniformBinding::EnableLighting => Some(VariableDataType::Bool),
            UniformBinding::AmbientReflection => Some(VariableDataType::FloatVec4),
            UniformBinding::DiffuseReflection => Some(VariableDataType::FloatVec4),
            UniformBinding::FromGeometry(_)
            | UniformBinding::FromMaterial(_)
            | UniformBinding::FromEntity(_) => None,
        }
    }
}

/// Uniform block binding sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UniformBlockBinding {
    DiffuseLights,
    FromGeometry(&'static str),
    FromMaterial(&'static str),
    FromEntity(&'static str),
}

impl UniformBlockBinding {
    /// Returns variable name.
    pub fn variable_name(&self) -> &str {
        match self {
            UniformBlockBinding::DiffuseLights => "DiffuseLights",
            UniformBlockBinding::FromGeometry(name)
            | UniformBlockBinding::FromMaterial(name)
            | UniformBlockBinding::FromEntity(name) => name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UniformStructuralBinding {
    AmbientLight,
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
            UniformStructuralBinding::AmbientLight => "u_AmbientLight",
            UniformStructuralBinding::FromGeometry { variable_name, .. }
            | UniformStructuralBinding::FromMaterial { variable_name, .. }
            | UniformStructuralBinding::FromEntity { variable_name, .. } => variable_name,
        }
    }

    /// Returns fields.
    pub fn fields(&self) -> &[&str] {
        match self {
            UniformStructuralBinding::AmbientLight => &["enabled", "color"],
            UniformStructuralBinding::FromGeometry { fields, .. }
            | UniformStructuralBinding::FromMaterial { fields, .. }
            | UniformStructuralBinding::FromEntity { fields, .. } => &fields,
        }
    }

    /// Returns array len. Returns `None` if not a array.
    pub fn array_len(&self) -> Option<usize> {
        match self {
            UniformStructuralBinding::AmbientLight => None,
            UniformStructuralBinding::FromGeometry { array_len, .. }
            | UniformStructuralBinding::FromMaterial { array_len, .. }
            | UniformStructuralBinding::FromEntity { array_len, .. } => array_len.clone(),
        }
    }

    /// Returns [`VariableDataType`] for non-custom bindings.
    pub fn data_type(&self) -> Option<VariableDataType> {
        match self {
            UniformStructuralBinding::AmbientLight => {
                Some(VariableDataType::Struct("AmbientLight"))
            }
            UniformStructuralBinding::FromGeometry { .. }
            | UniformStructuralBinding::FromMaterial { .. }
            | UniformStructuralBinding::FromEntity { .. } => None,
        }
    }
}

/// Binds uniform data from a entity.
pub fn bind_uniforms(
    state: &mut State,
    stuff: &dyn Stuff,
    entity: &BorrowedMut,
    geometry: &dyn Geometry,
    material: &dyn Material,
    program_item: &ProgramItem,
) -> Vec<BufferItem> {
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
            UniformBinding::ActiveCameraPosition => Some(UniformValue::FloatVector3(
                stuff.camera().position().to_gl(),
            )),
            UniformBinding::EnableLighting => Some(UniformValue::Bool(stuff.enable_lighting())),
            UniformBinding::AmbientReflection => material
                .ambient()
                .map(|c| UniformValue::FloatVector4(c.to_gl())),
            UniformBinding::DiffuseReflection => material
                .diffuse()
                .map(|c| UniformValue::FloatVector4(c.to_gl())),
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
            UniformStructuralBinding::AmbientLight => {
                for (field, location) in fields {
                    match field.as_str() {
                        "u_AmbientLight.enabled" => values.push((
                            location,
                            UniformValue::Bool(stuff.ambient_light().is_some()),
                        )),
                        "u_AmbientLight.color" => {
                            if let Some(light) = stuff.ambient_light() {
                                values.push((
                                    location,
                                    UniformValue::FloatVector3(light.color().to_gl()),
                                ))
                            }
                        }
                        _ => {}
                    }
                }
            }
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
    let mut buffer_items = Vec::with_capacity(program_item.uniform_block_indices().len());
    for (binding, uniform_block_index) in program_item.uniform_block_indices() {
        let value = match binding {
            UniformBlockBinding::DiffuseLights => Some(UniformBlockValue::BufferBase {
                descriptor: stuff.diffuse_lights_descriptor(),
                target: BufferTarget::UniformBuffer,
                uniform_block_binding: 1,
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
                target,
                uniform_block_binding,
            } => {
                let buffer_item = match state.buffer_store_mut().use_buffer(descriptor, target) {
                    Ok(buffer) => buffer,
                    Err(err) => {
                        warn!(
                            target: "BindUniforms",
                            "use buffer store error: {}",
                            err
                        );
                        continue;
                    }
                };

                state
                    .gl()
                    .bind_buffer(target.gl_enum(), Some(&buffer_item.gl_buffer()));

                state.gl().uniform_block_binding(
                    program_item.gl_program(),
                    *uniform_block_index,
                    uniform_block_binding,
                );
                state.gl().bind_buffer_base(
                    target.gl_enum(),
                    uniform_block_binding,
                    Some(&buffer_item.gl_buffer()),
                );

                state.gl().bind_buffer(target.gl_enum(), None);

                buffer_items.push(buffer_item);
            }
            UniformBlockValue::BufferRange {
                descriptor,
                target,
                offset,
                size,
                uniform_block_binding,
            } => {
                let buffer_item = match state.buffer_store_mut().use_buffer(descriptor, target) {
                    Ok(buffer) => buffer,
                    Err(err) => {
                        warn!(
                            target: "BindUniforms",
                            "use buffer store error: {}",
                            err
                        );
                        continue;
                    }
                };

                state
                    .gl()
                    .bind_buffer(target.gl_enum(), Some(&buffer_item.gl_buffer()));

                state.gl().uniform_block_binding(
                    program_item.gl_program(),
                    *uniform_block_index,
                    uniform_block_binding,
                );
                state.gl().bind_buffer_range_with_i32_and_i32(
                    target.gl_enum(),
                    *uniform_block_index,
                    Some(&buffer_item.gl_buffer()),
                    offset,
                    size,
                );

                state.gl().bind_buffer(target.gl_enum(), None);

                buffer_items.push(buffer_item);
            }
        }
    }

    buffer_items
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
            texture_unit,
        } => {
            // active texture
            state.gl().active_texture(texture_unit.gl_enum());

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
            state
                .gl()
                .uniform1i(Some(location), texture_unit.unit_index());
        }
    };
}
