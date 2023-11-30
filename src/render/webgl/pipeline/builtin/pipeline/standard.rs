use std::{
    any::Any,
    cell::RefCell,
    rc::{Rc, Weak},
};

use gl_matrix4rust::{mat4::Mat4, vec4::AsVec4};
use smallvec::SmallVec;
use web_sys::WebGl2RenderingContext;

use crate::{
    camera::Camera,
    entity::{Entity, EntityCollection, RenderEntity},
    material::{Material, MaterialRenderEntity},
    render::webgl::{
        attribute::{AttributeBinding, AttributeValue},
        draw::CullFace,
        error::Error,
        pipeline::{
            builtin::processor::{
                ClearColor, ClearDepth, EnableBlend, EnableCullFace, EnableDepthTest, Reset,
                SetCullFaceMode, UpdateCamera, UpdateViewport,
            },
            drawer::Drawer,
            flow::{BeforeDrawFlow, BeforeEachDrawFlow, PreparationFlow},
            process::Processor,
            RenderPipeline, RenderState, RenderStuff,
        },
        program::ShaderSource,
        uniform::{UniformBinding, UniformValue},
    },
    scene::Scene,
};

use super::picking::PickDetectionDrawer;

pub struct StandardRenderStuff<'a> {
    scene: &'a mut Scene,
}

impl<'a> StandardRenderStuff<'a> {
    pub fn new(scene: &'a mut Scene) -> Self {
        Self { scene }
    }
}

impl<'a> RenderStuff for StandardRenderStuff<'a> {
    fn camera(&self) -> &dyn Camera {
        self.scene.active_camera()
    }

    fn camera_mut(&mut self) -> &mut dyn Camera {
        self.scene.active_camera_mut()
    }

    fn entity_collection(&self) -> &EntityCollection {
        self.scene.entity_collection()
    }

    fn entity_collection_mut(&mut self) -> &mut EntityCollection {
        self.scene.entity_collection_mut()
    }
}

struct StandardDrawer;

