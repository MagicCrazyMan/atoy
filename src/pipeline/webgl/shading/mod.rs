use std::{borrow::Cow, cell::RefCell, rc::Rc};

use web_sys::WebGl2RenderingContext;

use crate::{
    entity::Entity,
    material::webgl::StandardMaterial,
    renderer::webgl::{
        conversion::ToGlEnum,
        draw::Draw,
        error::Error,
        program::{Define, Program, ProgramSource},
        state::FrameState,
        uniform::{UniformBinding, UniformValue},
    },
    scene::{
        AREA_LIGHTS_COUNT_DEFINE, DIRECTIONAL_LIGHTS_COUNT_DEFINE, MAX_AREA_LIGHTS_STRING,
        MAX_DIRECTIONAL_LIGHTS_STRING, MAX_POINT_LIGHTS_STRING, MAX_SPOT_LIGHTS_STRING,
        POINT_LIGHTS_COUNT_DEFINE, SPOT_LIGHTS_COUNT_DEFINE,
    },
};

use super::{
    collector::CollectedEntities, UBO_LIGHTS_BLOCK_BINDING, UBO_LIGHTS_UNIFORM_BLOCK_MOUNT_POINT,
    UBO_UNIVERSAL_UNIFORMS_BLOCK_BINDING, UBO_UNIVERSAL_UNIFORM_BLOCK_MOUNT_POINT,
};

pub mod deferred;
pub mod forward;
pub mod picking;

const BLOOM_THRESHOLD_VALUES: [f32; 3] = [0.2126, 0.7152, 0.0722];
const BLOOM_THRESHOLD_UNIFORM_NAME: &'static str = "u_BloomThreshold";
const BLOOM_THRESHOLD_UNIFORM_BINDING: UniformBinding =
    UniformBinding::Custom(Cow::Borrowed(BLOOM_THRESHOLD_UNIFORM_NAME));

const BASE_TEXTURE_UNIFORM_NAME: &'static str = "u_BaseTexture";
const BASE_TEXTURE_UNIFORM_BINDING: UniformBinding =
    UniformBinding::Custom(Cow::Borrowed(BASE_TEXTURE_UNIFORM_NAME));

const BLOOM_BLUR_TEXTURE_UNIFORM_NAME: &'static str = "u_BloomBlurTexture";
const BLOOM_BLUR_TEXTURE_UNIFORM_BINDING: UniformBinding =
    UniformBinding::Custom(Cow::Borrowed(BLOOM_BLUR_TEXTURE_UNIFORM_NAME));

const HDR_TEXTURE_UNIFORM_NAME: &'static str = "u_HdrTexture";
const HDR_TEXTURE_UNIFORM_BINDING: UniformBinding =
    UniformBinding::Custom(Cow::Borrowed(HDR_TEXTURE_UNIFORM_NAME));

const HDR_EXPOSURE_UNIFORM_NAME: &'static str = "u_HdrExposure";
const HDR_EXPOSURE_UNIFORM_BINDING: UniformBinding =
    UniformBinding::Custom(Cow::Borrowed(HDR_EXPOSURE_UNIFORM_NAME));

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(self) enum DrawState {
    Draw { lighting: bool, bloom: bool },
    GBuffer,
}

pub(self) fn draw_entities(
    state: &mut FrameState,
    draw_state: DrawState,
    collected_entities: &CollectedEntities,
) -> Result<(), Error> {
    draw_opaque_entities(state, draw_state, collected_entities)?;
    draw_translucent_entities(state, draw_state, collected_entities)?;
    Ok(())
}

fn draw_opaque_entities(
    state: &mut FrameState,
    draw_state: DrawState,
    collected_entities: &CollectedEntities,
) -> Result<(), Error> {
    state.gl().enable(WebGl2RenderingContext::DEPTH_TEST);
    state.gl().depth_mask(true);

    // draws opaque enable DEPTH_TEST and disable BLEND and draws them from nearest to farthest first
    for entity in collected_entities.opaque_entities() {
        let Some(entity) = entity.upgrade() else {
            continue;
        };
        draw_entity(state, draw_state, true, entity)?;
    }

    state.gl().disable(WebGl2RenderingContext::CULL_FACE);
    state.gl().cull_face(WebGl2RenderingContext::BACK);
    state.gl().disable(WebGl2RenderingContext::DEPTH_TEST);

    Ok(())
}

