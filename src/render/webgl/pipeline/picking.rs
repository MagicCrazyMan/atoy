use std::{any::Any, ptr::NonNull};

use gl_matrix4rust::vec3::Vec3;
use log::warn;
use wasm_bindgen::JsCast;
use web_sys::{js_sys::Uint32Array, HtmlCanvasElement, WebGl2RenderingContext};

use crate::{
    entity::Entity,
    event::EventAgency,
    material::{Material, Transparency},
    render::{
        pp::{Executor, GraphPipeline, ItemKey, Pipeline, ResourceKey, Resources, State},
        webgl::{
            attribute::{bind_attributes, unbind_attributes, AttributeBinding, AttributeValue},
            draw::draw,
            error::Error,
            framebuffer::{
                FramebufferAttachment, FramebufferDrawBuffer, FramebufferTarget,
                OffscreenFramebuffer, OffscreenRenderbufferProvider, OffscreenTextureProvider,
            },
            program::{ProgramSource, ShaderSource},
            renderbuffer::RenderbufferInternalFormat,
            texture::{TextureDataType, TextureFormat, TextureInternalFormat},
            uniform::{
                bind_uniforms, unbind_uniforms, UniformBinding, UniformBlockBinding,
                UniformBlockValue, UniformStructuralBinding, UniformValue,
            },
        },
    },
    scene::Scene,
};

use super::collector::StandardEntitiesCollector;

/// A [`Pipeline`] for picking purpose.
pub struct PickingPipeline {
    pipeline: GraphPipeline<Error>,
    picking: ItemKey,
    entities: ResourceKey<Vec<NonNull<Entity>>>,
    dirty: bool,
    gl: Option<WebGl2RenderingContext>,
}

impl PickingPipeline {
    /// Constructs a new picking pipeline.
    pub fn new() -> Self {
        let mut pipeline = GraphPipeline::new();

        let collector = ItemKey::new_uuid();
        let picking = ItemKey::new_uuid();

        let entities = ResourceKey::new_persist_uuid();

        pipeline.add_executor(
            collector.clone(),
            StandardEntitiesCollector::new(entities.clone()),
        );
        pipeline.add_executor(picking.clone(), Picking::new(entities.clone()));
        pipeline.connect(&collector, &picking).unwrap();

        Self {
            pipeline,
            picking,
            entities,
            dirty: true,
            gl: None,
        }
    }

    fn picking_executor(&mut self) -> &mut Picking {
        self.pipeline
            .executor_mut(&self.picking)
            .unwrap()
            .as_any_mut()
            .downcast_mut::<Picking>()
            .unwrap()
    }

    fn entities(&mut self) -> &mut Vec<NonNull<Entity>> {
        self.pipeline
            .resources_mut()
            .get_mut(&self.entities)
            .unwrap()
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_dirty(&mut self) {
        self.dirty = true;
    }

    /// Returns picked entity.
    pub fn pick_entity<'a, 'b>(
        &'a mut self,
        window_position_x: i32,
        window_position_y: i32,
    ) -> Result<Option<&'b mut Entity>, Error> {
        if self.dirty {
            return Ok(None);
        }
        let Some(gl) = self.gl.clone() else {
            return Ok(None);
        };

        let picking = self.picking_executor();
        picking.frame.bind(&gl)?;
        picking
            .frame
            .set_read_buffer(&gl, FramebufferDrawBuffer::COLOR_ATTACHMENT0);

        let canvas = gl
            .canvas()
            .and_then(|canvas| canvas.dyn_into::<HtmlCanvasElement>().ok())
            .ok_or(Error::CanvasNotFound)?;
        let pixel = Uint32Array::new_with_length(1);
        gl.read_pixels_with_opt_array_buffer_view(
            window_position_x,
            canvas.height() as i32 - window_position_y,
            1,
            1,
            WebGl2RenderingContext::RED_INTEGER,
            WebGl2RenderingContext::UNSIGNED_INT,
            Some(&pixel),
        )
        .or_else(|err| Err(Error::CommonWebGLError(err.as_string())))?;

        picking.frame.unbind(&gl);

        let index = pixel.get_index(0) as usize;
        if index > 0 {
            let entity = self
                .entities()
                .get_mut(index - 1)
                .map(|entity| unsafe { entity.as_mut() });
            if let Some(entity) = entity {
                return Ok(Some(entity));
            }
        }

        Ok(None)
    }

