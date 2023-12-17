pub mod collector;

use std::collections::VecDeque;

use gl_matrix4rust::vec3::AsVec3;
use log::warn;
use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::{
    bounding::Culling,
    entity::{BorrowedMut, Strong},
    geometry::Geometry,
    material::{Material, Transparency},
    render::{
        pp::ItemKey,
        webgl::{
            attribute::{bind_attributes, unbind_attributes},
            draw::draw,
            offscreen::{
                FramebufferAttachment, FramebufferTarget, OffscreenFrame,
                OffscreenFramebufferProvider, OffscreenRenderbufferProvider,
                OffscreenTextureProvider,
            },
            program::ProgramItem,
            renderbuffer::RenderbufferInternalFormat,
            texture::{TextureDataType, TextureFormat, TextureInternalFormat},
            uniform::bind_uniforms,
        },
    },
};

use self::collector::StandardEntitiesCollector;

use super::{
    outlining::Outlining, picking::Picking, Executor, Pipeline, ResourceKey, Resources, State,
    Stuff, error::Error,
};

pub fn create_standard_pipeline(window_position: ResourceKey<(i32, i32)>) -> Pipeline {
    let collector = ItemKey::from_uuid();
    // let picking = ItemKey::from_uuid();
    // let outlining = ItemKey::from_uuid();
    let drawer = ItemKey::from_uuid();

    let collected_entities = ResourceKey::runtime_uuid();
    // let picked_entity = ResourceKey::runtime_uuid();
    // let picked_position = ResourceKey::runtime_uuid();

    let mut pipeline = Pipeline::new();
    pipeline.add_executor(
        collector.clone(),
        StandardEntitiesCollector::new(collected_entities.clone()),
    );
    // pipeline.add_executor(
    //     picking.clone(),
    //     Picking::new(
    //         window_position,
    //         collected_entities.clone(),
    //         picked_entity.clone(),
    //         picked_position.clone(),
    //     ),
    // );
    // pipeline.add_executor(outlining.clone(), Outlining::new(picked_entity));
    pipeline.add_executor(drawer.clone(), StandardDrawer::new(collected_entities));

    // safely unwraps
    // pipeline.connect(&collector, &picking).unwrap();
    pipeline.connect(&collector, &drawer).unwrap();
    // pipeline.connect(&picking, &outlining).unwrap();
    // pipeline.connect(&outlining, &drawer).unwrap();

    pipeline
}

/// Standard drawer, draws all entities with its own material and geometry.
///
/// # Get Resources & Data Type
/// - `entities`: [`Vec<Strong>`], a list contains entities to draw.
pub struct StandardDrawer {
    frame: OffscreenFrame,
    entities: ResourceKey<Vec<Strong>>,
    last_program: Option<ProgramItem>,
}

impl StandardDrawer {
    pub fn new(entities: ResourceKey<Vec<Strong>>) -> Self {
        Self {
            entities,
            frame: OffscreenFrame::new(
                [OffscreenFramebufferProvider::new(
                    FramebufferTarget::FRAMEBUFFER,
                )],
                [OffscreenTextureProvider::new(
                    FramebufferTarget::FRAMEBUFFER,
                    FramebufferAttachment::COLOR_ATTACHMENT0,
                    TextureInternalFormat::RGBA,
                    TextureFormat::RGBA,
                    TextureDataType::UNSIGNED_BYTE,
                    0,
                )],
                [OffscreenRenderbufferProvider::new(
                    FramebufferTarget::FRAMEBUFFER,
                    FramebufferAttachment::COLOR_ATTACHMENT0,
                    RenderbufferInternalFormat::DEPTH_STENCIL,
                )],
            ),
            last_program: None,
        }
    }

