use std::{any::Any, collections::HashMap};

use web_sys::WebGl2RenderingContext;

use crate::{
    entity::{BorrowedMut, Weak},
    material::Material,
    render::webgl::{
        attribute::{AttributeBinding, AttributeValue},
        draw::{bind_attributes, bind_uniforms, draw},
        error::Error,
        program::ShaderSource,
        uniform::{UniformBinding, UniformValue},
    },
};

use super::{Executor, ResourceSource, State, Stuff};

pub struct Outlining {
    entity: ResourceSource,
    material: OutliningMaterial,
}

impl Outlining {
    pub fn new(entity: ResourceSource) -> Self {
        Self {
            entity,
            material: OutliningMaterial {
                outline_width: 15,
                outline_color: [1.0, 0.0, 0.0, 0.2],
                should_scale: true,
            },
        }
    }
}

impl Executor for Outlining {
    fn execute(
        &mut self,
        state: &mut State,
        stuff: &mut dyn Stuff,
        runtime_resources: &mut HashMap<String, Box<dyn std::any::Any>>,
        persist_resources: &mut HashMap<String, Box<dyn std::any::Any>>,
    ) -> Result<(), Error> {
        let entity = match &self.entity {
            ResourceSource::Runtime(key) => runtime_resources.get(key.as_str()),
            ResourceSource::Persist(key) => persist_resources.get(key.as_str()),
        };
        let Some(entity) = entity
            .and_then(|resource| resource.downcast_ref::<Weak>())
            .and_then(|e| e.upgrade())
        else {
            return Ok(());
        };

        let entity = entity.borrow_mut();
        let Some(geometry) = entity.geometry() else {
            return Ok(());
        };

        // setups webgl
        state.gl.enable(WebGl2RenderingContext::STENCIL_TEST);

        // prepares material
        let program = state.program_store.use_program(&self.material)?;
        state.gl.use_program(Some(program.gl_program()));

        // only have to binds attribute once
        bind_attributes(
            state,
            &entity,
            geometry,
            &self.material,
            program.attribute_locations(),
        );

        // one pass, enable stencil test, disable depth test, draw entity with scaling up, sets stencil values to 1
        {
            self.material.should_scale = true;

            state.gl.depth_mask(false);
            state.gl.stencil_mask(0xFF);
            state
                .gl
                .stencil_func(WebGl2RenderingContext::ALWAYS, 1, 0xff);
            state.gl.stencil_op(
                WebGl2RenderingContext::KEEP,
                WebGl2RenderingContext::REPLACE,
                WebGl2RenderingContext::REPLACE,
            );

            bind_uniforms(
                state,
                stuff,
                &entity,
                geometry,
                &self.material,
                program.uniform_locations(),
            );
            draw(state, geometry, &self.material);
        }

        // two pass, enable stencil test, disable depth test, draw entity with no scaling, sets stencil values to 0
        {
            self.material.should_scale = false;

            state.gl.depth_mask(false);
            state.gl.stencil_mask(0xFF);
            state
                .gl
                .stencil_func(WebGl2RenderingContext::ALWAYS, 0, 0xff);
            state.gl.stencil_op(
                WebGl2RenderingContext::KEEP,
                WebGl2RenderingContext::REPLACE,
                WebGl2RenderingContext::REPLACE,
            );

            bind_uniforms(
                state,
                stuff,
                &entity,
                geometry,
                &self.material,
                program.uniform_locations(),
            );
            draw(state, geometry, &self.material);
        }

        // three pass, disable stencil test, enable depth test, draw entity with scaling up, draws depth of where stencil value is 1
        {
            self.material.should_scale = true;

            state.gl.depth_mask(true);
            state.gl.stencil_mask(0);
            state
                .gl
                .stencil_func(WebGl2RenderingContext::EQUAL, 1, 0xff);
            state.gl.stencil_op(
                WebGl2RenderingContext::KEEP,
                WebGl2RenderingContext::KEEP,
                WebGl2RenderingContext::KEEP,
            );

            bind_uniforms(
                state,
                stuff,
                &entity,
                geometry,
                &self.material,
                program.uniform_locations(),
            );
            draw(state, geometry, &self.material);
        }

        // resets webgl
        state.gl.disable(WebGl2RenderingContext::STENCIL_TEST);
        state
            .gl
            .stencil_func(WebGl2RenderingContext::EQUAL, 0, 0xff);

        Ok(())
    }
}

struct OutliningMaterial {
    outline_width: u32,
    outline_color: [f32; 4],
    should_scale: bool,
}

impl Material for OutliningMaterial {
    fn name(&self) -> &'static str {
        "OutliningMaterial"
    }

    fn attribute_bindings(&self) -> &[AttributeBinding] {
        &[AttributeBinding::GeometryPosition]
    }

    fn uniform_bindings(&self) -> &[UniformBinding] {
        if self.should_scale {
            &[
                UniformBinding::ModelMatrix,
                UniformBinding::ViewProjMatrix,
                UniformBinding::FromMaterial("u_Color"),
                UniformBinding::FromMaterial("u_ShouldScale"),
                UniformBinding::CanvasSize,
                UniformBinding::FromMaterial("u_OutlineWidth"),
            ]
        } else {
            &[
                UniformBinding::ModelMatrix,
                UniformBinding::ViewProjMatrix,
                UniformBinding::FromMaterial("u_Color"),
                UniformBinding::FromMaterial("u_ShouldScale"),
            ]
        }
    }

    fn sources<'a>(&'a self) -> &[ShaderSource<'a>] {
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
            "u_ShouldScale" => Some(UniformValue::UnsignedInteger1(if self.should_scale {
                1
            } else {
                0
            })),
            "u_Color" => Some(UniformValue::FloatVector4(self.outline_color)),
            "u_OutlineWidth" => Some(UniformValue::UnsignedInteger1(self.outline_width)),
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
