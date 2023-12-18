use std::any::Any;

use gl_matrix4rust::vec2::Vec2;
use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::{
    entity::{BorrowedMut, Weak},
    geometry::rectangle::{Placement, Rectangle},
    material::{Material, Transparency},
    render::{
        pp::{Executor, ResourceKey, Resources, State, Stuff},
        webgl::{
            attribute::{bind_attributes, AttributeBinding, AttributeValue},
            draw::draw,
            error::Error,
            offscreen::{
                FramebufferAttachment, FramebufferTarget, OffscreenFrame,
                OffscreenFramebufferProvider, OffscreenTextureProvider,
            },
            program::{ProgramSource, ShaderSource},
            texture::{TextureDataType, TextureFormat, TextureInternalFormat},
            uniform::{bind_uniforms, UniformBinding, UniformBlockBinding, UniformValue},
        },
    },
};

pub struct Outlining {
    in_entity: ResourceKey<Weak>,
    out_texture: ResourceKey<WebGlTexture>,
    material: OutliningMaterial,
    geometry: Rectangle,
    onepass_frame: OffscreenFrame,
    twopass_frame: OffscreenFrame,
}

impl Outlining {
    pub fn new(in_entity: ResourceKey<Weak>, out_texture: ResourceKey<WebGlTexture>) -> Self {
        Self {
            in_entity,
            out_texture,
            material: OutliningMaterial {
                stage: 0,
                outline_color: [1.0, 0.0, 0.0, 1.0],
                outline_width: 5,
            },
            geometry: Rectangle::new(Vec2::from_values(-1.0, 1.0), Placement::TopLeft, 2.0, 2.0),
            onepass_frame: OffscreenFrame::new(
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
                [],
                [],
            ),
            twopass_frame: OffscreenFrame::new(
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
                [],
                [],
            ),
        }
    }
}

impl Executor for Outlining {
    type Error = Error;

    fn before(
        &mut self,
        state: &mut State,
        _: &mut dyn Stuff,
        _: &mut Resources,
    ) -> Result<bool, Self::Error> {
        self.onepass_frame.bind(state.gl())?;
        state.gl().clear_bufferfv_with_f32_array(
            WebGl2RenderingContext::COLOR,
            0,
            &[0.0, 0.0, 0.0, 0.0],
        );

        self.twopass_frame.bind(state.gl())?;
        state.gl().clear_bufferfv_with_f32_array(
            WebGl2RenderingContext::COLOR,
            0,
            &[0.0, 0.0, 0.0, 0.0],
        );

        Ok(true)
    }

    fn execute(
        &mut self,
        state: &mut State,
        stuff: &mut dyn Stuff,
        resources: &mut Resources,
    ) -> Result<(), Self::Error> {
        let Some(in_entity) = resources.get(&self.in_entity).and_then(|e| e.upgrade()) else {
            return Ok(());
        };
        let in_entity = in_entity.borrow_mut();
        let Some(geometry) = in_entity.geometry() else {
            return Ok(());
        };

        let program_item = state.program_store_mut().use_program(&self.material)?;
        state.gl().use_program(Some(program_item.gl_program()));

        // stage zero, draws entity with outline color to frame
        self.onepass_frame.bind(state.gl())?;
        self.material.stage = 0;
        let _ = bind_attributes(state, &in_entity, geometry, &self.material, &program_item);
        bind_uniforms(
            state,
            stuff,
            &in_entity,
            geometry,
            &self.material,
            &program_item,
        );
        draw(state, geometry, &self.material);
        self.onepass_frame.unbind(state.gl());

        // stage one, applies convolution kernel to texture to draw outline
        self.twopass_frame.bind(state.gl())?;
        self.material.stage = 1;
        state.gl().active_texture(WebGl2RenderingContext::TEXTURE0);
        state.gl().bind_texture(
            WebGl2RenderingContext::TEXTURE_2D,
            Some(
                &self
                    .onepass_frame
                    .textures()
                    .unwrap()
                    .get(0)
                    .as_ref()
                    .unwrap()
                    .0,
            ),
        );
        state.gl().tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );
        state.gl().tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );
        state.gl().tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_S,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        state.gl().tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_T,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        let _ = bind_attributes(
            state,
            &in_entity,
            &self.geometry,
            &self.material,
            &program_item,
        );
        bind_uniforms(
            state,
            stuff,
            &in_entity,
            &self.geometry,
            &self.material,
            &program_item,
        );
        draw(state, &self.geometry, &self.material);

        // stage two, clear color drawn in stage one
        self.material.stage = 2;
        let _ = bind_attributes(state, &in_entity, geometry, &self.material, &program_item);
        bind_uniforms(
            state,
            stuff,
            &in_entity,
            geometry,
            &self.material,
            &program_item,
        );
        draw(state, geometry, &self.material);

        resources.insert(
            self.out_texture.clone(),
            self.twopass_frame
                .textures()
                .unwrap()
                .get(0)
                .as_ref()
                .unwrap()
                .0
                .clone(),
        );

        Ok(())
    }
}

struct OutliningMaterial {
    stage: i32,
    outline_color: [f32; 4],
    outline_width: i32,
}

impl ProgramSource for OutliningMaterial {
    fn name(&self) -> &'static str {
        "OutliningMaterial"
    }

    fn sources<'a>(&'a self) -> &[ShaderSource<'a>] {
        &[
            ShaderSource::Vertex(include_str!("./shaders/outlining.vert")),
            ShaderSource::Fragment(include_str!("./shaders/outlining.frag")),
        ]
    }

    fn attribute_bindings(&self) -> &[AttributeBinding] {
        &[
            AttributeBinding::GeometryPosition,
            AttributeBinding::GeometryTextureCoordinate,
        ]
    }

    fn uniform_bindings(&self) -> &[UniformBinding] {
        &[
            UniformBinding::ModelMatrix,
            UniformBinding::ViewProjMatrix,
            UniformBinding::FromMaterial("u_StageVertex"),
            UniformBinding::FromMaterial("u_StageFrag"),
            UniformBinding::FromMaterial("u_OutlineColor"),
            UniformBinding::FromMaterial("u_OutlineWidth"),
            UniformBinding::FromMaterial("u_OutlineSampler"),
        ]
    }

    fn uniform_block_bindings(&self) -> &[UniformBlockBinding] {
        &[]
    }
}

impl Material for OutliningMaterial {
    fn transparency(&self) -> Transparency {
        Transparency::Opaque
    }

    fn attribute_value(&self, _: &str, _: &BorrowedMut) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, name: &str, _: &BorrowedMut) -> Option<UniformValue> {
        match name {
            "u_StageVertex" => Some(UniformValue::Integer1(self.stage)),
            "u_StageFrag" => Some(UniformValue::Integer1(self.stage)),
            "u_OutlineColor" => Some(UniformValue::FloatVector4(self.outline_color)),
            "u_OutlineWidth" => Some(UniformValue::Integer1(self.outline_width)),
            "u_OutlineSampler" => Some(UniformValue::Integer1(0)),
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
