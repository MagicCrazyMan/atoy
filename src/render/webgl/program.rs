use std::collections::HashMap;

use wasm_bindgen_test::console_log;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation};

use crate::material::WebGLMaterial;

use super::{
    buffer::{BufferComponentSize, BufferDataType, BufferDescriptor, BufferTarget},
    error::Error,
    texture::{TextureDescriptor, TextureParameter},
};

pub enum AttributeValue<'a> {
    Buffer {
        descriptor: &'a BufferDescriptor,
        target: BufferTarget,
        component_size: BufferComponentSize,
        data_type: BufferDataType,
        normalized: bool,
        bytes_stride: i32,
        bytes_offset: i32,
    },
    InstancedBuffer {
        descriptor: &'a BufferDescriptor,
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

pub enum UniformValue<'a> {
    UnsignedInteger1(u32),
    UnsignedInteger2(u32, u32),
    UnsignedInteger3(u32, u32, u32),
    UnsignedInteger4(u32, u32, u32, u32),
    FloatVector1 {
        data: &'a dyn AsRef<[f32]>,
        src_offset: u32,
        src_length: u32,
    },
    FloatVector2 {
        data: &'a dyn AsRef<[f32]>,
        src_offset: u32,
        src_length: u32,
    },
    FloatVector3 {
        data: &'a dyn AsRef<[f32]>,
        src_offset: u32,
        src_length: u32,
    },
    FloatVector4 {
        data: &'a dyn AsRef<[f32]>,
        src_offset: u32,
        src_length: u32,
    },
    IntegerVector1 {
        data: &'a dyn AsRef<[i32]>,
        src_offset: u32,
        src_length: u32,
    },
    IntegerVector2 {
        data: &'a dyn AsRef<[i32]>,
        src_offset: u32,
        src_length: u32,
    },
    IntegerVector3 {
        data: &'a dyn AsRef<[i32]>,
        src_offset: u32,
        src_length: u32,
    },
    IntegerVector4 {
        data: &'a dyn AsRef<[i32]>,
        src_offset: u32,
        src_length: u32,
    },
    UnsignedIntegerVector1 {
        data: &'a dyn AsRef<[u32]>,
        src_offset: u32,
        src_length: u32,
    },
    UnsignedIntegerVector2 {
        data: &'a dyn AsRef<[u32]>,
        src_offset: u32,
        src_length: u32,
    },
    UnsignedIntegerVector3 {
        data: &'a dyn AsRef<[u32]>,
        src_offset: u32,
        src_length: u32,
    },
    UnsignedIntegerVector4 {
        data: &'a dyn AsRef<[u32]>,
        src_offset: u32,
        src_length: u32,
    },
    Matrix2 {
        data: &'a dyn AsRef<[f32]>,
        transpose: bool,
        src_offset: u32,
        src_length: u32,
    },
    Matrix3 {
        data: &'a dyn AsRef<[f32]>,
        transpose: bool,
        src_offset: u32,
        src_length: u32,
    },
    Matrix4 {
        data: &'a dyn AsRef<[f32]>,
        transpose: bool,
        src_offset: u32,
        src_length: u32,
    },
    Texture {
        descriptor: &'a TextureDescriptor,
        params: Vec<TextureParameter>,
        active_unit: u32,
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
    FromGeometry(&'static str),
    FromMaterial(&'static str),
    FromEntity(&'static str),
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
            | UniformBinding::FromEntity(name) => name,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ShaderSource<'a> {
    Vertex(&'a str),
    Fragment(&'a str),
}

#[derive(Debug)]
pub struct ProgramItem {
    program: WebGlProgram,
    // shaders: Vec<WebGlShader>,
    attributes: HashMap<AttributeBinding, u32>,
    uniforms: HashMap<UniformBinding, WebGlUniformLocation>,
}

impl ProgramItem {
    pub fn program(&self) -> &WebGlProgram {
        &self.program
    }

    pub fn attribute_locations(&self) -> &HashMap<AttributeBinding, u32> {
        &self.attributes
    }

    pub fn uniform_locations(&self) -> &HashMap<UniformBinding, WebGlUniformLocation> {
        &self.uniforms
    }
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
    ) -> Result<&ProgramItem, Error> {
        let gl = self.gl.clone();
        let item = self
            .store
            .entry(material.name().to_string())
            .or_insert_with(move || compile_material(&gl, material).unwrap());

        Ok(item)
    }

    // pub fn material(&self, name: &str) -> Option<&ProgramItem> {
    //     self.store.get(name)
    // }
}

fn compile_material(
    gl: &WebGl2RenderingContext,
    material: &dyn WebGLMaterial,
) -> Result<ProgramItem, Error> {
    let mut shaders = Vec::with_capacity(material.sources().len());
    material.sources().iter().try_for_each(|source| {
        shaders.push(compile_shader(gl, source)?);
        Ok(()) as Result<(), Error>
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
) -> Result<WebGlShader, Error> {
    let (shader, code) = match source {
        ShaderSource::Vertex(code) => {
            let shader = gl
                .create_shader(WebGl2RenderingContext::VERTEX_SHADER)
                .ok_or(Error::CreateVertexShaderFailure)?;
            (shader, code)
        }
        ShaderSource::Fragment(code) => {
            let shader = gl
                .create_shader(WebGl2RenderingContext::FRAGMENT_SHADER)
                .ok_or(Error::CreateFragmentShaderFailure)?;
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
        let err = gl.get_shader_info_log(&shader).map(|err| err);
        gl.delete_shader(Some(&shader));
        Err(Error::CompileShaderFailure(err))
    } else {
        Ok(shader)
    }
}

fn create_program(
    gl: &WebGl2RenderingContext,
    shaders: &[WebGlShader],
) -> Result<WebGlProgram, Error> {
    let program = gl.create_program().ok_or(Error::CreateProgramFailure)?;

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
        let err = gl.get_program_info_log(&program).map(|err| err);
        gl.delete_program(Some(&program));
        Err(Error::CompileProgramFailure(err))
    } else {
        Ok(program)
    }
}

fn collect_attribute_locations(
    gl: &WebGl2RenderingContext,
    program: &WebGlProgram,
    bindings: &[AttributeBinding],
) -> Result<HashMap<AttributeBinding, u32>, Error> {
    let mut locations = HashMap::with_capacity(bindings.len());

    bindings.into_iter().for_each(|binding| {
        let variable_name = binding.as_str();
        let location = gl.get_attrib_location(program, variable_name);
        if location == -1 {
            // should log warning
            console_log!("failed to get attribute location of {}", variable_name);
        } else {
            locations.insert(binding.clone(), location as u32);
        }
    });

    Ok(locations)
}

fn collect_uniform_locations(
    gl: &WebGl2RenderingContext,
    program: &WebGlProgram,
    bindings: &[UniformBinding],
) -> Result<HashMap<UniformBinding, WebGlUniformLocation>, Error> {
    let mut locations = HashMap::with_capacity(bindings.len());

    bindings.into_iter().for_each(|binding| {
        let variable_name = binding.as_str();
        let location = gl.get_uniform_location(program, variable_name);
        match location {
            None => {
                // should log warning
                console_log!("failed to get uniform location of {}", variable_name);
            }
            Some(location) => {
                locations.insert(binding.clone(), location);
            }
        }
    });

    Ok(locations)
}
