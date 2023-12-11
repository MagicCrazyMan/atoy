use std::collections::HashMap;

use wasm_bindgen_test::console_log;

use crate::{render::pp::State, entity::BorrowedMut, geometry::Geometry, material::Material};

use super::{
    buffer::{BufferComponentSize, BufferDataType, BufferDescriptor, BufferTarget, BufferItem},
    conversion::{GLboolean, GLintptr, GLsizei, GLuint, GLint, ToGlEnum},
};

#[derive(Clone)]
pub enum AttributeValue {
    Buffer {
        descriptor: BufferDescriptor,
        target: BufferTarget,
        component_size: BufferComponentSize,
        data_type: BufferDataType,
        normalized: GLboolean,
        bytes_stride: GLsizei,
        bytes_offset: GLintptr,
    },
    InstancedBuffer {
        descriptor: BufferDescriptor,
        target: BufferTarget,
        component_size: BufferComponentSize,
        data_type: BufferDataType,
        normalized: GLboolean,
        component_count_per_instance: i32,
        divisor: GLuint,
    },
    Vertex1f(f32),
    Vertex2f(f32, f32),
    Vertex3f(f32, f32, f32),
    Vertex4f(f32, f32, f32, f32),
    Vertex1fv([f32; 1]),
    Vertex2fv([f32; 2]),
    Vertex3fv([f32; 3]),
    Vertex4fv([f32; 4]),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AttributeBinding {
    GeometryPosition,
    GeometryTextureCoordinate,
    GeometryNormal,
    FromGeometry(&'static str),
    FromMaterial(&'static str),
    FromEntity(&'static str),
}

impl AttributeBinding {
    pub fn as_str(&self) -> &str {
        match self {
            AttributeBinding::GeometryPosition => "a_Position",
            AttributeBinding::GeometryTextureCoordinate => "a_TexCoord",
            AttributeBinding::GeometryNormal => "a_Normal",
            AttributeBinding::FromGeometry(name)
            | AttributeBinding::FromMaterial(name)
            | AttributeBinding::FromEntity(name) => name,
        }
    }
}


/// Binds attributes of the entity.
/// Holds returning values until finishing next draw call
/// to prevent buffer store drops the binding buffer unexpectedly.
pub fn bind_attributes(
    state: &mut State,
    entity: &BorrowedMut,
    geometry: &dyn Geometry,
    material: &dyn Material,
    attribute_locations: &HashMap<AttributeBinding, GLuint>,
) -> Vec<BufferItem> {
    let mut buffer_items = Vec::with_capacity(attribute_locations.len());
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
                let item = match state.buffer_store_mut().use_buffer(descriptor, target) {
                    Ok(buffer) => buffer,
                    Err(err) => {
                        // should log error
                        console_log!("{}", err);
                        continue;
                    }
                };

                state
                    .gl()
                    .bind_buffer(target.gl_enum(), Some(&item.gl_buffer()));
                state.gl().vertex_attrib_pointer_with_i32(
                    *location,
                    component_size as GLint,
                    data_type.gl_enum(),
                    normalized,
                    bytes_stride,
                    bytes_offset,
                );
                state.gl().enable_vertex_attrib_array(*location);

                buffer_items.push(item);
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
                let item = match state.buffer_store_mut().use_buffer(descriptor, target) {
                    Ok(buffer) => buffer,
                    Err(err) => {
                        // should log error
                        console_log!("{}", err);
                        continue;
                    }
                };

                state
                    .gl()
                    .bind_buffer(target.gl_enum(), Some(&item.gl_buffer()));

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

                buffer_items.push(item);
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

    buffer_items
}