    /// Returns picked position.
    pub fn pick_position(
        &mut self,
        window_position_x: i32,
        window_position_y: i32,
    ) -> Result<Option<Vec3>, Error> {
        if self.dirty {
            return Ok(None);
        }
        let Some(gl) = self.gl.clone() else {
            return Ok(None);
        };

        let picking = self.picking_executor();
        picking.frame.bind(&gl)?;
        picking
            .frame
            .set_read_buffer(&gl, FramebufferDrawBuffer::COLOR_ATTACHMENT1);

        let canvas = gl
            .canvas()
            .and_then(|canvas| canvas.dyn_into::<HtmlCanvasElement>().ok())
            .ok_or(Error::CanvasNotFound)?;
        let pixel = Uint32Array::new_with_length(4);
        gl.read_pixels_with_opt_array_buffer_view(
            window_position_x,
            canvas.height() as i32 - window_position_y,
            1,
            1,
            WebGl2RenderingContext::RGBA_INTEGER,
            WebGl2RenderingContext::UNSIGNED_INT,
            Some(&pixel),
        )
        .or_else(|err| Err(Error::CommonWebGLError(err.as_string())))?;

        picking.frame.unbind(&gl);

        let position = [
            f32::from_ne_bytes(pixel.get_index(0).to_ne_bytes()),
            f32::from_ne_bytes(pixel.get_index(1).to_ne_bytes()),
            f32::from_ne_bytes(pixel.get_index(2).to_ne_bytes()),
            f32::from_ne_bytes(pixel.get_index(3).to_ne_bytes()),
        ]; // converts unsigned int back to float
        if position != [0.0, 0.0, 0.0, 0.0] {
            Ok(Some(Vec3::from_values(
                position[0] as f64,
                position[1] as f64,
                position[2] as f64,
            )))
        } else {
            Ok(None)
        }
    }
}

impl Pipeline for PickingPipeline {
    type Error = Error;

    fn execute(&mut self, state: &mut State, scene: &mut Scene) -> Result<(), Self::Error> {
        self.pipeline.execute(state, scene)?;
        self.dirty = false;
        self.gl = Some(state.gl().clone());
        Ok(())
    }
}

/// Picking detection.
///
/// # Get Resources & Data Type
/// - `entities`: [`Vec<NonNull<Entity>>`], a list contains entities to pick.
/// - `position`: `(i32, i32)`, a window coordinate position, skip picking if none.
///
/// # Provide Resources & Data Type
/// - `picked_entity`: [`Weak`](crate::entity::Weak), picked entity.
/// - `picked_position`: `[f32; 4]`, picked position. Picked position regards as `None` if components are all `0.0`.
pub struct Picking {
    in_entities: ResourceKey<Vec<NonNull<Entity>>>,
    frame: OffscreenFramebuffer,
    material: PickingMaterial,
}

impl Picking {
    pub fn new(in_entities: ResourceKey<Vec<NonNull<Entity>>>) -> Self {
        Self {
            in_entities,
            frame: OffscreenFramebuffer::with_draw_buffers(
                FramebufferTarget::FRAMEBUFFER,
                [
                    OffscreenTextureProvider::new(
                        FramebufferTarget::FRAMEBUFFER,
                        FramebufferAttachment::COLOR_ATTACHMENT0,
                        TextureInternalFormat::R32UI,
                        TextureFormat::RED_INTEGER,
                        TextureDataType::UNSIGNED_INT,
                        0,
                    ),
                    OffscreenTextureProvider::new(
                        FramebufferTarget::FRAMEBUFFER,
                        FramebufferAttachment::COLOR_ATTACHMENT1,
                        TextureInternalFormat::RGBA32UI,
                        TextureFormat::RGBA_INTEGER,
                        TextureDataType::UNSIGNED_INT,
                        0,
                    ),
                ],
                [OffscreenRenderbufferProvider::new(
                    FramebufferTarget::FRAMEBUFFER,
                    FramebufferAttachment::DEPTH_ATTACHMENT,
                    RenderbufferInternalFormat::DEPTH_COMPONENT24,
                )],
                [
                    FramebufferDrawBuffer::COLOR_ATTACHMENT0,
                    FramebufferDrawBuffer::COLOR_ATTACHMENT1,
                ],
            ),
            material: PickingMaterial { index: 0 },
        }
    }
}

