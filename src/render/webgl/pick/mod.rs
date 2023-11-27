use std::collections::HashMap;

use uuid::Uuid;
use web_sys::{
    js_sys::Uint32Array, WebGl2RenderingContext, WebGlFramebuffer, WebGlRenderbuffer, WebGlTexture,
};

use crate::{material::Material, entity::Entity};

use super::{
    attribute::{AttributeBinding, AttributeValue},
    conversion::GLint,
    error::Error,
    program::ShaderSource,
    uniform::{UniformBinding, UniformValue},
    RenderingEntityState,
};

pub(super) struct EntityPicker {
    gl: WebGl2RenderingContext,
    width: i32,
    height: i32,
    /// Program of entity picker does not manage by program store
    // program: Option<WebGlProgram>,
    material: PickDetectionMaterial,
    /// Framebuffer.
    framebuffer: Option<WebGlFramebuffer>,
    /// Texture receiving picking result.
    /// Texture is recreated when width and height of context size changed
    index_renderbuffer: Option<(WebGlRenderbuffer, i32, i32)>,
    index_texture: Option<(WebGlTexture, i32, i32)>,
    depth_renderbuffer: Option<(WebGlRenderbuffer, i32, i32)>,
}

impl EntityPicker {
    pub(super) fn new(gl: WebGl2RenderingContext) -> Self {
        Self {
            gl,
            width: 0,
            height: 0,
            // program: None,
            material: PickDetectionMaterial::new(),
            framebuffer: None,
            index_renderbuffer: None,
            index_texture: None,
            depth_renderbuffer: None,
        }
    }

    pub(super) fn prepare(&mut self) -> Result<(), Error> {
        let gl = self.gl.clone();

        self.width = self.gl.drawing_buffer_width();
        self.height = self.gl.drawing_buffer_height();

        gl.bind_framebuffer(
            WebGl2RenderingContext::FRAMEBUFFER,
            Some(self.use_framebuffer()?),
        );

        // // binds renderbuffer for writing indices
        // let indices_renderbuffer = self.use_renderbuffer(false)?;
        // gl.bind_renderbuffer(
        //     WebGl2RenderingContext::RENDERBUFFER,
        //     Some(indices_renderbuffer),
        // );
        // gl.framebuffer_renderbuffer(
        //     WebGl2RenderingContext::FRAMEBUFFER,
        //     WebGl2RenderingContext::COLOR_ATTACHMENT0,
        //     WebGl2RenderingContext::RENDERBUFFER,
        //     Some(indices_renderbuffer),
        // );

        // binds renderbuffer for writing indices
        let index_texture = self.use_index_texture()?;
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(index_texture));
        gl.framebuffer_texture_2d(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::COLOR_ATTACHMENT0,
            WebGl2RenderingContext::TEXTURE_2D,
            Some(index_texture),
            0,
        );

        // binds renderbuffer for writing depth
        let depth_renderbuffer = self.use_renderbuffer(true)?;
        gl.bind_renderbuffer(
            WebGl2RenderingContext::RENDERBUFFER,
            Some(depth_renderbuffer),
        );
        gl.framebuffer_renderbuffer(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::DEPTH_ATTACHMENT,
            WebGl2RenderingContext::RENDERBUFFER,
            Some(depth_renderbuffer),
        );

        self.material.prepare();

