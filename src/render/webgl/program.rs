use std::{borrow::Cow, collections::HashMap};

use wasm_bindgen_test::console_log;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation};

use crate::{material::WebGLMaterial, ncor::Ncor};

use super::buffer::{BufferComponentSize, BufferDescriptor, BufferTarget};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferDataType {
    Float,
    Byte,
    Short,
    Int,
    UnsignedByte,
    UnsignedShort,
    UnsignedInt,
    HalfFloat,
    Int2_10_10_10Rev,
    UnsignedInt2_10_10_10Rev,
}

impl BufferDataType {
    pub fn to_gl_enum(&self) -> u32 {
        match self {
            BufferDataType::Float => WebGl2RenderingContext::FLOAT,
            BufferDataType::Byte => WebGl2RenderingContext::BYTE,
            BufferDataType::Short => WebGl2RenderingContext::SHORT,
            BufferDataType::Int => WebGl2RenderingContext::INT,
            BufferDataType::UnsignedByte => WebGl2RenderingContext::UNSIGNED_BYTE,
            BufferDataType::UnsignedShort => WebGl2RenderingContext::UNSIGNED_SHORT,
            BufferDataType::UnsignedInt => WebGl2RenderingContext::UNSIGNED_INT,
            BufferDataType::HalfFloat => WebGl2RenderingContext::HALF_FLOAT,
            BufferDataType::Int2_10_10_10Rev => WebGl2RenderingContext::INT_2_10_10_10_REV,
            BufferDataType::UnsignedInt2_10_10_10Rev => {
                WebGl2RenderingContext::UNSIGNED_INT_2_10_10_10_REV
            }
        }
    }

    pub fn bytes_length(&self) -> i32 {
        match self {
            BufferDataType::Float => 4,
            BufferDataType::Byte => 1,
            BufferDataType::Short => 2,
            BufferDataType::Int => 4,
            BufferDataType::UnsignedByte => 1,
            BufferDataType::UnsignedShort => 2,
            BufferDataType::UnsignedInt => 4,
            BufferDataType::HalfFloat => 2,
            BufferDataType::Int2_10_10_10Rev => 4,
            BufferDataType::UnsignedInt2_10_10_10Rev => 4,
        }
    }
}

pub enum AttributeValue<'a> {
    Buffer {
        descriptor: Ncor<'a, BufferDescriptor>,
        target: BufferTarget,
        component_size: BufferComponentSize,
        data_type: BufferDataType,
        normalized: bool,
        bytes_stride: i32,
        bytes_offset: i32,
    },
    Instanced {
        descriptor: Ncor<'a, BufferDescriptor>,
        target: BufferTarget,
        component_size: BufferComponentSize,
        data_type: BufferDataType,
        normalized: bool,
        components_length_per_instance: u32,
        divisor: u32,
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
    FromGeometry(String),
    FromMaterial(String),
    FromEntity(String),
}

impl AttributeBinding {
    pub fn as_str(&self) -> &str {
        match self {
            AttributeBinding::GeometryPosition => "a_Position",
            AttributeBinding::GeometryTextureCoordinate => "a_TexCoord",
            AttributeBinding::GeometryNormal => "a_Normals",
            AttributeBinding::FromGeometry(name)
            | AttributeBinding::FromMaterial(name)
            | AttributeBinding::FromEntity(name) => name.as_str(),
        }
    }

    pub fn to_glsl<'a>(&self) -> Cow<'a, str> {
        match self {
            AttributeBinding::GeometryPosition => Cow::Borrowed("attribute vec3 a_Position;"),
            AttributeBinding::GeometryTextureCoordinate => {
                Cow::Borrowed("attribute vec3 a_TexCoords;")
            }
            AttributeBinding::GeometryNormal => Cow::Borrowed("attribute vec3 a_Normal;"),
            AttributeBinding::FromGeometry(name)
            | AttributeBinding::FromMaterial(name)
            | AttributeBinding::FromEntity(name) => Cow::Owned(name.clone()),
        }
    }
}

