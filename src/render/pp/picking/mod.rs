use std::{any::Any, collections::HashMap};

use web_sys::{
    js_sys::Uint32Array, WebGl2RenderingContext, WebGlFramebuffer, WebGlRenderbuffer, WebGlTexture,
};

use crate::{
    entity::{BorrowedMut, Strong},
    material::Material,
    render::webgl::{
        attribute::{AttributeBinding, AttributeValue},
        draw::{bind_attributes, bind_uniforms, draw},
        error::Error,
        program::ShaderSource,
        uniform::{UniformBinding, UniformValue},
    },
};

use super::{
    standard::{ResetWebGLState, StandardEntitiesCollector, StandardSetup, UpdateCameraFrame},
    Executor, Pipeline, ResourceSource, State, Stuff,
};

pub fn create_picking_pipeline(
    position_key: impl Into<String>,
    picked_key: impl Into<String>,
) -> Pipeline {
    let mut pipeline = Pipeline::new();

    pipeline.add_executor("__clear", StandardSetup);
    pipeline.add_executor("__update_camera", UpdateCameraFrame);
    pipeline.add_executor(
        "__collector",
        StandardEntitiesCollector::new(ResourceSource::runtime("entities")),
    );
    pipeline.add_executor(
        "__picking",
        Picking::new(
            ResourceSource::persist(position_key),
            ResourceSource::runtime("entities"),
            ResourceSource::persist(picked_key),
        ),
    );
    pipeline.add_executor("__reset", ResetWebGLState);

    pipeline.connect("__clear", "__update_camera").unwrap();
    pipeline.connect("__update_camera", "__collector").unwrap();
    pipeline.connect("__collector", "__picking").unwrap();
    pipeline.connect("__picking", "__reset").unwrap();

    pipeline
}

/// Picking detection.
///
/// # Get Resources & Data Type
/// - `entities`: [`Vec<Strong>`], a list contains entities to pick.
/// - `position`: `(i32, i32)`, a window coordinate position, skip picking if none.
///
/// # Provide Resources & Data Type
/// - `pick`: [`Weak`](crate::entity::Weak), picked result.
/// - `persist`: If `true`, provides data in both `runtime_resources` and `persist_resources`.
pub struct Picking {
    entities: ResourceSource,
    position: ResourceSource,
    picked: ResourceSource,
    pixel: Uint32Array,
    framebuffer: Option<WebGlFramebuffer>,
    renderbuffer: Option<(WebGlRenderbuffer, u32, u32)>,
    texture: Option<(WebGlTexture, u32, u32)>,
    material: PickingMaterial,
}

impl Picking {
    pub fn new(position: ResourceSource, entities: ResourceSource, result: ResourceSource) -> Self {
        Self {
            entities,
            position,
            picked: result,
            pixel: Uint32Array::new_with_length(1),
            framebuffer: None,
            renderbuffer: None,
            texture: None,
            material: PickingMaterial { index: 0 },
        }
    }

    fn use_framebuffer(&mut self, gl: &WebGl2RenderingContext) -> Result<WebGlFramebuffer, Error> {
        let framebuffer = &mut self.framebuffer;
        let framebuffer = match framebuffer {
            Some(framebuffer) => framebuffer.clone(),
            None => {
                let fbo = gl
                    .create_framebuffer()
                    .ok_or(Error::CreateFramebufferFailure)?;
                *framebuffer = Some(fbo.clone());
                fbo
            }
        };

        Ok(framebuffer)
    }

    fn use_depth_renderbuffer(&mut self, state: &State) -> Result<WebGlRenderbuffer, Error> {
        let w = state.canvas.width();
        let h = state.canvas.height();

        if let Some((renderbuffer, width, height)) = &self.renderbuffer {
            if w == *width && h == *height {
                return Ok(renderbuffer.clone());
            } else {
                state.gl.delete_renderbuffer(Some(renderbuffer));
            }
        }

        let rb = state
            .gl
            .create_renderbuffer()
            .ok_or(Error::CreateRenderbufferFailure)?;

        state
            .gl
            .bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, Some(&rb));
        state.gl.renderbuffer_storage(
            WebGl2RenderingContext::RENDERBUFFER,
            WebGl2RenderingContext::DEPTH_COMPONENT16,
            w as i32,
            h as i32,
        );
        state
            .gl
            .bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, None);

        self.renderbuffer = Some((rb.clone(), w, h));

        Ok(rb)
    }

    fn use_texture(&mut self, state: &State) -> Result<WebGlTexture, Error> {
        let w = state.canvas.width();
        let h = state.canvas.height();

        if let Some((texture, width, height)) = &self.texture {
            if w == *width && h == *height {
                return Ok(texture.clone());
            } else {
                state.gl.delete_texture(Some(texture));
            }
        }

        let tx = state
            .gl
            .create_texture()
            .ok_or(Error::CreateTextureFailure)?;

        state.gl.active_texture(WebGl2RenderingContext::TEXTURE0);
        state
            .gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&tx));

        state
            .gl
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                WebGl2RenderingContext::TEXTURE_2D,
                0,
                WebGl2RenderingContext::R32UI as i32,
                w as i32,
                h as i32,
                0,
                WebGl2RenderingContext::RED_INTEGER,
                WebGl2RenderingContext::UNSIGNED_INT,
                None,
            )
            .or_else(|err| Err(Error::TexImageFailure(err.as_string())))?;
        state
            .gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

        self.texture = Some((tx.clone(), w, h));

        Ok(tx)
    }
}