fn draw_translucent_entities(
    state: &mut FrameState,
    draw_state: DrawState,
    collected_entities: &CollectedEntities,
) -> Result<(), Error> {
    state.gl().enable(WebGl2RenderingContext::DEPTH_TEST);
    state.gl().depth_mask(false);
    state.gl().enable(WebGl2RenderingContext::BLEND);
    state.gl().blend_equation(WebGl2RenderingContext::FUNC_ADD);
    state.gl().blend_func(
        WebGl2RenderingContext::ONE,
        WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
    );

    // draws translucents first with DEPTH_TEST unchangeable and enable BLEND and draws them from farthest to nearest
    for entity in collected_entities.translucent_entities().iter().rev() {
        // transparency entities never cull face
        let Some(entity) = entity.upgrade() else {
            continue;
        };
        draw_entity(state, draw_state, false, entity)?;
    }

    state.gl().depth_mask(true);
    state.gl().disable(WebGl2RenderingContext::DEPTH_TEST);
    state.gl().disable(WebGl2RenderingContext::CULL_FACE);
    state.gl().cull_face(WebGl2RenderingContext::BACK);
    state.gl().disable(WebGl2RenderingContext::BLEND);
    state
        .gl()
        .blend_func(WebGl2RenderingContext::ONE, WebGl2RenderingContext::ZERO);

    Ok(())
}

fn prepare_program<'a, 'b, 'c>(
    state: &'a mut FrameState,
    draw_state: DrawState,
    material: &'b dyn StandardMaterial,
) -> Result<Program, Error> {
    let source = StandardMaterialProgramSource::new(material, draw_state);
    let program = state
        .program_store_mut()
        .get_or_compile_program(&source)?;
    program.use_program()?;
    // binds atoy_Universal
    program.mount_uniform_block_by_binding(
        &UBO_UNIVERSAL_UNIFORMS_BLOCK_BINDING,
        UBO_UNIVERSAL_UNIFORM_BLOCK_MOUNT_POINT,
    )?;

    if let DrawState::Draw { lighting, bloom } = draw_state {
        // binds atoy_Lights
        if lighting {
            program.mount_uniform_block_by_binding(
                &UBO_LIGHTS_BLOCK_BINDING,
                UBO_LIGHTS_UNIFORM_BLOCK_MOUNT_POINT,
            )?;
        }

        // binds bloom blur threshold
        if bloom {
            program.bind_uniform_value_by_binding(
                &BLOOM_THRESHOLD_UNIFORM_BINDING,
                &UniformValue::FloatVector3(BLOOM_THRESHOLD_VALUES),
                None,
            )?;
        }
    }

    Ok(program)
}

fn draw_entity(
    state: &mut FrameState,
    draw_state: DrawState,
    should_cull_face: bool,
    entity: Rc<RefCell<dyn Entity>>,
) -> Result<(), Error> {
    // tries vertex array object entity
    let vao = match entity.borrow_mut().as_vertex_array_object_entity_mut() {
        Some(entity) => {
            let vao = entity.vertex_array_object();
            match vao {
                Some(vao) => Some((vao, false)),
                None => {
                    let vao = state
                        .gl()
                        .create_vertex_array()
                        .ok_or(Error::CreateVertexArrayObjectFailure)?;
                    entity.store_vertex_array_object(vao.clone());

                    Some((vao, true))
                }
            }
        }
        None => None,
    };

    let entity = entity.borrow_mut();
    let geometry = entity.geometry().unwrap();
    let material = entity.material().unwrap();

    // culls face
    if should_cull_face {
        if let Some(cull_face) = geometry.cull_face() {
            state.gl().enable(WebGl2RenderingContext::CULL_FACE);
            state.gl().cull_face(cull_face.gl_enum());
        } else {
            state.gl().disable(WebGl2RenderingContext::CULL_FACE);
        }
    } else {
        state.gl().disable(WebGl2RenderingContext::CULL_FACE);
    }

    let program = prepare_program(state, draw_state, material)?;
    match vao {
        Some((vao, is_new)) => {
            program.bind_vertex_array_object(vao)?;
            if is_new {
                program.bind_attributes(
                    Some(&state),
                    Some(&*entity),
                    Some(geometry),
                    Some(material),
                )?;
            }
        }
        None => {
            program.bind_attributes(
                Some(&state),
                Some(&*entity),
                Some(geometry),
                Some(material),
            )?;
        }
    };
    program.bind_uniforms(Some(&state), Some(&*entity), Some(geometry), Some(material))?;
    program.bind_uniform_blocks(Some(&state), Some(&*entity), Some(geometry), Some(material))?;
    Draw::from_geometry(geometry).draw(state.gl(), Some(state.buffer_store()))?;
    program.unuse_program()?;

    Ok(())
}

struct StandardMaterialProgramSource<'a> {
    material: &'a dyn StandardMaterial,
    draw_state: DrawState,
}

