use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::Debug,
};

use wasm_bindgen_test::console_log;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation};

use crate::material::Material;

use super::{
    attribute::AttributeBinding, conversion::GLuint, error::Error, uniform::UniformBinding,
};

#[derive(Debug, Clone)]
pub enum ShaderSource<'a> {
    Vertex(&'a str),
    Fragment(&'a str),
}

#[derive(Debug)]
pub struct ProgramItem {
    program: WebGlProgram,
    // shaders: Vec<WebGlShader>,
    attributes: HashMap<AttributeBinding, GLuint>,
    uniforms: HashMap<UniformBinding, WebGlUniformLocation>,
}

impl ProgramItem {
    pub fn program(&self) -> &WebGlProgram {
        &self.program
    }

    pub fn attribute_locations(&self) -> &HashMap<AttributeBinding, GLuint> {
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
    pub fn use_program<'a>(
        &'a mut self,
        material: &dyn Material,
    ) -> Result<&'a ProgramItem, Error> {
        let store = &mut self.store;

        match store.entry(material.name().to_string()) {
            Entry::Occupied(occupied) => {
                let item: *const ProgramItem = occupied.get();
                Ok(unsafe { &*item })
            }
            Entry::Vacant(vacant) => {
                let item = vacant.insert(compile_material(&self.gl, material)?);
                Ok(item)
            }
        }
    }

    // pub fn material(&self, name: &str) -> Option<&ProgramItem> {
    //     self.store.get(name)
    // }
}

fn compile_material(
    gl: &WebGl2RenderingContext,
    material: &dyn Material,
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

pub fn compile_shader(
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

pub fn create_program(
    gl: &WebGl2RenderingContext,
    shaders: &[WebGlShader],
) -> Result<WebGlProgram, Error> {
    let program = gl.create_program().ok_or(Error::CreateProgramFailure)?;

    // attaches shader to program
    for shader in shaders {
        gl.attach_shader(&program, shader);
    }
    // links program
    gl.link_program(&program);
    // validates program
    gl.validate_program(&program);

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

pub fn collect_attribute_locations(
    gl: &WebGl2RenderingContext,
    program: &WebGlProgram,
    bindings: &[AttributeBinding],
) -> Result<HashMap<AttributeBinding, GLuint>, Error> {
    let mut locations = HashMap::with_capacity(bindings.len());

    bindings.into_iter().for_each(|binding| {
        let variable_name = binding.as_str();
        let location = gl.get_attrib_location(program, variable_name);
        if location == -1 {
            // should log warning
            console_log!("failed to get attribute location of {}", variable_name);
        } else {
            locations.insert(binding.clone(), location as GLuint);
        }
    });

    Ok(locations)
}

pub fn collect_uniform_locations(
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