impl Executor for Picking {
    fn execute(
        &mut self,
        state: &mut State,
        stuff: &mut dyn Stuff,
        runtime_resources: &mut HashMap<String, Box<dyn Any>>,
        persist_resources: &mut HashMap<String, Box<dyn Any>>,
    ) -> Result<(), Error> {
        let position = match &self.position {
            ResourceSource::Runtime(key) => runtime_resources.get(key.as_str()),
            ResourceSource::Persist(key) => persist_resources.get(key.as_str()),
        };
        let Some((x, y)) = position.and_then(|position| position.downcast_ref::<(i32, i32)>())
        else {
            return Ok(());
        };

        let entities = match &self.entities {
            ResourceSource::Runtime(key) => runtime_resources.get(key.as_str()),
            ResourceSource::Persist(key) => persist_resources.get(key.as_str()),
        };
        let Some(entities) = entities.and_then(|entities| entities.downcast_ref::<Vec<Strong>>())
        else {
            return Ok(());
        };

        if entities.len() - 1 > u32::MAX as usize {
            // should warning
            return Ok(());
        }

        // replace framebuffer for pick detection
        let framebuffer = self.use_framebuffer(&state.gl)?;
        let renderbuffer = self.use_depth_renderbuffer(state)?;
        let texture = self.use_texture(state)?;
        state.gl.active_texture(WebGl2RenderingContext::TEXTURE0);
        state
            .gl
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&framebuffer));
        state
            .gl
            .bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, Some(&renderbuffer));
        state
            .gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
        state.gl.framebuffer_texture_2d(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::COLOR_ATTACHMENT0,
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&texture),
            0,
        );
        state.gl.framebuffer_renderbuffer(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::DEPTH_ATTACHMENT,
            WebGl2RenderingContext::RENDERBUFFER,
            Some(&renderbuffer),
        );

        state
            .gl
            .clear_bufferuiv_with_u32_array(WebGl2RenderingContext::COLOR, 0, &[0, 0, 0, 0]);
        state
            .gl
            .clear_bufferfv_with_f32_array(WebGl2RenderingContext::DEPTH, 0, &[1.0]);

        // prepare material
        let program = state.program_store.use_program(&self.material)?;
        state.gl.use_program(Some(program.gl_program()));

        // render each entity by material
        for (index, entity) in entities.iter().enumerate() {
            let entity = entity.borrow_mut();
            let Some(geometry) = entity.geometry() else {
                continue;
            };

            // sets index and window position for current draw
            self.material.index = (index + 1) as u32;

            bind_attributes(
                state,
                &entity,
                geometry,
                &self.material,
                program.attribute_locations(),
            );
            bind_uniforms(
                state,
                stuff,
                &entity,
                geometry,
                &self.material,
                program.uniform_locations(),
            );
            draw(state, &*geometry, &self.material);
        }

        // get result
        state
            .gl
            .read_pixels_with_opt_array_buffer_view(
                *x,
                state.canvas.height() as i32 - *y,
                1,
                1,
                WebGl2RenderingContext::RED_INTEGER,
                WebGl2RenderingContext::UNSIGNED_INT,
                Some(&self.pixel),
            )
            .or_else(|err| Err(Error::CommonWebGLError(err.as_string())))?;

        if let Some(entity) = entities
            .get(self.pixel.get_index(0) as usize - 1)
            .map(|entity| entity.downgrade())
        {
            let picked = Box::new(entity.clone());
            match &self.picked {
                ResourceSource::Runtime(key) => runtime_resources.insert(key.clone(), picked),
                ResourceSource::Persist(key) => persist_resources.insert(key.clone(), picked),
            };
        }

        // resets WebGL status
        state
            .gl
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);
        state
            .gl
            .bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, None);
        state
            .gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

        Ok(())
    }
}

struct PickingMaterial {
    index: u32,
}

impl Material for PickingMaterial {
    fn name(&self) -> &'static str {
        "PickingMaterial"
    }

    fn attribute_bindings(&self) -> &[AttributeBinding] {
        &[AttributeBinding::GeometryPosition]
    }

    fn uniform_bindings(&self) -> &[UniformBinding] {
        &[
            UniformBinding::ModelMatrix,
            UniformBinding::ViewProjMatrix,
            UniformBinding::FromMaterial("u_Index"),
        ]
    }

    fn sources(&self) -> &[ShaderSource] {
        &[
            ShaderSource::Vertex(include_str!("./vertex.glsl")),
            ShaderSource::Fragment(include_str!("./fragment.glsl")),
        ]
    }

    fn attribute_value(&self, _: &str, _: &BorrowedMut) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, name: &str, _: &BorrowedMut) -> Option<UniformValue> {
        match name {
            "u_Index" => Some(UniformValue::UnsignedInteger1(self.index)),
            _ => None,
        }
    }

    fn ready(&self) -> bool {
        true
    }

    fn instanced(&self) -> Option<i32> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
