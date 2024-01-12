use std::{any::Any, borrow::Cow, ptr::NonNull};

use gl_matrix4rust::{mat4::AsMat4, vec3::Vec3};
use log::warn;
use wasm_bindgen::JsCast;
use web_sys::{js_sys::Uint32Array, HtmlCanvasElement, WebGl2RenderingContext};

use crate::{
    entity::Entity,
    material::Transparency,
    render::{
        webgl::{
            attribute::AttributeBinding,
            error::Error,
            framebuffer::{
                Framebuffer, FramebufferAttachment, FramebufferDrawBuffer, FramebufferTarget,
                RenderbufferProvider, TextureProvider,
            },
            program::{ProgramSource, ShaderSource},
            renderbuffer::RenderbufferInternalFormat,
            state::FrameState,
            texture::{TextureDataType, TextureFormat, TextureInternalFormat},
            uniform::{UniformBinding, UniformBlockBinding, UniformStructuralBinding},
        },
        Executor, GraphPipeline, ItemKey, Pipeline, ResourceKey, Resources,
    },
    scene::Scene,
};

use super::collector::StandardEntitiesCollector;

/// A [`Pipeline`] for picking purpose.
pub struct PickingPipeline {
    pipeline: GraphPipeline<FrameState, Error>,
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
            StandardEntitiesCollector::new(entities.clone(), None, None),
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
        let Some(picking_framebuffer) = picking.picking_framebuffer.as_mut() else {
            return Ok(None);
        };

        let Some(canvas) = gl
            .canvas()
            .and_then(|canvas| canvas.dyn_into::<HtmlCanvasElement>().ok())
        else {
            return Ok(None);
        };

        let pixel = Uint32Array::new_with_length(1);
        picking_framebuffer.bind(FramebufferTarget::FRAMEBUFFER)?;
        picking_framebuffer.read_pixels_with_read_buffer(
            FramebufferDrawBuffer::COLOR_ATTACHMENT0,
            window_position_x,
            canvas.height() as i32 - window_position_y,
            1,
            1,
            WebGl2RenderingContext::RED_INTEGER,
            WebGl2RenderingContext::UNSIGNED_INT,
            &pixel,
            0,
        )?;
        picking_framebuffer.unbind();

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
        let Some(picking_framebuffer) = picking.picking_framebuffer.as_mut() else {
            return Ok(None);
        };

        let Some(canvas) = gl
            .canvas()
            .and_then(|canvas| canvas.dyn_into::<HtmlCanvasElement>().ok())
        else {
            return Ok(None);
        };

        let pixel = Uint32Array::new_with_length(1);
        picking_framebuffer.bind(FramebufferTarget::FRAMEBUFFER)?;
        picking_framebuffer.read_pixels_with_read_buffer(
            FramebufferDrawBuffer::COLOR_ATTACHMENT0,
            window_position_x,
            canvas.height() as i32 - window_position_y,
            1,
            1,
            WebGl2RenderingContext::RGBA32UI,
            WebGl2RenderingContext::UNSIGNED_INT,
            &pixel,
            0,
        )?;
        picking_framebuffer.unbind();

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
    type State = FrameState;

    type Error = Error;

    fn execute(&mut self, state: &mut Self::State, scene: &mut Scene) -> Result<(), Self::Error> {
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
    entities_key: ResourceKey<Vec<NonNull<Entity>>>,
    picking_framebuffer: Option<Framebuffer>,
}

impl Picking {
    pub fn new(entities_key: ResourceKey<Vec<NonNull<Entity>>>) -> Self {
        Self {
            entities_key,
            picking_framebuffer: None,
        }
    }

    fn picking_framebuffer(&mut self, state: &FrameState) -> &mut Framebuffer {
        self.picking_framebuffer.get_or_insert_with(|| {
            state.create_framebuffer(
                [
                    TextureProvider::new(
                        FramebufferAttachment::COLOR_ATTACHMENT0,
                        TextureInternalFormat::R32UI,
                        TextureFormat::RED_INTEGER,
                        TextureDataType::UNSIGNED_INT,
                        0,
                    ),
                    TextureProvider::new(
                        FramebufferAttachment::COLOR_ATTACHMENT1,
                        TextureInternalFormat::RGBA32UI,
                        TextureFormat::RGBA_INTEGER,
                        TextureDataType::UNSIGNED_INT,
                        0,
                    ),
                ],
                [RenderbufferProvider::new(
                    FramebufferAttachment::DEPTH_ATTACHMENT,
                    RenderbufferInternalFormat::DEPTH_COMPONENT24,
                )],
                [
                    FramebufferDrawBuffer::COLOR_ATTACHMENT0,
                    FramebufferDrawBuffer::COLOR_ATTACHMENT1,
                ],
                None,
            )
        })
    }
}

impl Executor for Picking {
    type State = FrameState;

