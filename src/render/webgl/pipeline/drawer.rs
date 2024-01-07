use std::{any::Any, ptr::NonNull};

use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::{
    entity::Entity,
    geometry::Geometry,
    material::{Material, Transparency},
    render::{
        pp::{Executor, ResourceKey, Resources, State},
        webgl::{
            attribute::{bind_attributes, unbind_attributes},
            draw::draw,
            error::Error,
            offscreen::{
                FramebufferAttachment, FramebufferTarget, OffscreenFramebuffer,
                OffscreenRenderbufferProvider, OffscreenTextureProvider,
            },
            program::ProgramItem,
            renderbuffer::RenderbufferInternalFormat,
            texture::{TextureDataType, TextureFormat, TextureInternalFormat},
            uniform::{bind_uniforms, unbind_uniforms},
        },
    },
    scene::Scene,
};

/// Standard drawer, draws all entities with its own material and geometry.
///
/// # Get Resources & Data Type
/// - `entities`: [`Vec<NonNull<Entity>>`], a list contains entities to draw.
///
/// # Provides Resources & Data Type
/// - `texture`: [`ResourceKey<WebGlTexture>`], a resource key telling where to get the draw texture.
pub struct StandardDrawer {
    frame: OffscreenFramebuffer,
    in_entities: ResourceKey<Vec<NonNull<Entity>>>,
    out_texture: ResourceKey<WebGlTexture>,
    last_program: Option<ProgramItem>,
}

impl StandardDrawer {
    pub fn new(
        in_entities: ResourceKey<Vec<NonNull<Entity>>>,
        out_texture: ResourceKey<WebGlTexture>,
    ) -> Self {
        Self {
            in_entities,
            frame: OffscreenFramebuffer::new(
                FramebufferTarget::FRAMEBUFFER,
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
                    FramebufferAttachment::DEPTH_STENCIL_ATTACHMENT,
                    RenderbufferInternalFormat::DEPTH32F_STENCIL8,
                )],
            ),
            out_texture,
            last_program: None,
        }
    }

    fn draw(
        &mut self,
        state: &mut State,
        entity: &Entity,
        geometry: &dyn Geometry,
        material: &dyn Material,
    ) -> Result<(), Error> {
        // compile and bind program only when last program isn't equals the material
        if self
            .last_program
            .as_ref()
            .map(|last_program| last_program.name() != material.name())
            .unwrap_or(true)
        {
            let program_item = state.program_store_mut().use_program(&*material)?;
            state.gl().use_program(Some(program_item.gl_program()));
            self.last_program = Some(program_item.clone());
        }

        let program_item = self.last_program.as_ref().unwrap();

        // binds attributes
        let bound_attributes = bind_attributes(state, &entity, geometry, material, program_item);
        // binds uniforms
        let bound_uniforms = bind_uniforms(state, &entity, geometry, material, program_item);

        // draws
        draw(state, geometry, material);

        unbind_attributes(state, bound_attributes);
        unbind_uniforms(state, bound_uniforms);

        Ok(())
    }
}

impl Executor for StandardDrawer {
    type Error = Error;

    fn before(
        &mut self,
        state: &mut State,
        _: &mut Scene,
        resources: &mut Resources,
    ) -> Result<bool, Self::Error> {
        if !resources.contains_key(&self.in_entities) {
            return Ok(false);
        }

        // self.frame.bind(state.gl())?;
        state.gl().enable(WebGl2RenderingContext::DEPTH_TEST);
        state.gl().clear_depth(1.0);
        state.gl().clear(
            WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT,
        );

        Ok(true)
    }

    fn after(
        &mut self,
        state: &mut State,
        _: &mut Scene,
        _: &mut Resources,
    ) -> Result<(), Self::Error> {
        // self.frame.unbind(state.gl());
        Ok(())
    }

    fn execute(
        &mut self,
        state: &mut State,
        _: &mut Scene,
        resources: &mut Resources,
    ) -> Result<(), Self::Error> {
        let Some(entities) = resources.get_mut(&self.in_entities) else {
            return Ok(());
        };

        // splits opaques and translucents
        let mut opaques = Vec::new();
        let mut translucents = Vec::new();
        entities.iter_mut().for_each(|entity| unsafe {
            // prepares material and geometry
            if let Some(material) = entity.as_mut().material_mut() {
                material.prepare(state, entity.as_ref());
            };

            let entity = entity.as_ref();
            if let (Some(geometry), Some(material)) = (entity.geometry(), entity.material()) {
                // filters unready material
                if !material.ready() {
                    return;
                }

                // filters transparent material
                if material.transparency() == Transparency::Transparent {
                    return;
                }

                if material.transparency() == Transparency::Opaque {
                    opaques.push((entity, geometry, material));
                } else {
                    translucents.push((entity, geometry, material));
                }
            }
        });

        // draws opaque enable DEPTH_TEST and disable BLEND and draws them from nearest to farthest first
        state.gl().depth_mask(true);
        for (entity, geometry, material) in opaques {
            self.draw(state, entity, geometry, material)?;
        }

        // then draws translucents first with DEPTH_TEST unchangeable and enable BLEND and draws theme from farthest to nearest
        state.gl().enable(WebGl2RenderingContext::BLEND);
        state.gl().blend_equation(WebGl2RenderingContext::FUNC_ADD);
        state.gl().blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );
        state.gl().depth_mask(false);
        for (entity, geometry, material) in translucents.into_iter().rev() {
            self.draw(state, entity, geometry, material)?;
        }

        self.last_program = None;

        // resources.insert(
        //     self.out_texture.clone(),
        //     self.frame.textures().unwrap().get(0).unwrap().0.clone(),
        // );

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