impl<Pipeline> Drawer<Pipeline> for StandardDrawer
where
    Pipeline: RenderPipeline,
{
    #[inline(always)]
    fn before_draw(
        &mut self,
        _: &[Rc<RefCell<Entity>>],
        _: &mut Pipeline,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<BeforeDrawFlow, Error> {
        Ok(BeforeDrawFlow::FollowCollectedEntities)
    }

    #[inline(always)]
    fn before_each_draw(
        &mut self,
        _: &Rc<RefCell<Entity>>,
        _: usize,
        _: &[Rc<RefCell<Entity>>],
        _: &[Rc<RefCell<Entity>>],
        _: &mut Pipeline,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<BeforeEachDrawFlow, Error> {
        Ok(BeforeEachDrawFlow::FollowEntity)
    }

    #[inline(always)]
    fn after_each_draw(
        &mut self,
        _: &RenderEntity,
        _: usize,
        _: &[Rc<RefCell<Entity>>],
        _: &[Rc<RefCell<Entity>>],
        _: &mut Pipeline,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        Ok(())
    }

    #[inline(always)]
    fn after_draw(
        &mut self,
        _: &[Rc<RefCell<Entity>>],
        _: &[Rc<RefCell<Entity>>],
        _: &mut Pipeline,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        Ok(())
    }
}
struct OutliningOnePassDrawer {
    entity: Rc<RefCell<Option<Weak<RefCell<Entity>>>>>,
    entity_model_matrix: Mat4,
    scale: Mat4,
    material: OutliningMaterial,
}

impl OutliningOnePassDrawer {
    fn new(entity: Rc<RefCell<Option<Weak<RefCell<Entity>>>>>) -> Self {
        Self {
            entity,
            entity_model_matrix: Mat4::new_identity(),
            scale: Mat4::from_scaling(&[1.1, 1.1, 1.1]),
            material: OutliningMaterial {
                outline_color: [1.0, 0.0, 0.0, 1.0],
            },
        }
    }
}

impl<Pipeline> Drawer<Pipeline> for OutliningOnePassDrawer
where
    Pipeline: RenderPipeline,
{
    fn before_draw(
        &mut self,
        _: &[Rc<RefCell<Entity>>],
        _: &mut Pipeline,
        state: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<BeforeDrawFlow, Error> {
        let Some(entity) = &self
            .entity
            .borrow()
            .as_ref()
            .and_then(|entity| entity.upgrade())
        else {
            return Ok(BeforeDrawFlow::Skip);
        };

        let gl = state.gl();
        gl.enable(WebGl2RenderingContext::STENCIL_TEST);
        gl.clear_stencil(0);
        gl.clear(WebGl2RenderingContext::STENCIL_BUFFER_BIT);

        // render twice
        Ok(BeforeDrawFlow::Custom(vec![
            Rc::clone(entity),
            Rc::clone(entity),
            Rc::clone(entity),
        ]))
    }

    fn before_each_draw(
        &mut self,
        entity: &Rc<RefCell<Entity>>,
        drawing_index: usize,
        _: &[Rc<RefCell<Entity>>],
        _: &[Rc<RefCell<Entity>>],
        _: &mut Pipeline,
        state: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<BeforeEachDrawFlow, Error> {
        let gl = state.gl();

        match drawing_index {
            0 => {
                self.entity_model_matrix = *entity.borrow().model_matrix();
                entity
                    .borrow_mut()
                    .set_model_matrix(self.entity_model_matrix * self.scale);
                gl.depth_mask(false);
                gl.stencil_mask(0xFF);
                gl.stencil_func(WebGl2RenderingContext::ALWAYS, 1, 0xFF);
                gl.stencil_op(
                    WebGl2RenderingContext::KEEP,
                    WebGl2RenderingContext::REPLACE,
                    WebGl2RenderingContext::REPLACE,
                );
                Ok(BeforeEachDrawFlow::OverwriteMaterial(&mut self.material))
            }
            1 => {
                gl.depth_mask(false);
                gl.stencil_mask(0xFF);
                gl.stencil_func(WebGl2RenderingContext::ALWAYS, 0, 0xFF);
                gl.stencil_op(
                    WebGl2RenderingContext::KEEP,
                    WebGl2RenderingContext::REPLACE,
                    WebGl2RenderingContext::REPLACE,
                );
                Ok(BeforeEachDrawFlow::OverwriteMaterial(&mut self.material))
            }
            2 => {
                entity
                    .borrow_mut()
                    .set_model_matrix(self.entity_model_matrix * self.scale);
                gl.depth_mask(true);
                gl.stencil_mask(0x00);
                gl.stencil_func(WebGl2RenderingContext::EQUAL, 1, 0xFF);
                gl.stencil_op(
                    WebGl2RenderingContext::KEEP,
                    WebGl2RenderingContext::KEEP,
                    WebGl2RenderingContext::KEEP,
                );
                Ok(BeforeEachDrawFlow::OverwriteMaterial(&mut self.material))
            }
            _ => {
                panic!("unexpected drawing index when outlining")
            }
        }
    }

    fn after_each_draw(
        &mut self,
        render_entity: &RenderEntity,
        _: usize,
        _: &[Rc<RefCell<Entity>>],
        _: &[Rc<RefCell<Entity>>],
        _: &mut Pipeline,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        render_entity
            .entity()
            .borrow_mut()
            .set_model_matrix(self.entity_model_matrix);
        Ok(())
    }

    fn after_draw(
        &mut self,
        _: &[Rc<RefCell<Entity>>],
        _: &[Rc<RefCell<Entity>>],
        _: &mut Pipeline,
        state: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        let gl = state.gl();
        gl.disable(WebGl2RenderingContext::STENCIL_TEST);
        gl.depth_mask(true);
        gl.stencil_mask(0x00);
        gl.stencil_func(WebGl2RenderingContext::EQUAL, 0, 0xFF);
        gl.stencil_op(
            WebGl2RenderingContext::KEEP,
            WebGl2RenderingContext::KEEP,
            WebGl2RenderingContext::KEEP,
        );
        Ok(())
    }
}

struct OutliningMaterial {
    outline_color: [f32; 4],
}

impl Material for OutliningMaterial {
    fn name(&self) -> &'static str {
        "OutliningMaterial"
    }

    fn attribute_bindings(&self) -> &[AttributeBinding] {
        &[AttributeBinding::GeometryPosition]
    }

    fn uniform_bindings(&self) -> &[UniformBinding] {
        &[
            UniformBinding::ModelMatrix,
            UniformBinding::ViewProjMatrix,
            UniformBinding::FromMaterial("u_Color"),
        ]
    }

    fn sources<'a>(&'a self) -> &[ShaderSource<'a>] {
        &[
            ShaderSource::Vertex(include_str!("./shaders/outlining_vertex.glsl")),
            ShaderSource::Fragment(include_str!("./shaders/outlining_fragment.glsl")),
        ]
    }

    fn attribute_value(&self, _: &str, _: &MaterialRenderEntity) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, name: &str, _: &MaterialRenderEntity) -> Option<UniformValue> {
        match name {
            "u_Color" => Some(UniformValue::FloatVector4(self.outline_color)),
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

pub struct StandardPipeline {
    pick_receiver: Rc<RefCell<Option<Weak<RefCell<Entity>>>>>,
    pick_drawer: Rc<RefCell<PickDetectionDrawer>>,
    outlining_drawer: Rc<RefCell<OutliningOnePassDrawer>>,
    pre_processors: SmallVec<[Rc<RefCell<dyn Processor<Self>>>; 16]>,
    post_processors: SmallVec<[Rc<RefCell<dyn Processor<Self>>>; 16]>,
    drawers: SmallVec<[Rc<RefCell<dyn Drawer<Self>>>; 8]>,
}

impl StandardPipeline {
    pub fn new() -> Self {
        let mut pre_processors: SmallVec<[Rc<RefCell<dyn Processor<Self>>>; 16]> = SmallVec::new();
        pre_processors.push(Rc::new(RefCell::new(UpdateCamera)));
        pre_processors.push(Rc::new(RefCell::new(UpdateViewport)));
        pre_processors.push(Rc::new(RefCell::new(EnableDepthTest)));
        pre_processors.push(Rc::new(RefCell::new(EnableCullFace)));
        pre_processors.push(Rc::new(RefCell::new(EnableBlend)));
        pre_processors.push(Rc::new(RefCell::new(ClearColor::new(0.0, 0.0, 0.0, 0.0))));
        pre_processors.push(Rc::new(RefCell::new(ClearDepth::new(1.0))));
        pre_processors.push(Rc::new(RefCell::new(SetCullFaceMode::new(CullFace::Back))));

        let mut post_processors: SmallVec<[Rc<RefCell<dyn Processor<Self>>>; 16]> = SmallVec::new();
        post_processors.push(Rc::new(RefCell::new(Reset)));

        let pick_receiver = Rc::new(RefCell::new(None));
        let pick_drawer = Rc::new(RefCell::new(PickDetectionDrawer::new(
            Rc::clone(&pick_receiver),
            false,
        )));
        let outlining_drawer = Rc::new(RefCell::new(OutliningOnePassDrawer::new(Rc::clone(
            &pick_receiver,
        ))));
        let mut drawers: SmallVec<[Rc<RefCell<dyn Drawer<Self>>>; 8]> = SmallVec::new();
        drawers.push(Rc::clone(&pick_drawer) as Rc<RefCell<dyn Drawer<Self>>>);
        drawers.push(Rc::clone(&outlining_drawer) as Rc<RefCell<dyn Drawer<Self>>>);
        drawers.push(Rc::new(RefCell::new(StandardDrawer)));

        Self {
            pick_receiver,
            pick_drawer,
            outlining_drawer,
            pre_processors,
            post_processors,
            drawers,
        }
    }

    pub fn set_outline_color<V: AsVec4<f64>>(&mut self, color: V) {
        self.outlining_drawer.borrow_mut().material.outline_color = color.to_gl();
    }

    pub fn set_pick_position(&mut self, x: i32, y: i32) {
        self.pick_drawer.borrow_mut().set_position(x, y);
    }

    pub fn picked_entity(&self) -> Option<Rc<RefCell<Entity>>> {
        self.pick_receiver
            .borrow()
            .as_ref()
            .and_then(|entity| entity.upgrade())
    }

    pub fn take_picked_entity(&mut self) -> Option<Rc<RefCell<Entity>>> {
        self.pick_receiver
            .borrow_mut()
            .take()
            .and_then(|entity| entity.upgrade())
    }
}

impl RenderPipeline for StandardPipeline {
    #[inline(always)]
    fn prepare(
        &mut self,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<PreparationFlow, Error> {
        Ok(PreparationFlow::Continue)
    }

    #[inline(always)]
    fn pre_processors(
        &mut self,
        _: &[Rc<RefCell<Entity>>],
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<SmallVec<[Rc<RefCell<dyn Processor<Self>>>; 16]>, Error> {
        Ok(self.pre_processors.clone())
    }

    #[inline(always)]
    fn drawers(
        &mut self,
        _: &[Rc<RefCell<Entity>>],
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<SmallVec<[Rc<RefCell<dyn Drawer<Self>>>; 8]>, Error> {
        Ok(self.drawers.clone())
    }

    #[inline(always)]
    fn post_processors(
        &mut self,
        _: &[Rc<RefCell<Entity>>],
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<SmallVec<[Rc<RefCell<dyn Processor<Self>>>; 16]>, Error> {
        Ok(self.post_processors.clone())
    }

    #[inline(always)]
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[inline(always)]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