impl<'a> StandardMaterialProgramSource<'a> {
    fn new(material: &'a dyn StandardMaterial, draw_state: DrawState) -> Self {
        Self {
            material,
            draw_state,
        }
    }
}

impl<'a> ProgramSource for StandardMaterialProgramSource<'a> {
    fn name(&self) -> Cow<'_, str> {
        const DEFINE_NAME_VALUE_SEPARATOR: &'static str = "!!";
        const DEFINE_SEPARATOR: &'static str = "##";
        const DEFINES_SEPARATOR: &'static str = "@@";

        trait JoinDefines {
            fn join_defines(&self) -> String;
        }

        impl<'a> JoinDefines for &'a [Define<'a>] {
            fn join_defines(&self) -> String {
                self.iter()
                    .map(|define| match define {
                        Define::WithValue(name, value) => {
                            Cow::Owned(format!("{}{}{}", name, DEFINE_NAME_VALUE_SEPARATOR, value))
                        }
                        Define::WithoutValue(name) => Cow::Borrowed(name.as_ref()),
                    })
                    .collect::<Vec<_>>()
                    .join(DEFINE_SEPARATOR)
            }
        }

        let type_name = match self.draw_state {
            DrawState::Draw { .. } => "Draw",
            DrawState::GBuffer => "GBuffer",
        };
        let defines = self.universal_defines().as_ref().join_defines();
        let vertex_defines = self.vertex_defines().as_ref().join_defines();
        let fragment_defines = self.fragment_defines().as_ref().join_defines();

        if defines.len() + vertex_defines.len() + fragment_defines.len() == 0 {
            self.material.name()
        } else {
            Cow::Owned(format!(
                "{}{}{}{}{}{}{}{}{}",
                self.material.name().as_ref(),
                DEFINES_SEPARATOR,
                type_name,
                DEFINES_SEPARATOR,
                defines,
                DEFINES_SEPARATOR,
                vertex_defines,
                DEFINES_SEPARATOR,
                fragment_defines
            ))
        }
    }

    fn vertex_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("../shaders/standard.vert"))
    }

    fn fragment_source(&self) -> Cow<'_, str> {
        match self.draw_state {
            DrawState::Draw { .. } => Cow::Borrowed(include_str!("../shaders/draw.frag")),
            DrawState::GBuffer => Cow::Borrowed(include_str!("../shaders/gbuffer.frag")),
        }
    }

    fn universal_defines(&self) -> Cow<'_, [Define<'_>]> {
        let mut defines = Vec::with_capacity(12);

        if self.material.use_position_eye_space() {
            defines.push(Define::WithoutValue(Cow::Borrowed(
                "USE_POSITION_EYE_SPACE",
            )));
        }
        match (self.material.use_normal(), self.draw_state) {
            (true, _) | (false, DrawState::GBuffer) => {
                defines.push(Define::WithoutValue(Cow::Borrowed("USE_NORMAL")));
            }
            (false, DrawState::Draw { lighting, .. }) => {
                if lighting {
                    defines.push(Define::WithoutValue(Cow::Borrowed("USE_NORMAL")));
                }
            }
        };
        if self.material.use_texture_coordinate() {
            defines.push(Define::WithoutValue(Cow::Borrowed(
                "USE_TEXTURE_COORDINATE",
            )));
        }
        if self.material.use_tbn() {
            defines.push(Define::WithoutValue(Cow::Borrowed("USE_TBN")));
        }
        if self.material.use_tbn_invert() {
            defines.push(Define::WithoutValue(Cow::Borrowed("USE_TBN_INVERT")));
        }
        if self.material.use_calculated_bitangent() {
            defines.push(Define::WithoutValue(Cow::Borrowed(
                "USE_CALCULATED_BITANGENT",
            )));
        }

        if let DrawState::Draw { lighting, bloom } = self.draw_state {
            if lighting {
                defines.extend([
                    Define::WithoutValue(Cow::Borrowed("USE_LIGHTING")),
                    Define::WithValue(
                        Cow::Borrowed(DIRECTIONAL_LIGHTS_COUNT_DEFINE),
                        Cow::Borrowed(MAX_DIRECTIONAL_LIGHTS_STRING),
                    ),
                    Define::WithValue(
                        Cow::Borrowed(POINT_LIGHTS_COUNT_DEFINE),
                        Cow::Borrowed(MAX_POINT_LIGHTS_STRING),
                    ),
                    Define::WithValue(
                        Cow::Borrowed(SPOT_LIGHTS_COUNT_DEFINE),
                        Cow::Borrowed(MAX_SPOT_LIGHTS_STRING),
                    ),
                    Define::WithValue(
                        Cow::Borrowed(AREA_LIGHTS_COUNT_DEFINE),
                        Cow::Borrowed(MAX_AREA_LIGHTS_STRING),
                    ),
                ]);
            }

            if bloom {
                defines.push(Define::WithoutValue(Cow::Borrowed("USE_BLOOM")));
            }
        }

        Cow::Owned(defines)
    }

    fn vertex_defines(&self) -> Cow<'_, [Define<'_>]> {
        self.material.vertex_defines()
    }

    fn fragment_defines(&self) -> Cow<'_, [Define<'_>]> {
        self.material.fragment_defines()
    }

    fn snippet(&self, name: &str) -> Option<Cow<'_, str>> {
        match name {
            "FragmentProcess" => Some(self.material.fragment_process()),
            _ => self.material.snippet(name),
        }
    }
}