pub enum UniformValue {
    UnsignedInteger1(u32),
    UnsignedInteger2(u32, u32),
    UnsignedInteger3(u32, u32, u32),
    UnsignedInteger4(u32, u32, u32, u32),
    FloatVector1 {
        data: Box<dyn AsRef<[f32]>>,
        src_offset: u32,
        src_length: u32,
    },
    FloatVector2 {
        data: Box<dyn AsRef<[f32]>>,
        src_offset: u32,
        src_length: u32,
    },
    FloatVector3 {
        data: Box<dyn AsRef<[f32]>>,
        src_offset: u32,
        src_length: u32,
    },
    FloatVector4 {
        data: Box<dyn AsRef<[f32]>>,
        src_offset: u32,
        src_length: u32,
    },
    IntegerVector1 {
        data: Box<dyn AsRef<[i32]>>,
        src_offset: u32,
        src_length: u32,
    },
    IntegerVector2 {
        data: Box<dyn AsRef<[i32]>>,
        src_offset: u32,
        src_length: u32,
    },
    IntegerVector3 {
        data: Box<dyn AsRef<[i32]>>,
        src_offset: u32,
        src_length: u32,
    },
    IntegerVector4 {
        data: Box<dyn AsRef<[i32]>>,
        src_offset: u32,
        src_length: u32,
    },
    UnsignedIntegerVector1 {
        data: Box<dyn AsRef<[u32]>>,
        src_offset: u32,
        src_length: u32,
    },
    UnsignedIntegerVector2 {
        data: Box<dyn AsRef<[u32]>>,
        src_offset: u32,
        src_length: u32,
    },
    UnsignedIntegerVector3 {
        data: Box<dyn AsRef<[u32]>>,
        src_offset: u32,
        src_length: u32,
    },
    UnsignedIntegerVector4 {
        data: Box<dyn AsRef<[u32]>>,
        src_offset: u32,
        src_length: u32,
    },
    Matrix2 {
        data: Box<dyn AsRef<[f32]>>,
        transpose: bool,
        src_offset: u32,
        src_length: u32,
    },
    Matrix3 {
        data: Box<dyn AsRef<[f32]>>,
        transpose: bool,
        src_offset: u32,
        src_length: u32,
    },
    Matrix4 {
        data: Box<dyn AsRef<[f32]>>,
        transpose: bool,
        src_offset: u32,
        src_length: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UniformBinding {
    ParentModelMatrix,
    ModelMatrix,
    NormalMatrix,
    ModelViewMatrix,
    ModelViewProjMatrix,
    ViewProjMatrix,
    ActiveCameraPosition,
    ActiveCameraDirection,
    FromGeometry(String),
    FromMaterial(String),
    FromEntity(String),
}

impl UniformBinding {
    pub fn as_str(&self) -> &str {
        match self {
            UniformBinding::ParentModelMatrix => "u_ParentModelMatrix",
            UniformBinding::ModelMatrix => "u_ModelMatrix",
            UniformBinding::NormalMatrix => "u_NormalMatrix",
            UniformBinding::ModelViewMatrix => "u_ModelViewMatrix",
            UniformBinding::ViewProjMatrix => "u_ViewProjMatrix",
            UniformBinding::ModelViewProjMatrix => "u_ModelViewProjMatrix",
            UniformBinding::ActiveCameraPosition => "u_ActiveCameraPosition",
            UniformBinding::ActiveCameraDirection => "u_ActiveCameraDirection",
            UniformBinding::FromGeometry(name)
            | UniformBinding::FromMaterial(name)
            | UniformBinding::FromEntity(name) => name.as_str(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ShaderSource {
    Vertex(String),
    Fragment(String),
}

#[derive(Debug)]
struct ProgramItem {
    program: WebGlProgram,
    // shaders: Vec<WebGlShader>,
    attributes: HashMap<AttributeBinding, u32>,
    uniforms: HashMap<UniformBinding, WebGlUniformLocation>,
}

#[derive(Debug)]
pub struct ProgramStore {
    gl: WebGl2RenderingContext,
    store: HashMap<String, ProgramItem>,
}

impl ProgramStore {
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self {
            gl,
            store: HashMap::new(),
        }
    }
}

impl ProgramStore {
    // pub fn add_material<M: WebGLMaterial>(&mut self, material: &M) -> Result<(), JsError> {
    //     if self.store.contains_key(material.name()) {
    //         return Err(JsError::new(&format!(
    //             "duplicated material name {}",
    //             material.name()
    //         )));
    //     }

    //     let replaced = self.store.insert(
    //         material.name().to_string(),
    //         compile_material(&self.gl, material)?,
    //     );

    //     if let Some(material) = replaced {
    //         delete_material(&self.gl, &material);
    //     };

    //     Ok(())
    // }

    // pub fn remove_material(&mut self, name: &str) {
    //     if let Some(material) = self.store.remove(name) {
    //         delete_material(&self.gl, &material);
    //     }
    // }

    /// Gets program of a specified material from store, if not exists, compiles  and stores it.
    pub fn program_or_compile(
        &mut self,
        material: &dyn WebGLMaterial,
    ) -> Result<
        (
            &WebGlProgram,
            &HashMap<AttributeBinding, u32>,
            &HashMap<UniformBinding, WebGlUniformLocation>,
        ),
        String,
    > {
        let gl = self.gl.clone();
        let item = self
            .store
            .entry(material.name().to_string())
            .or_insert_with(move || compile_material_to_program(&gl, material).unwrap());

        Ok((&item.program, &item.attributes, &item.uniforms))
    }

    // pub fn material(&self, name: &str) -> Option<&ProgramItem> {
    //     self.store.get(name)
    // }
}

fn compile_material_to_program(
    gl: &WebGl2RenderingContext,
    material: &dyn WebGLMaterial,
) -> Result<ProgramItem, String> {
    console_log!("compile material: {}", material.name());
    let mut shaders = Vec::with_capacity(material.sources().len());
    material.sources().iter().try_for_each(|source| {
        let shader = compile_shader(gl, source)?;
        shaders.push(shader);
        Ok(()) as Result<(), String>
    })?;

    let program = create_program(gl, &shaders)?;
    Ok(ProgramItem {
        attributes: collect_attribute_locations(gl, &program, material.attribute_bindings())?,
        uniforms: collect_uniform_locations(gl, &program, material.uniform_bindings())?,
        program,
        // shaders,
    })
}

// fn delete_program(gl: &WebGl2RenderingContext, material: &ProgramItem) {
//     let ProgramItem {
//         program, shaders, ..
//     } = material;
//     gl.use_program(None);
//     shaders.into_iter().for_each(|shader| {
//         gl.delete_shader(Some(&shader));
//     });
//     gl.delete_program(Some(&program));
// }

fn compile_shader(
    gl: &WebGl2RenderingContext,
    source: &ShaderSource,
) -> Result<WebGlShader, String> {
    let (shader, code) = match source {
        ShaderSource::Vertex(code) => {
            let shader = gl
                .create_shader(WebGl2RenderingContext::VERTEX_SHADER)
                .ok_or(String::from("failed to create WebGL2 vertex shader"))?;

            (shader, code)
        }
        ShaderSource::Fragment(code) => {
            let shader = gl
                .create_shader(WebGl2RenderingContext::FRAGMENT_SHADER)
                .ok_or(String::from("failed to create WebGL2 fragment shader"))?;

            (shader, code)
        }
    };

    // attaches shader source
    gl.shader_source(&shader, &code);
    // compiles shader
    gl.compile_shader(&shader);

    let success = gl
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap();
    if !success {
        let err = gl
            .get_shader_info_log(&shader)
            .map(|err| Cow::Owned(err))
            .unwrap_or(Cow::Borrowed("unknown compile shader error"));
        gl.delete_shader(Some(&shader));
        console_log!("{err}");
        return Err(String::from(err));
    }

    Ok(shader)
}

fn create_program(
    gl: &WebGl2RenderingContext,
    shaders: &[WebGlShader],
) -> Result<WebGlProgram, String> {
    let program = gl
        .create_program()
        .ok_or(String::from("failed to create WebGL2 program"))?;

    // attaches shader to program
    for shader in shaders {
        gl.attach_shader(&program, shader);
    }
    // lins program to GPU
    gl.link_program(&program);

    let success = gl
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap();
    if !success {
        let err = gl
            .get_program_info_log(&program)
            .map(|err| Cow::Owned(err))
            .unwrap_or(Cow::Borrowed("unknown link program error"));
        gl.delete_program(Some(&program));
        console_log!("{err}");
        return Err(String::from(err.as_ref()));
    }

    Ok(program)
}

fn collect_attribute_locations(
    gl: &WebGl2RenderingContext,
    program: &WebGlProgram,
    bindings: &[AttributeBinding],
) -> Result<HashMap<AttributeBinding, u32>, String> {
    let mut locations = HashMap::with_capacity(bindings.len());

    bindings.into_iter().try_for_each(|binding| {
        let variable_name = binding.as_str();
        if locations.contains_key(binding) {
            return Err(format!("duplicated attribute name {}", variable_name));
        }

        let location = gl.get_attrib_location(program, variable_name);
        if location == -1 {
            // should log warning
        } else {
            locations.insert(binding.clone(), location as u32);
        }
        Ok(())
    })?;

    Ok(locations)
}

fn collect_uniform_locations(
    gl: &WebGl2RenderingContext,
    program: &WebGlProgram,
    bindings: &[UniformBinding],
) -> Result<HashMap<UniformBinding, WebGlUniformLocation>, String> {
    let mut locations = HashMap::with_capacity(bindings.len());

    bindings.into_iter().try_for_each(|binding| {
        let variable_name = binding.as_str();
        if locations.contains_key(binding) {
            return Err(String::from(&format!(
                "duplicated uniform name {}",
                variable_name
            )));
        }

        let location = gl.get_uniform_location(program, variable_name);
        match location {
            Some(location) => {
                locations.insert(binding.clone(), location);
            }
            None => {
                // should log warning
            }
        };

        Ok(())
    })?;

    Ok(locations)
}
