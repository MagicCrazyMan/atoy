use std::collections::HashMap;

use gl_matrix4rust::{mat4::AsMat4, vec3::AsVec3};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::console_log;
use web_sys::{HtmlCanvasElement, WebGlUniformLocation};

use crate::{
    entity::BorrowedMut,
    geometry::Geometry,
    material::Material,
    render::pp::{State, Stuff},
};

use super::{
    attribute::{AttributeBinding, AttributeValue},
    buffer::{BufferDescriptor, BufferTarget},
    conversion::{GLint, GLintptr, GLsizei, GLuint, ToGlEnum},
    uniform::{UniformBinding, UniformValue},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CullFace {
    Front,
    Back,
    Both,
}

#[derive(Clone)]
pub enum Draw {
    Arrays {
        mode: DrawMode,
        first: GLint,
        count: GLsizei,
    },
    Elements {
        mode: DrawMode,
        count: GLsizei,
        element_type: DrawElementType,
        offset: GLintptr,
        indices: BufferDescriptor,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrawElementType {
    UnsignedByte,
    UnsignedShort,
    UnsignedInt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrawMode {
    Points,
    Lines,
    LineLoop,
    LineStrip,
    Triangles,
    TriangleStrip,
    TriangleFan,
}

/// Binds attributes of the entity.
pub(crate) unsafe fn bind_attributes(
    state: &mut State,
    entity: &BorrowedMut,
    geometry: *const dyn Geometry,
    material: *const dyn Material,
    attribute_locations: &HashMap<AttributeBinding, GLuint>,
) {
    for (binding, location) in attribute_locations {
        let value = match binding {
            AttributeBinding::GeometryPosition => (*geometry).vertices(),
            AttributeBinding::GeometryTextureCoordinate => (*geometry).texture_coordinates(),
            AttributeBinding::GeometryNormal => (*geometry).normals(),
            AttributeBinding::FromGeometry(name) => (*geometry).attribute_value(name, entity),
            AttributeBinding::FromMaterial(name) => (*material).attribute_value(name, entity),
            AttributeBinding::FromEntity(name) => entity.attribute_values().get(*name).cloned(),
        };
        let Some(value) = value else {
            // should log warning
            console_log!("3");
            continue;
        };

        match value {
            AttributeValue::Buffer {
                descriptor,
                target,
                component_size,
                data_type,
                normalized,
                bytes_stride,
                bytes_offset,
            } => {
                let buffer = match state.buffer_store_mut().use_buffer(descriptor, target) {
                    Ok(buffer) => buffer,
                    Err(err) => {
                        // should log error
                        console_log!("{}", err);
                        continue;
                    }
                };

                state.gl().bind_buffer(target.gl_enum(), Some(&buffer));
                state.gl().vertex_attrib_pointer_with_i32(
                    *location,
                    component_size as GLint,
                    data_type.gl_enum(),
                    normalized,
                    bytes_stride,
                    bytes_offset,
                );
                state.gl().enable_vertex_attrib_array(*location);
            }
            AttributeValue::InstancedBuffer {
                descriptor,
                target,
                component_size,
                data_type,
                normalized,
                component_count_per_instance: components_length_per_instance,
                divisor,
            } => {
                let buffer = match state.buffer_store_mut().use_buffer(descriptor, target) {
                    Ok(buffer) => buffer,
                    Err(err) => {
                        // should log error
                        console_log!("{}", err);
                        continue;
                    }
                };

                state.gl().bind_buffer(target.gl_enum(), Some(&buffer));

                let component_size = component_size as GLint;
                // binds each instance
                for i in 0..components_length_per_instance {
                    let offset_location = *location + (i as GLuint);
                    state.gl().vertex_attrib_pointer_with_i32(
                        offset_location,
                        component_size,
                        data_type.gl_enum(),
                        normalized,
                        data_type.bytes_length() * component_size * components_length_per_instance,
                        i * data_type.bytes_length() * component_size,
                    );
                    state.gl().enable_vertex_attrib_array(offset_location);
                    state.gl().vertex_attrib_divisor(offset_location, divisor);
                }
            }
            AttributeValue::Vertex1f(x) => state.gl().vertex_attrib1f(*location, x),
            AttributeValue::Vertex2f(x, y) => state.gl().vertex_attrib2f(*location, x, y),
            AttributeValue::Vertex3f(x, y, z) => state.gl().vertex_attrib3f(*location, x, y, z),
            AttributeValue::Vertex4f(x, y, z, w) => {
                state.gl().vertex_attrib4f(*location, x, y, z, w)
            }
            AttributeValue::Vertex1fv(v) => {
                state.gl().vertex_attrib1fv_with_f32_array(*location, &v)
            }
            AttributeValue::Vertex2fv(v) => {
                state.gl().vertex_attrib2fv_with_f32_array(*location, &v)
            }
            AttributeValue::Vertex3fv(v) => {
                state.gl().vertex_attrib3fv_with_f32_array(*location, &v)
            }
            AttributeValue::Vertex4fv(v) => {
                state.gl().vertex_attrib4fv_with_f32_array(*location, &v)
            }
        };
    }
}

/// Binds uniform data of the entity.
pub(crate) unsafe fn bind_uniforms(
    state: &mut State,
    stuff: &dyn Stuff,
    entity: &BorrowedMut,
    geometry: *const dyn Geometry,
    material: *const dyn Material,
    uniform_locations: &HashMap<UniformBinding, WebGlUniformLocation>,
) {
    for (binding, location) in uniform_locations {
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
            UniformValue::UnsignedInteger1(x) => state.gl().uniform1ui(Some(location), x),
            UniformValue::UnsignedInteger2(x, y) => state.gl().uniform2ui(Some(location), x, y),
            UniformValue::UnsignedInteger3(x, y, z) => {
                state.gl().uniform3ui(Some(location), x, y, z)
            }
            UniformValue::UnsignedInteger4(x, y, z, w) => {
                state.gl().uniform4ui(Some(location), x, y, z, w)
            }
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
}

pub(crate) unsafe fn draw(
    state: &mut State,
    geometry: *const dyn Geometry,
    material: *const dyn Material,
) {
    // draws entity
    if let Some(num_instances) = (*material).instanced() {
        // draw instanced
        match (*geometry).draw() {
            Draw::Arrays {
                mode,
                first,
                count: num_vertices,
            } => {
                state
                    .gl()
                    .draw_arrays_instanced(mode.gl_enum(), first, num_vertices, num_instances)
            }
            Draw::Elements {
                mode,
                count: num_vertices,
                element_type,
                offset,
                indices,
            } => {
                let buffer = match state
                    .buffer_store_mut()
                    .use_buffer(indices, BufferTarget::ElementArrayBuffer)
                {
                    Ok(buffer) => buffer,
                    Err(err) => {
                        // should log warning
                        console_log!("{}", err);
                        return;
                    }
                };

                state
                    .gl()
                    .bind_buffer(BufferTarget::ElementArrayBuffer.gl_enum(), Some(&buffer));
                state.gl().draw_elements_instanced_with_i32(
                    mode.gl_enum(),
                    num_vertices,
                    element_type.gl_enum(),
                    offset,
                    num_instances,
                );
            }
        }
    } else {
        // draw normally!
        match (*geometry).draw() {
            Draw::Arrays {
                mode,
                first,
                count: num_vertices,
            } => state.gl().draw_arrays(mode.gl_enum(), first, num_vertices),
            Draw::Elements {
                mode,
                count: num_vertices,
                element_type,
                offset,
                indices,
            } => {
                let buffer = match state
                    .buffer_store_mut()
                    .use_buffer(indices, BufferTarget::ElementArrayBuffer)
                {
                    Ok(buffer) => buffer,
                    Err(err) => {
                        // should log warning
                        console_log!("{}", err);
                        return;
                    }
                };

                state
                    .gl()
                    .bind_buffer(BufferTarget::ElementArrayBuffer.gl_enum(), Some(&buffer));
                state.gl().draw_elements_with_i32(
                    mode.gl_enum(),
                    num_vertices,
                    element_type.gl_enum(),
                    offset,
                );
            }
        }
    }
}