    fn draw(
        &mut self,
        state: &mut State,
        stuff: &dyn Stuff,
        entity: BorrowedMut,
        geometry: *mut dyn Geometry,
        material: *mut dyn Material,
    ) -> Result<(), Error> {
        unsafe {
            // compile and bind program only when last program isn't equals the material
            if self
                .last_program
                .as_ref()
                .map(|last_program| last_program.name() != (*material).name())
                .unwrap_or(true)
            {
                let item = state.program_store.use_program(&*material)?;
                state.gl.use_program(Some(item.gl_program()));
                self.last_program = Some(item.clone());
            }

            let program = self.last_program.as_ref().unwrap();

            // binds attributes
            let items = bind_attributes(state, &entity, &*geometry, &*material, program);
            // binds uniforms
            bind_uniforms(state, stuff, &entity, &*geometry, &*material, program);

            // before draw of material and geometry
            (&mut *material).before_draw(state, &entity);
            (&mut *geometry).before_draw(state, &entity);
            // draws
            draw(state, &*geometry, &*material);
            // after draw of material and geometry
            (&mut *material).after_draw(state, &entity);
            (&mut *geometry).after_draw(state, &entity);

            unbind_attributes(state, items);
        }

        Ok(())
    }
}

impl Executor for StandardDrawer {
    fn before(
        &mut self,
        state: &mut State,
        _: &mut dyn Stuff,
        resources: &mut Resources,
    ) -> Result<bool, Error> {
        if !resources.contains_key(&self.entities) {
            return Ok(false);
        }

        self.frame.bind(&state.gl)?;
        state.gl.viewport(
            0,
            0,
            state.canvas.width() as i32,
            state.canvas.height() as i32,
        );
        state.gl.enable(WebGl2RenderingContext::DEPTH_TEST);
        state.gl.enable(WebGl2RenderingContext::BLEND);
        state.gl.clear(
            WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT,
        );

        Ok(true)
    }

    fn after(
        &mut self,
        state: &mut State,
        _: &mut dyn Stuff,
        _: &mut Resources,
    ) -> Result<(), Error> {
        self.frame.bind(&state.gl)?;
        Ok(())
    }

    fn execute(
        &mut self,
        state: &mut State,
        stuff: &mut dyn Stuff,
        resources: &mut Resources,
    ) -> Result<(), Error> {
        let Some(entities) = resources.get(&self.entities) else {
            return Ok(());
        };

        // splits opaques and translucents
        let mut opaques = Vec::new();
        let mut translucents = Vec::new();
        let state_ptr: *const State = state;
        entities.iter().for_each(|entity| unsafe {
            let mut entity = entity.borrow_mut();

            // prepare material and geometry if exists
            if let Some(geometry) = entity.geometry_raw() {
                (*geometry).prepare(&*state_ptr, &entity);
            };
            if let Some(material) = entity.material_raw() {
                (*material).prepare(&*state_ptr, &entity);
            };

            if let (Some(geometry), Some(material)) = (entity.geometry_raw(), entity.material_raw())
            {
                // filters unready material
                if !(*material).ready() {
                    return;
                }

                // filters transparent material
                if (*material).transparency() == Transparency::Transparent {
                    return;
                }

                if (*material).transparency() == Transparency::Opaque {
                    opaques.push((entity, geometry, material));
                } else {
                    translucents.push((entity, geometry, material));
                }
            }
        });

        // draws opaque enable DEPTH_TEST and disable BLEND and draws them from nearest to farthest first
        state.gl.disable(WebGl2RenderingContext::BLEND);
        state.gl.depth_mask(true);
        for (entity, geometry, material) in opaques {
            self.draw(state, stuff, entity, geometry, material)?;
        }

        // then draws translucents first with DEPTH_TEST unchangeable and enable BLEND and draws theme from farthest to nearest
        state.gl.enable(WebGl2RenderingContext::BLEND);
        state.gl.blend_equation(WebGl2RenderingContext::FUNC_ADD);
        state.gl.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );
        state.gl.depth_mask(false);
        for (entity, geometry, material) in translucents.into_iter().rev() {
            self.draw(state, stuff, entity, geometry, material)?;
        }

        self.last_program = None;

        Ok(())
    }
}

/// Standard framebuffer composer.
pub struct StandardComposer {
    textures_keys: ResourceKey<Vec<ResourceKey<WebGlTexture>>>,
}

impl Executor for StandardComposer {
    fn execute(
        &mut self,
        state: &mut State,
        stuff: &mut dyn Stuff,
        resources: &mut Resources,
    ) -> Result<(), Error> {
    }
}