pub(self) struct HdrReinhardToneMapping;

impl ProgramSource for HdrReinhardToneMapping {
    fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed("HdrReinhardToneMapping")
    }

    fn vertex_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("../shaders/computation.vert"))
    }

    fn fragment_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("../shaders/hdr_reinhard_tone_mapping.frag"))
    }

    fn universal_defines(&self) -> Cow<'_, [Define<'_>]> {
        Cow::Borrowed(&[])
    }

    fn vertex_defines(&self) -> Cow<'_, [Define<'_>]> {
        Cow::Borrowed(&[])
    }

    fn fragment_defines(&self) -> Cow<'_, [Define<'_>]> {
        Cow::Borrowed(&[])
    }

    fn snippet(&self, _: &str) -> Option<Cow<'_, str>> {
        None
    }
}

pub(self) struct HdrExposureToneMapping;

impl ProgramSource for HdrExposureToneMapping {
    fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed("HdrExposureToneMapping")
    }

    fn vertex_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("../shaders/computation.vert"))
    }

    fn fragment_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("../shaders/hdr_exposure_tone_mapping.frag"))
    }

    fn universal_defines(&self) -> Cow<'_, [Define<'_>]> {
        Cow::Borrowed(&[])
    }

    fn vertex_defines(&self) -> Cow<'_, [Define<'_>]> {
        Cow::Borrowed(&[])
    }

    fn fragment_defines(&self) -> Cow<'_, [Define<'_>]> {
        Cow::Borrowed(&[])
    }

    fn snippet(&self, _: &str) -> Option<Cow<'_, str>> {
        None
    }
}

pub(self) struct BloomMapping;

impl ProgramSource for BloomMapping {
    fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed("BloomMapping")
    }

    fn vertex_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("../shaders/computation.vert"))
    }

    fn fragment_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("../shaders/bloom_mapping.frag"))
    }

    fn universal_defines(&self) -> Cow<'_, [Define<'_>]> {
        Cow::Borrowed(&[])
    }

    fn vertex_defines(&self) -> Cow<'_, [Define<'_>]> {
        Cow::Borrowed(&[])
    }

    fn fragment_defines(&self) -> Cow<'_, [Define<'_>]> {
        Cow::Borrowed(&[])
    }

    fn snippet(&self, _: &str) -> Option<Cow<'_, str>> {
        None
    }
}

struct GaussianBlurMapping;

impl ProgramSource for GaussianBlurMapping {
    fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed("GaussianBlurMapping")
    }

    fn vertex_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("../shaders/computation.vert"))
    }

    fn fragment_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("../shaders/gaussian_blur.frag"))
    }

    fn universal_defines(&self) -> Cow<'_, [Define<'_>]> {
        Cow::Borrowed(&[])
    }

    fn vertex_defines(&self) -> Cow<'_, [Define<'_>]> {
        Cow::Borrowed(&[])
    }

    fn fragment_defines(&self) -> Cow<'_, [Define<'_>]> {
        Cow::Borrowed(&[])
    }

    fn snippet(&self, _: &str) -> Option<Cow<'_, str>> {
        None
    }
}

struct BloomBlendMapping;

impl ProgramSource for BloomBlendMapping {
    fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed("BloomBlendMapping")
    }

    fn vertex_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("../shaders/computation.vert"))
    }

    fn fragment_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("../shaders/bloom_blend.frag"))
    }

    fn universal_defines(&self) -> Cow<'_, [Define<'_>]> {
        Cow::Borrowed(&[])
    }

    fn vertex_defines(&self) -> Cow<'_, [Define<'_>]> {
        Cow::Borrowed(&[])
    }

    fn fragment_defines(&self) -> Cow<'_, [Define<'_>]> {
        Cow::Borrowed(&[])
    }

    fn snippet(&self, _: &str) -> Option<Cow<'_, str>> {
        None
    }
}