        Ok(())
    }

    pub(super) fn pick(&mut self, x: i32, y: i32) -> Result<Option<&mut Entity>, Error> {
        // read index from texture
        let dst = Uint32Array::new_with_length(1);
        self.gl
            .read_pixels_with_opt_array_buffer_view(
                x,
                self.height - y, // flip Y
                1,
                1,
                WebGl2RenderingContext::RED_INTEGER,
                WebGl2RenderingContext::UNSIGNED_INT,
                Some(&dst),
            )
            .or_else(|err| Err(Error::PickFailure(err.as_string())))?;

        // gets entity from mapping
        let index = dst.get_index(0);
        let entity = if index != 0 {
            self.material
                .index2entity
                .get(&index)
                .map(|entity| unsafe { &mut **entity })
        } else {
            None
        };

        self.gl
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);
        self.gl
            .bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, None);
        self.gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

        Ok(entity)
    }

    pub(super) fn material(&mut self) -> &mut PickDetectionMaterial {
        &mut self.material
    }

    fn use_framebuffer(&mut self) -> Result<&WebGlFramebuffer, Error> {
        let framebuffer = &mut self.framebuffer;

        match framebuffer {
            Some(framebuffer) => Ok(framebuffer),
            None => {
                let framebuffer = framebuffer.insert(
                    self.gl
                        .create_framebuffer()
                        .ok_or(Error::CreateFragmentShaderFailure)?,
                );
                Ok(framebuffer)
            }
        }
    }

    fn use_renderbuffer(&mut self, is_depth: bool) -> Result<&WebGlRenderbuffer, Error> {
        let renderbuffer: *mut Option<(WebGlRenderbuffer, i32, i32)> = if is_depth {
            &mut self.depth_renderbuffer
        } else {
            &mut self.index_renderbuffer
        };

        if let Some((gl_renderbuffer, fw, fh)) = unsafe { &mut *renderbuffer } {
            // recrates a new renderbuffer if context size changed
            if self.width == *fw && self.height == *fh {
                return Ok(gl_renderbuffer);
            } else {
                self.gl.delete_renderbuffer(Some(gl_renderbuffer));
            }
        }

        // creates a new renderbuffer
        let gl_renderbuffer = self
            .gl
            .create_renderbuffer()
            .ok_or(Error::CreateRenderbufferFailure)?;
        self.gl
            .bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, Some(&gl_renderbuffer));
        if is_depth {
            self.gl.renderbuffer_storage(
                WebGl2RenderingContext::RENDERBUFFER,
                WebGl2RenderingContext::DEPTH_COMPONENT16,
                self.width,
                self.height,
            );
        } else {
            self.gl.renderbuffer_storage(
                WebGl2RenderingContext::RENDERBUFFER,
                WebGl2RenderingContext::R32UI,
                self.width,
                self.height,
            );
        }

        let (gl_renderbuffer, _, _) =
            self.index_renderbuffer
                .insert((gl_renderbuffer, self.width, self.height));
        self.gl
            .bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, None);

        Ok(gl_renderbuffer)
    }

    fn use_index_texture(&mut self) -> Result<&WebGlTexture, Error> {
        let texture: *mut Option<(WebGlTexture, i32, i32)> = &mut self.index_texture;

        if let Some((gl_texture, fw, fh)) = unsafe { &mut *texture } {
            // recrates a new texture if context size changed
            if self.width == *fw && self.height == *fh {
                return Ok(gl_texture);
            } else {
                self.gl.delete_texture(Some(gl_texture));
            }
        }

        // creates a new texture
        let gl_texture = self
            .gl
            .create_texture()
            .ok_or(Error::CreateTextureFailure)?;
        self.gl.active_texture(WebGl2RenderingContext::TEXTURE0);
        self.gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&gl_texture));
        self.gl
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                WebGl2RenderingContext::TEXTURE_2D,
                0,
                WebGl2RenderingContext::R32UI as GLint,
                self.width,
                self.height,
                0,
                WebGl2RenderingContext::RED_INTEGER,
                WebGl2RenderingContext::UNSIGNED_INT,
                None,
            )
            .or_else(|err| Err(Error::TexImageFailure(err.as_string())))?;
        self.gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

        let (gl_texture, _, _) =
            unsafe { &mut *texture }.insert((gl_texture, self.width, self.height));
        Ok(gl_texture)
    }
}

const VERTEX_SHADER_SOURCE: &'static str = include_str!("./vertex.gl");
const FRAGMENT_SHADER_SOURCE: &'static str = include_str!("./fragment.gl");

pub(crate) struct PickDetectionMaterial {
    id2index: HashMap<Uuid, UniformValue>,
    index2entity: HashMap<u32, *mut Entity>,
}

impl PickDetectionMaterial {
    fn new() -> Self {
        Self {
            id2index: HashMap::new(),
            index2entity: HashMap::new(),
        }
    }

    fn prepare(&mut self) {
        self.id2index.clear();
        self.index2entity.clear();
    }
}

impl Material for PickDetectionMaterial {
    fn name(&self) -> &'static str {
        "PickDetectionMaterial"
    }

    fn attribute_bindings(&self) -> &[AttributeBinding] {
        &[AttributeBinding::GeometryPosition]
    }

    fn uniform_bindings(&self) -> &[UniformBinding] {
        &[
            UniformBinding::ModelViewProjMatrix,
            UniformBinding::FromMaterial("u_Index"),
        ]
    }

    fn sources<'a>(&'a self) -> &[ShaderSource<'a>] {
        &[
            ShaderSource::Vertex(VERTEX_SHADER_SOURCE),
            ShaderSource::Fragment(FRAGMENT_SHADER_SOURCE),
        ]
    }

    fn attribute_value(&self, _: &str, _: &RenderingEntityState) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, name: &str, state: &RenderingEntityState) -> Option<UniformValue> {
        match name {
            "u_Index" => self.id2index.get(state.entity().id()).cloned(),
            _ => None,
        }
    }

    fn ready(&self) -> bool {
        true
    }

    fn instanced(&self) -> Option<i32> {
        None
    }

    fn prepare(&mut self, state: &RenderingEntityState) {
        let entity = state.entity();

        let index = self.id2index.len() + 1; // index 0 as nothing
        if index >= u32::MAX as usize {
            panic!("too may entities in scene");
        }

        let index = index as u32;
        self.id2index
            .insert(*entity.id(), UniformValue::UnsignedInteger1(index));
        self.index2entity.insert(index, entity);
    }
}
