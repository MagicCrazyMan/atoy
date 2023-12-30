use log::warn;

use crate::{entity::BorrowedMut, geometry::Geometry, material::Material, render::pp::State};

use super::{
    buffer::{BufferComponentSize, BufferDataType, BufferDescriptor, BufferTarget},
    conversion::{GLboolean, GLint, GLintptr, GLsizei, GLuint, ToGlEnum},
    program::ProgramItem,
    shader::VariableDataType,
};

/// Available attribute values.
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
    UnsignedInteger4(u32, u32, u32, u32),
    UnsignedIntegerVector4([u32; 4]),
    Integer4(i32, i32, i32, i32),
    IntegerVector4([i32; 4]),
}

/// Attribute binding sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttributeBinding {
    GeometryPosition,
    GeometryTextureCoordinate,
    GeometryNormal,
    FromGeometry(&'static str),
    FromMaterial(&'static str),
    FromEntity(&'static str),
}

impl AttributeBinding {
    /// Returns variable name.
    pub fn variable_name(&self) -> &str {
        match self {
            AttributeBinding::GeometryPosition => "a_Position",
            AttributeBinding::GeometryTextureCoordinate => "a_TexCoord",
            AttributeBinding::GeometryNormal => "a_Normal",
            AttributeBinding::FromGeometry(name)
            | AttributeBinding::FromMaterial(name)
            | AttributeBinding::FromEntity(name) => name,
        }
    }

    /// Returns [`VariableDataType`] for non-custom bindings.
    pub fn data_type(&self) -> Option<VariableDataType> {
        match self {
            AttributeBinding::GeometryPosition => Some(VariableDataType::FloatVec4),
            AttributeBinding::GeometryNormal => Some(VariableDataType::FloatVec4),
            AttributeBinding::GeometryTextureCoordinate => Some(VariableDataType::FloatVec2),
            AttributeBinding::FromGeometry(_)
            | AttributeBinding::FromMaterial(_)
            | AttributeBinding::FromEntity(_) => None,
        }
    }
}

pub struct BoundAttribute {
    location: u32,
    descriptor: BufferDescriptor,
}

/// Binds attributes for a entity.
/// Holds returning values until finishing next draw call
/// to prevent buffer store drops the binding buffer unexpectedly.
pub fn bind_attributes(
    state: &mut State,
    entity: &BorrowedMut,
    geometry: &dyn Geometry,
    material: &dyn Material,
    program_item: &ProgramItem,
) -> Vec<BoundAttribute> {
    let mut bounds = Vec::with_capacity(program_item.attribute_locations().len());
    for (binding, location) in program_item.attribute_locations() {
        let value = match binding {
            AttributeBinding::GeometryPosition => (*geometry).vertices(),
            AttributeBinding::GeometryTextureCoordinate => (*geometry).texture_coordinates(),
            AttributeBinding::GeometryNormal => (*geometry).normals(),
            AttributeBinding::FromGeometry(name) => (*geometry).attribute_value(name, entity),
            AttributeBinding::FromMaterial(name) => (*material).attribute_value(name, entity),
            AttributeBinding::FromEntity(name) => entity.attribute_values().get(*name).cloned(),
        };
        let Some(value) = value else {
            warn!(
                target: "BindAttributes",
                "no value specified for attribute {}",
                binding.variable_name()
            );
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
                let buffer = match state.buffer_store_mut().use_buffer(&descriptor, target) {
                    Ok(buffer) => buffer,
                    Err(err) => {
                        warn!(
                            target: "BindAttributes",
                            "use buffer store error: {}",
                            err
                        );
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
                state.gl().bind_buffer(target.gl_enum(), None);

                bounds.push(BoundAttribute {
                    location: *location,
                    descriptor,
                });
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
                let buffer = match state.buffer_store_mut().use_buffer(&descriptor, target) {
                    Ok(buffer) => buffer,
                    Err(err) => {
                        warn!(
                            target: "BindAttributes",
                            "use buffer store error: {}",
                            err
                        );
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

                    bounds.push(BoundAttribute {
                        location: offset_location,
                        descriptor: descriptor.clone(),
                    });
                }
                state.gl().bind_buffer(target.gl_enum(), None);
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
            AttributeValue::UnsignedInteger4(x, y, z, w) => {
                state.gl().vertex_attrib_i4ui(*location, x, y, z, w)
            }
            AttributeValue::Integer4(x, y, z, w) => {
                state.gl().vertex_attrib_i4i(*location, x, y, z, w)
            }
            AttributeValue::IntegerVector4(mut values) => state
                .gl()
                .vertex_attrib_i4iv_with_i32_array(*location, &mut values),
            AttributeValue::UnsignedIntegerVector4(mut values) => state
                .gl()
                .vertex_attrib_i4uiv_with_u32_array(*location, &mut values),
        };
    }

    bounds
}

/// Unbinds all attributes after draw calls.
///
/// If you bind buffer attributes ever,
/// remember to unbind them by yourself or use this function.
pub fn unbind_attributes(state: &mut State, bounds: Vec<BoundAttribute>) {
    for BoundAttribute {
        location,
        descriptor,
    } in bounds
    {
        state.gl().disable_vertex_attrib_array(location);
        state.buffer_store_mut().unuse_buffer(&descriptor);
    }
}
