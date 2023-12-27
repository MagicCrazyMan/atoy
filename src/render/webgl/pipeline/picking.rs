use std::any::Any;

use gl_matrix4rust::vec3::Vec3;
use log::warn;
use web_sys::{js_sys::Uint32Array, WebGl2RenderingContext};

use crate::{
    entity::{BorrowedMut, Strong, Weak},
    material::{Material, Transparency},
    render::{
        pp::{Executor, ItemKey, Pipeline, ResourceKey, Resources, State, Stuff},
        webgl::{
            attribute::{bind_attributes, unbind_attributes, AttributeBinding, AttributeValue},
            draw::draw,
            error::Error,
            offscreen::{
                FramebufferAttachment, FramebufferDrawBuffer, FramebufferTarget, OffscreenFramebuffer,
                OffscreenRenderbufferProvider, OffscreenTextureProvider,
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
};

use super::collector::StandardEntitiesCollector;

pub fn create_picking_pipeline(
    in_window_position: ResourceKey<(i32, i32)>,
    out_picked_entity: ResourceKey<Weak>,
    out_picked_position: ResourceKey<Vec3>,
) -> Pipeline<Error> {
    let collector = ItemKey::from_uuid();
    let picking = ItemKey::from_uuid();

    let collected_entities = ResourceKey::new_runtime_uuid();

    let mut pipeline = Pipeline::new();

    pipeline.add_executor(
        collector.clone(),
        StandardEntitiesCollector::new(collected_entities.clone()),
    );
    pipeline.add_executor(
        picking.clone(),
        Picking::new(
            collected_entities,
            in_window_position,
            out_picked_entity,
            out_picked_position,
        ),
    );

    pipeline.connect(&collector, &picking).unwrap();

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
    in_entities: ResourceKey<Vec<Strong>>,
    in_window_position: ResourceKey<(i32, i32)>,
    out_picked_entity: ResourceKey<Weak>,
    out_picked_position: ResourceKey<Vec3>,
    pixel: Uint32Array,
    frame: OffscreenFramebuffer,
    material: PickingMaterial,
}

impl Picking {
    pub fn new(
        in_entities: ResourceKey<Vec<Strong>>,
        in_window_position: ResourceKey<(i32, i32)>,
        out_picked_entity: ResourceKey<Weak>,
        out_picked_position: ResourceKey<Vec3>,
    ) -> Self {
        Self {
            in_entities,
            in_window_position,
            out_picked_entity,
            out_picked_position,
            pixel: Uint32Array::new_with_length(4),
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
        _: &mut dyn Stuff,
        resources: &mut Resources,
    ) -> Result<bool, Self::Error> {
        resources.remove_unchecked(&self.out_picked_entity);
        resources.remove_unchecked(&self.out_picked_position);

        if !resources.contains_key_unchecked(&self.in_window_position) {
            return Ok(false);
        }
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
        _: &mut dyn Stuff,
        _: &mut Resources,
    ) -> Result<(), Self::Error> {
        self.frame.unbind(state.gl());
        Ok(())
    }

    fn execute(
        &mut self,
        state: &mut State,
        stuff: &mut dyn Stuff,
        resources: &mut Resources,
    ) -> Result<(), Self::Error> {
        let Some((x, y)) = resources.get(&self.in_window_position) else {
            return Ok(());
        };
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

        let (x, y) = (*x, *y);

        // prepare material
        let program_item = state.program_store_mut().use_program(&self.material)?;
        state.gl().use_program(Some(program_item.gl_program()));

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

            let bound_attributes =
                bind_attributes(state, &entity, geometry, &self.material, &program_item);
            let bound_uniforms = bind_uniforms(
                state,
                stuff,
                &entity,
                geometry,
                &self.material,
                &program_item,
            );
            draw(state, geometry, &self.material);
            unbind_attributes(state, bound_attributes);
            unbind_uniforms(state, bound_uniforms);
        }

        // gets picking entity
        self.frame.set_read_buffer(state.gl(), FramebufferDrawBuffer::COLOR_ATTACHMENT0);
        state
            .gl()
            .read_pixels_with_opt_array_buffer_view(
                x,
                state.canvas().height() as i32 - y,
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
                resources.insert(self.out_picked_entity.clone(), entity.clone());
            }
        }

        // gets picking position
        self.frame.set_read_buffer(state.gl(), FramebufferDrawBuffer::COLOR_ATTACHMENT1);
        state
            .gl()
            .read_pixels_with_opt_array_buffer_view(
                x,
                state.canvas().height() as i32 - y,
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
            resources.insert(
                self.out_picked_position.clone(),
                Vec3::from_values(position[0] as f64, position[1] as f64, position[2] as f64),
            );
        }

        Ok(())
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

    fn attribute_value(&self, _: &str, _: &BorrowedMut) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, name: &str, _: &BorrowedMut) -> Option<UniformValue> {
        match name {
            "u_Index" => Some(UniformValue::UnsignedInteger1(self.index)),
            _ => None,
        }
    }

    fn uniform_block_value(&self, _: &str, _: &BorrowedMut) -> Option<UniformBlockValue> {
        None
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
