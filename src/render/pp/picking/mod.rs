use std::{any::Any, collections::HashMap};

use wasm_bindgen::JsValue;
use web_sys::{
    js_sys::{Array, Uint32Array},
    WebGl2RenderingContext, WebGlFramebuffer, WebGlRenderbuffer, WebGlTexture,
};

use crate::{
    entity::{BorrowedMut, Strong},
    material::{Material, Transparency},
    render::webgl::{
        attribute::{bind_attributes, AttributeBinding, AttributeValue},
        draw::draw,
        error::Error,
        program::ShaderSource,
        uniform::{UniformBinding, UniformValue, bind_uniforms},
    },
};

use super::{
    standard::{ResetWebGLState, StandardEntitiesCollector, StandardSetup, UpdateCameraFrame},
    Executor, Pipeline, ResourceSource, State, Stuff,
};

pub fn create_picking_pipeline(
    position_key: impl Into<String>,
    picked_entity_key: impl Into<String>,
    picked_position_key: impl Into<String>,
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
            ResourceSource::persist(picked_entity_key),
            ResourceSource::persist(picked_position_key),
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
/// - `picked_entity`: [`Weak`](crate::entity::Weak), picked entity.
/// - `picked_position`: `[f32; 4]`, picked position. Picked position regards as `None` if components are all `0.0`.
pub struct Picking {
    entities: ResourceSource,
    position: ResourceSource,
    picked_entity: ResourceSource,
    picked_position: ResourceSource,
    pixel: Uint32Array,
    framebuffer: Option<WebGlFramebuffer>,
    renderbuffer: Option<(WebGlRenderbuffer, u32, u32)>,
    indices_texture: Option<(WebGlTexture, u32, u32)>,
    positions_texture: Option<(WebGlTexture, u32, u32)>,
    material: PickingMaterial,
}

impl Picking {
    pub fn new(
        position: ResourceSource,
        entities: ResourceSource,
        picked_entity: ResourceSource,
        picked_position: ResourceSource,
    ) -> Self {
        Self {
            entities,
            position,
            picked_entity,
            picked_position,
            pixel: Uint32Array::new_with_length(4),
            framebuffer: None,
            renderbuffer: None,
            indices_texture: None,
            positions_texture: None,
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
            WebGl2RenderingContext::DEPTH_COMPONENT32F,
            w as i32,
            h as i32,
        );
        state
            .gl
            .bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, None);

        self.renderbuffer = Some((rb.clone(), w, h));

        Ok(rb)
    }

    fn use_indices_texture(&mut self, state: &State) -> Result<WebGlTexture, Error> {
        let w = state.canvas.width();
        let h = state.canvas.height();

        if let Some((texture, width, height)) = &self.indices_texture {
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

        self.indices_texture = Some((tx.clone(), w, h));

        Ok(tx)
    }

    fn use_positions_texture(&mut self, state: &State) -> Result<WebGlTexture, Error> {
        let w = state.canvas.width();
        let h = state.canvas.height();

        if let Some((texture, width, height)) = &self.positions_texture {
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
                WebGl2RenderingContext::RGBA32UI as i32,
                w as i32,
                h as i32,
                0,
                WebGl2RenderingContext::RGBA_INTEGER,
                WebGl2RenderingContext::UNSIGNED_INT,
                None,
            )
            .or_else(|err| Err(Error::TexImageFailure(err.as_string())))?;
        state
            .gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

        self.positions_texture = Some((tx.clone(), w, h));

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
        // clear first
        match &self.picked_entity {
            ResourceSource::Runtime(key) => runtime_resources.remove(key),
            ResourceSource::Persist(key) => persist_resources.remove(key),
        };
        match &self.picked_position {
            ResourceSource::Runtime(key) => runtime_resources.remove(key),
            ResourceSource::Persist(key) => persist_resources.remove(key),
        };

        let position = match &self.position {
            ResourceSource::Runtime(key) => runtime_resources.get(key.as_str()),
            ResourceSource::Persist(key) => persist_resources.get(key.as_str()),
        };
        let Some((x, y)) = position.and_then(|position| position.downcast_ref::<(i32, i32)>())
        else {
            return Ok(());
        };
        let (x, y) = (*x, *y);

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
        let indices_texture = self.use_indices_texture(state)?;
        let positions_texture = self.use_positions_texture(state)?;
        state.gl.active_texture(WebGl2RenderingContext::TEXTURE0);
        state
            .gl
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&framebuffer));
        state
            .gl
            .bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, Some(&renderbuffer));
        state.gl.framebuffer_renderbuffer(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::DEPTH_ATTACHMENT,
            WebGl2RenderingContext::RENDERBUFFER,
            Some(&renderbuffer),
        );
        state
            .gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&indices_texture));
        state.gl.framebuffer_texture_2d(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::COLOR_ATTACHMENT0,
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&indices_texture),
            0,
        );
        state
            .gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&positions_texture));
        state.gl.framebuffer_texture_2d(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::COLOR_ATTACHMENT1,
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&positions_texture),
            0,
        );

        let draw_buffers = Array::new();
        draw_buffers.push(&JsValue::from_f64(
            WebGl2RenderingContext::COLOR_ATTACHMENT0 as f64,
        ));
        draw_buffers.push(&JsValue::from_f64(
            WebGl2RenderingContext::COLOR_ATTACHMENT1 as f64,
        ));
        state.gl.draw_buffers(&draw_buffers);

        state
            .gl
            .clear_bufferuiv_with_u32_array(WebGl2RenderingContext::COLOR, 0, &[0, 0, 0, 0]);
        state
            .gl
            .clear_bufferuiv_with_u32_array(WebGl2RenderingContext::COLOR, 1, &[0, 0, 0, 0]);
        state
            .gl
            .clear_bufferfv_with_f32_array(WebGl2RenderingContext::DEPTH, 0, &[1.0]);

        state.gl.disable(WebGl2RenderingContext::BLEND);

        // prepare material
        let program = state.program_store.use_program(&self.material)?;
        state.gl.use_program(Some(program.gl_program()));

        // render each entity by material
        for (index, entity) in entities.iter().enumerate() {
            let entity = entity.borrow_mut();
            let Some(geometry) = entity.geometry() else {
                continue;
            };

            // do not pick entity has no material or transparent material
            if let Some(material) = entity.material() {
                if material.transparency() == Transparency::Transparent {
                    continue;
                }
            } else {
                continue;
            }

            // sets index and window position for current draw
            self.material.index = (index + 1) as u32;

            let items = bind_attributes(
                state,
                &entity,
                geometry,
                &self.material,
                &program,
            );
            bind_uniforms(
                state,
                stuff,
                &entity,
                geometry,
                &self.material,
                &program,
            );
            draw(state, &*geometry, &self.material);

            drop(items);
        }

        // gets picking entity
        state
            .gl
            .read_buffer(WebGl2RenderingContext::COLOR_ATTACHMENT0);
        state
            .gl
            .read_pixels_with_opt_array_buffer_view(
                x,
                state.canvas.height() as i32 - y,
                1,
                1,
                WebGl2RenderingContext::RED_INTEGER,
                WebGl2RenderingContext::UNSIGNED_INT,
                Some(&self.pixel),
            )
            .or_else(|err| Err(Error::CommonWebGLError(err.as_string())))?;

        let index = self.pixel.get_index(0) as usize;
        if index > 0 {
            if let Some(entity) = entities.get(index - 1).map(|entity| entity.downgrade()) {
                let picked = Box::new(entity.clone());
                match &self.picked_entity {
                    ResourceSource::Runtime(key) => runtime_resources.insert(key.clone(), picked),
                    ResourceSource::Persist(key) => persist_resources.insert(key.clone(), picked),
                };
            }
        }

        // gets picking position
        state
            .gl
            .read_buffer(WebGl2RenderingContext::COLOR_ATTACHMENT1);
        state
            .gl
            .read_pixels_with_opt_array_buffer_view(
                x,
                state.canvas.height() as i32 - y,
                1,
                1,
                WebGl2RenderingContext::RGBA_INTEGER,
                WebGl2RenderingContext::UNSIGNED_INT,
                Some(&self.pixel),
            )
            .or_else(|err| Err(Error::CommonWebGLError(err.as_string())))?;
        let position = [
            f32::from_ne_bytes(self.pixel.get_index(0).to_ne_bytes()),
            f32::from_ne_bytes(self.pixel.get_index(1).to_ne_bytes()),
            f32::from_ne_bytes(self.pixel.get_index(2).to_ne_bytes()),
            f32::from_ne_bytes(self.pixel.get_index(3).to_ne_bytes()),
        ]; // converts unsigned int back to float
        if position != [0.0, 0.0, 0.0, 0.0] {
            match &self.picked_position {
                ResourceSource::Runtime(key) => {
                    runtime_resources.insert(key.to_string(), Box::new(position))
                }
                ResourceSource::Persist(key) => {
                    persist_resources.insert(key.to_string(), Box::new(position))
                }
            };
        }

        // resets WebGL status
        state
            .gl
            .read_buffer(WebGl2RenderingContext::COLOR_ATTACHMENT0);
        state.gl.enable(WebGl2RenderingContext::BLEND);
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

    fn transparency(&self) -> Transparency {
        Transparency::Opaque
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
            ShaderSource::Vertex(include_str!("./picking.vert")),
            ShaderSource::Fragment(include_str!("./picking.frag")),
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