impl Executor for Picking {
    type Error = Error;

    fn before(
        &mut self,
        state: &mut State,
        _: &mut Scene,
        resources: &mut Resources,
    ) -> Result<bool, Self::Error> {
        if !resources.contains_key_unchecked(&self.in_entities) {
            return Ok(false);
        }

        self.frame.bind(&state.gl())?;
        state.gl().enable(WebGl2RenderingContext::DEPTH_TEST);

        state
            .gl()
            .clear_bufferuiv_with_u32_array(WebGl2RenderingContext::COLOR, 0, &[0, 0, 0, 0]);
        state
            .gl()
            .clear_bufferuiv_with_u32_array(WebGl2RenderingContext::COLOR, 1, &[0, 0, 0, 0]);
        state
            .gl()
            .clear_bufferfv_with_f32_array(WebGl2RenderingContext::DEPTH, 0, &[1.0]);

        Ok(true)
    }

    fn after(
        &mut self,
        state: &mut State,
        _: &mut Scene,
        _: &mut Resources,
    ) -> Result<(), Self::Error> {
        self.frame.unbind(state.gl());
        Ok(())
    }

    fn execute(
        &mut self,
        state: &mut State,
        _: &mut Scene,
        resources: &mut Resources,
    ) -> Result<(), Self::Error> {
        let Some(entities) = resources.get(&self.in_entities) else {
            return Ok(());
        };

        if entities.len() - 1 > u32::MAX as usize {
            warn!(
                target: "Picking",
                "too many entities, skip picking."
            );
            return Ok(());
        }

        // prepare material
        let program_item = state.program_store_mut().use_program(&self.material)?;
        state.gl().use_program(Some(program_item.gl_program()));

        // render each entity by material
        for (index, entity) in entities.iter().enumerate() {
            let entity = unsafe { entity.as_ref() };
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

            let bound_attributes =
                bind_attributes(state, &entity, geometry, &self.material, &program_item);
            let bound_uniforms =
                bind_uniforms(state, &entity, geometry, &self.material, &program_item);
            draw(state, geometry, &self.material);
            unbind_attributes(state, bound_attributes);
            unbind_uniforms(state, bound_uniforms);
        }

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

struct PickingMaterial {
    index: u32,
}

impl ProgramSource for PickingMaterial {
    fn name(&self) -> &'static str {
        "PickingMaterial"
    }

    fn sources(&self) -> Vec<ShaderSource> {
        vec![
            ShaderSource::VertexRaw(include_str!("./shaders/picking.vert")),
            ShaderSource::FragmentRaw(include_str!("./shaders/picking.frag")),
        ]
    }

    fn attribute_bindings(&self) -> Vec<AttributeBinding> {
        vec![AttributeBinding::GeometryPosition]
    }

    fn uniform_bindings(&self) -> Vec<UniformBinding> {
        vec![
            UniformBinding::ModelMatrix,
            UniformBinding::ViewProjMatrix,
            UniformBinding::FromMaterial("u_Index"),
        ]
    }

    fn uniform_structural_bindings(&self) -> Vec<UniformStructuralBinding> {
        vec![]
    }

    fn uniform_block_bindings(&self) -> Vec<UniformBlockBinding> {
        vec![]
    }
}

impl Material for PickingMaterial {
    fn transparency(&self) -> Transparency {
        Transparency::Opaque
    }

    fn attribute_value(&self, _: &str, _: &Entity) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, name: &str, _: &Entity) -> Option<UniformValue> {
        match name {
            "u_Index" => Some(UniformValue::UnsignedInteger1(self.index)),
            _ => None,
        }
    }

    fn uniform_block_value(&self, _: &str, _: &Entity) -> Option<UniformBlockValue> {
        None
    }

    fn ready(&self) -> bool {
        true
    }

    fn instanced(&self) -> Option<i32> {
        None
    }

    fn changed_event(&self) -> &EventAgency<()> {
        unreachable!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn prepare(&mut self, _: &mut State, _: &Entity) {}
}
