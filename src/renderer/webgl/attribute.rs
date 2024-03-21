use std::{borrow::Cow, sync::OnceLock};

use regex::Regex;
use web_sys::{WebGl2RenderingContext, WebGlVertexArrayObject};

use crate::value::Readonly;

use super::{
    buffer::{Buffer, BufferComponentSize, BufferDataType},
    error::Error,
};

/// Available attribute values.
pub enum AttributeValue<'a> {
    ArrayBuffer {
        buffer: Readonly<'a, Buffer>,
        component_size: BufferComponentSize,
        data_type: BufferDataType,
        normalized: bool,
        bytes_stride: usize,
        byte_offset: usize,
    },
    InstancedBuffer {
        buffer: Readonly<'a, Buffer>,
        component_size: BufferComponentSize,
        data_type: BufferDataType,
        normalized: bool,
        component_count_per_instance: usize,
        divisor: usize,
    },
    Vertex1f(f32),
    Vertex2f(f32, f32),
    Vertex3f(f32, f32, f32),
    Vertex4f(f32, f32, f32, f32),
    Integer4(i32, i32, i32, i32),
    UnsignedInteger4(u32, u32, u32, u32),
    Vertex1fv([f32; 1]),
    Vertex2fv([f32; 2]),
    Vertex3fv([f32; 3]),
    Vertex4fv([f32; 4]),
    IntegerVector4([i32; 4]),
    UnsignedIntegerVector4([u32; 4]),
}

/// Attribute internal bindings.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AttributeBinding {
    GeometryPosition,
    GeometryTextureCoordinate,
    GeometryNormal,
    GeometryTangent,
    GeometryBitangent,
    FromEntity(Cow<'static, str>),
    FromGeometry(Cow<'static, str>),
    FromMaterial(Cow<'static, str>),
    Custom(Cow<'static, str>),
}

impl AttributeBinding {
    /// Returns variable name.
    pub fn variable_name(&self) -> &str {
        match self {
            AttributeBinding::GeometryPosition => "a_Position",
            AttributeBinding::GeometryTextureCoordinate => "a_TexCoord",
            AttributeBinding::GeometryNormal => "a_Normal",
            AttributeBinding::GeometryTangent => "a_Tangent",
            AttributeBinding::GeometryBitangent => "a_Bitangent",
            AttributeBinding::FromEntity(name)
            | AttributeBinding::FromGeometry(name)
            | AttributeBinding::FromMaterial(name)
            | AttributeBinding::Custom(name) => name.as_ref(),
        }
    }
}

/// Regular expression to find where to get value for a attribute.
const GLSL_ATTRIBUTE_REGEX_STRING: &'static str = "a_(.*)_(.*)";

static GLSL_ATTRIBUTE_REGEX: OnceLock<Regex> = OnceLock::new();

impl<T> From<T> for AttributeBinding
where
    T: AsRef<str>,
{
    fn from(value: T) -> Self {
        let value = value.as_ref();
        match value {
            "a_Position" => AttributeBinding::GeometryPosition,
            "a_TexCoord" => AttributeBinding::GeometryTextureCoordinate,
            "a_Normal" => AttributeBinding::GeometryNormal,
            "a_Tangent" => AttributeBinding::GeometryTangent,
            "a_Bitangent" => AttributeBinding::GeometryBitangent,
            _ => {
                let regex = GLSL_ATTRIBUTE_REGEX
                    .get_or_init(|| Regex::new(GLSL_ATTRIBUTE_REGEX_STRING).unwrap());

                let name = Cow::Owned(value.to_string());

                // when regular expression capture nothing, fallback to FromMaterial
                let Some(captures) = regex.captures(value) else {
                    return AttributeBinding::Custom(name);
                };
                let Some(c1) = captures.get(1) else {
                    return AttributeBinding::Custom(name);
                };

                match c1.as_str() {
                    "Entity" => AttributeBinding::FromEntity(name),
                    "Geometry" => AttributeBinding::FromGeometry(name),
                    "Material" => AttributeBinding::FromMaterial(name),
                    _ => AttributeBinding::Custom(name),
                }
            }
        }
    }
}

/// Unbinder to unbind vertex attribute array.
#[derive(Debug, Clone)]
pub struct VertexAttributeArrayUnbinder {
    gl: WebGl2RenderingContext,
    location: u32,
}

impl VertexAttributeArrayUnbinder {
    pub(crate) fn new(location: u32, gl: WebGl2RenderingContext) -> Self {
        Self { gl, location }
    }

    pub fn unbind(self) {
        self.gl.disable_vertex_attrib_array(self.location)
    }
}

/// A wrapper for vertex array object.
#[derive(Debug, Clone)]
pub struct VertexArray {
    gl: WebGl2RenderingContext,
    vao: WebGlVertexArrayObject,
}

impl VertexArray {
    /// Constructs a new vertex array object wrapper.
    pub fn new(gl: WebGl2RenderingContext) -> Result<Self, Error> {
        let vao = gl
            .create_vertex_array()
            .ok_or(Error::CreateVertexArrayObjectFailure)?;
        Ok(Self { gl, vao })
    }

    pub fn bind(&self) {
        self.gl.bind_vertex_array(Some(&self.vao))
    }

    pub fn unbind(&self) {
        self.gl.bind_vertex_array(None)
    }
}