    type Error = Error;

    fn before(
        &mut self,
        state: &mut Self::State,
        _: &mut Scene,
        resources: &mut Resources,
    ) -> Result<bool, Self::Error> {
        if !resources.contains_resource_unchecked(&self.entities_key) {
            return Ok(false);
        }

        let picking_framebuffer = self.picking_framebuffer(&state);
        picking_framebuffer.bind(FramebufferTarget::FRAMEBUFFER)?;
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
        state: &mut Self::State,
        _: &mut Scene,
        _: &mut Resources,
    ) -> Result<(), Self::Error> {
        self.picking_framebuffer(&state).unbind();
        state.gl().disable(WebGl2RenderingContext::DEPTH_TEST);
        Ok(())
    }

    fn execute(
        &mut self,
        state: &mut Self::State,
        _: &mut Scene,
        resources: &mut Resources,
    ) -> Result<(), Self::Error> {
        let Some(entities) = resources.get(&self.entities_key) else {
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
        let program = state.program_store_mut().use_program(&PickingMaterial)?;
        let position_location = program
            .attribute_locations()
            .get(POSITIONS)
            .cloned()
            .unwrap();
        let index_location = program.uniform_locations().get(INDEX);
        let model_matrix_location = program.uniform_locations().get(MODEL_MATRIX);
        let view_proj_matrix_location = program.uniform_locations().get(VIEW_PROJ_MATRIX);

        let view_proj_matrix = state.camera().view_proj_matrix();
        state.gl().uniform_matrix4fv_with_f32_array(
            view_proj_matrix_location,
            false,
            &view_proj_matrix.to_gl(),
        );

        // render each entity by material
        for (index, entity) in entities.iter().enumerate() {
            let entity = unsafe { entity.as_ref() };

            let Some(geometry) = entity.geometry() else {
                continue;
            };
            let Some(vertices) = geometry.vertices() else {
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

            state.gl().uniform_matrix4fv_with_f32_array(
                model_matrix_location,
                false,
                &entity.compose_model_matrix().to_gl(),
            );
            state.gl().uniform1ui(index_location, (index + 1) as u32);
            let bound_attributes = state.bind_attribute(vertices, position_location)?;
            state.draw(&geometry.draw())?;
            state.unbind_attributes(bound_attributes);
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

static POSITIONS: &'static str = "a_Position";
static INDEX: &'static str = "u_Index";
static MODEL_MATRIX: &'static str = "u_ModelMatrix";
static VIEW_PROJ_MATRIX: &'static str = "u_ViewProjMatrix";

struct PickingMaterial;

impl ProgramSource for PickingMaterial {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("PickingMaterial")
    }

    fn sources(&self) -> Vec<ShaderSource> {
        vec![
            ShaderSource::VertexRaw(Cow::Borrowed(include_str!("./shaders/picking.vert"))),
            ShaderSource::FragmentRaw(Cow::Borrowed(include_str!("./shaders/picking.frag"))),
        ]
    }

    fn attribute_bindings(&self) -> Vec<AttributeBinding> {
        vec![AttributeBinding::Manual(Cow::Borrowed(POSITIONS))]
    }

    fn uniform_bindings(&self) -> Vec<UniformBinding> {
        vec![
            UniformBinding::Manual(Cow::Borrowed(MODEL_MATRIX)),
            UniformBinding::Manual(Cow::Borrowed(VIEW_PROJ_MATRIX)),
            UniformBinding::Manual(Cow::Borrowed(INDEX)),
        ]
    }

    fn uniform_structural_bindings(&self) -> Vec<UniformStructuralBinding> {
        vec![]
    }

    fn uniform_block_bindings(&self) -> Vec<UniformBlockBinding> {
        vec![]
    }
}
