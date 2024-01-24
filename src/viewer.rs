use std::{
    cell::{Ref, RefCell, RefMut},
    rc::{Rc, Weak},
};

use gl_matrix4rust::{vec3::Vec3, vec4::Vec4};
use log::{error, warn};
use wasm_bindgen::{closure::Closure, prelude::wasm_bindgen, JsValue};
use web_sys::Element;

use crate::{
    camera::Camera,
    controller::Controller,
    entity::Entity,
    error::Error,
    render::{
        webgl::{
            pipeline::{HdrToneMappingType, StandardPipeline, StandardPipelineShading},
            WebGL2Render,
        },
        Render,
    },
    request_animation_frame,
    scene::Scene,
};

// pub const DEFAULT_RENDER_WHEN_NEEDED: bool = false;
pub const DEFAULT_RENDER_LOOP_INTERRUPTED_WHEN_ERROR: bool = true;

struct Status {
    mount: Option<Element>,
    timestamp: f64,

    render_loop: Option<Closure<dyn FnMut(f64)>>,
    render_loop_stopping: bool,
    render_loop_interrupted_when_error: bool,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Viewer {
    scene: Rc<RefCell<Scene>>,
    camera: Rc<RefCell<dyn Camera + 'static>>,
    render: Rc<RefCell<WebGL2Render>>,
    standard_pipeline: Rc<RefCell<StandardPipeline>>,
    controllers: Rc<RefCell<Vec<Box<dyn Controller>>>>,
    status: Rc<RefCell<Status>>,
}

#[wasm_bindgen]
impl Viewer {
    pub fn mount_wasm(&self) -> Option<Element> {
        self.mount()
    }

    pub fn set_mount_wasm(&mut self, mount: Option<Element>) -> Result<(), Error> {
        self.set_mount(mount)
    }

    pub fn render_when_needed_wasm(&self) -> bool {
        // self.render_when_needed()
        true
    }

    pub fn enable_render_when_needed_wasm(&mut self) {
        // self.enable_render_when_needed()
    }

    pub fn disable_render_when_needed_wasm(&mut self) {
        // self.disable_render_when_needed()
    }

    /// Returns `true` if entity culling enabled.
    pub fn culling_enabled_wasm(&self) -> bool {
        self.culling_enabled()
    }

    pub fn enable_culling_wasm(&mut self) {
        self.enable_culling()
    }

    pub fn disable_culling_wasm(&mut self) {
        self.disable_culling()
    }

    /// Returns `true` if entity distance sorting enabled.
    pub fn distance_sorting_enabled_wasm(&self) -> bool {
        self.distance_sorting_enabled()
    }

    pub fn enable_distance_sorting_wasm(&mut self) {
        self.enable_distance_sorting()
    }

    pub fn disable_distance_sorting_wasm(&mut self) {
        self.disable_distance_sorting()
    }

    pub fn pipeline_shading_wasm(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.pipeline_shading()).unwrap()
    }

    pub fn set_pipeline_shading_wasm(&mut self, shading: JsValue) {
        let shading = serde_wasm_bindgen::from_value::<StandardPipelineShading>(shading).unwrap();
        self.set_pipeline_shading(shading);
    }

    pub fn clear_color_wasm(&self) -> Box<[f32]> {
        Box::new(self.clear_color().raw().clone())
    }

    pub fn set_clear_color_wasm(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.set_clear_color(Vec4::new(r, g, b, a))
    }

    pub fn gamma_correction_enabled_wasm(&self) -> bool {
        self.gamma_correction_enabled()
    }

    pub fn enable_gamma_correction_wasm(&mut self) {
        self.enable_gamma_correction();
    }

    pub fn disable_gamma_correction_wasm(&mut self) {
        self.disable_gamma_correction();
    }

    pub fn gamma_wasm(&self) -> f32 {
        self.gamma()
    }

    pub fn set_gamma_wasm(&mut self, gamma: f32) {
        self.set_gamma(gamma);
    }

    pub fn lighting_enabled_wasm(&self) -> bool {
        self.lighting_enabled()
    }

    pub fn enable_lighting_wasm(&mut self) {
        self.enable_lighting();
    }

    pub fn disable_lighting_wasm(&mut self) {
        self.disable_lighting();
    }

    pub fn multisamples_wasm(&self) -> Option<i32> {
        self.multisamples()
    }

    pub fn set_multisamples_wasm(&mut self, samples: Option<i32>) {
        self.set_multisamples(samples)
    }

    pub fn hdr_enabled_wasm(&self) -> bool {
        self.hdr_enabled()
    }

    pub fn enable_hdr_wasm(&mut self) {
        self.enable_hdr();
    }

    pub fn disable_hdr_wasm(&mut self) {
        self.disable_hdr();
    }

    pub fn hdr_tone_mapping_type_wasm(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.hdr_tone_mapping_type()).unwrap()
    }

    pub fn set_hdr_tone_mapping_type_wasm(&mut self, hdr_tone_mapping_type: JsValue) {
        let t =
            serde_wasm_bindgen::from_value::<HdrToneMappingType>(hdr_tone_mapping_type).unwrap();
        self.set_hdr_tone_mapping_type(t);
    }

    pub fn bloom_enabled_wasm(&self) -> bool {
        self.bloom_enabled()
    }

    pub fn enable_bloom_wasm(&mut self) {
        self.enable_bloom()
    }

    pub fn disable_bloom_wasm(&mut self) {
        self.disable_bloom()
    }

    pub fn bloom_blur_epoch_wasm(&self) -> usize {
        self.bloom_blur_epoch()
    }

    pub fn set_bloom_blur_epoch_wasm(&mut self, epoch: usize) {
        self.set_bloom_blur_epoch(epoch);
    }
}

impl Viewer {
    pub fn new<C>(scene: Scene, camera: C) -> Result<Self, Error>
    where
        C: Camera + 'static,
    {
        let mut render = WebGL2Render::new(scene.canvas().clone(), None)?;
        let standard_pipeline =
            Rc::new(RefCell::new(StandardPipeline::new(render.hdr_supported())));
        let render = Rc::new(RefCell::new(render));
        let scene = Rc::new(RefCell::new(scene));
        let camera = Rc::new(RefCell::new(camera));
        let controllers = Rc::new(RefCell::new(Vec::new()));
        let status = Rc::new(RefCell::new(Status {
            mount: None,
            timestamp: 0.0,
            render_loop: None,
            render_loop_stopping: false,
            render_loop_interrupted_when_error: DEFAULT_RENDER_LOOP_INTERRUPTED_WHEN_ERROR,
        }));

        Ok(Self {
            scene,
            camera,
            render,
            standard_pipeline,
            controllers,
            status,
        })
    }

    pub fn downgrade(&self) -> ViewerWeak {
        ViewerWeak {
            scene: Rc::downgrade(&self.scene),
            camera: Rc::downgrade(&self.camera),
            render: Rc::downgrade(&self.render),
            standard_pipeline: Rc::downgrade(&self.standard_pipeline),
            controllers: Rc::downgrade(&self.controllers),
            status: Rc::downgrade(&self.status),
        }
    }

    #[inline]
    fn status(&self) -> Ref<'_, Status> {
        self.status.borrow()
    }

    #[inline]
    fn status_mut(&self) -> RefMut<'_, Status> {
        self.status.borrow_mut()
    }

    #[inline]
    fn standard_pipeline(&self) -> Ref<'_, StandardPipeline> {
        self.standard_pipeline.borrow()
    }

    #[inline]
    fn standard_pipeline_mut(&self) -> RefMut<'_, StandardPipeline> {
        self.standard_pipeline.borrow_mut()
    }

    #[inline]
    pub fn camera(&self) -> Ref<'_, dyn Camera + 'static> {
        self.camera.borrow()
    }

    #[inline]
    pub fn camera_mut(&self) -> RefMut<'_, dyn Camera + 'static> {
        self.camera.borrow_mut()
    }

    #[inline]
    pub fn scene(&self) -> Ref<'_, Scene> {
        self.scene.borrow()
    }

    #[inline]
    pub fn scene_mut(&self) -> RefMut<'_, Scene> {
        self.scene.borrow_mut()
    }

    #[inline]
    pub fn render(&self) -> Ref<'_, WebGL2Render> {
        self.render.borrow()
    }

    #[inline]
    pub fn render_mut(&self) -> RefMut<'_, WebGL2Render> {
        self.render.borrow_mut()
    }

    #[inline]
    pub fn controllers(&self) -> Ref<'_, Vec<Box<dyn Controller>>> {
        self.controllers.borrow()
    }

    #[inline]
    fn controllers_mut(&self) -> RefMut<'_, Vec<Box<dyn Controller>>> {
        self.controllers.borrow_mut()
    }

    pub fn timestamp(&self) -> f64 {
        self.status().timestamp
    }

    pub fn mount(&self) -> Option<Element> {
        self.status().mount.clone()
    }

    pub fn set_mount(&mut self, mount: Option<Element>) -> Result<(), Error> {
        let mut status = self.status_mut();
        let render = self.render();

        if let Some(mounting) = status.mount.take() {
            let _ = mounting.remove_child(render.canvas());
        }

        if let Some(mount) = &mount {
            mount
                .append_child(render.canvas())
                .or(Err(Error::MountElementFailure))?;
        }
        status.mount = mount;

        Ok(())
    }

    // pub fn render_when_needed(&self) -> bool {
    //     self.status().render_when_needed
    // }

    // pub fn enable_render_when_needed(&mut self) {
    //     let mut status = self.status_mut();
    //     status.render_when_needed = true;
    //     status.render_next = true;
    // }

    // pub fn disable_render_when_needed(&mut self) {
    //     let mut status = self.status_mut();
    //     status.render_when_needed = false;
    //     status.render_next = true;
    // }

    /// Returns `true` if entity culling enabled.
    pub fn culling_enabled(&self) -> bool {
        self.standard_pipeline().culling_enabled()
    }

    pub fn enable_culling(&mut self) {
        self.standard_pipeline_mut().enable_culling();
    }

    pub fn disable_culling(&mut self) {
        self.standard_pipeline_mut().disable_culling();
    }

    pub fn lighting_enabled(&self) -> bool {
        self.standard_pipeline().lighting_enabled()
    }

    pub fn enable_lighting(&mut self) {
        self.standard_pipeline_mut().enable_lighting();
    }

    pub fn disable_lighting(&mut self) {
        self.standard_pipeline_mut().disable_lighting();
    }

    /// Returns `true` if entity distance sorting enabled.
    pub fn distance_sorting_enabled(&self) -> bool {
        self.standard_pipeline().distance_sorting_enabled()
    }

    pub fn enable_distance_sorting(&mut self) {
        self.standard_pipeline_mut().enable_distance_sorting();
    }

    pub fn disable_distance_sorting(&mut self) {
        self.standard_pipeline_mut().disable_distance_sorting();
    }

    pub fn pipeline_shading(&self) -> StandardPipelineShading {
        self.standard_pipeline().pipeline_shading()
    }

    pub fn set_pipeline_shading(&mut self, shading: StandardPipelineShading) {
        if shading == StandardPipelineShading::Picking {
            warn!("manually setting pipeline shading to picking is not allowed");
            return;
        }

        self.standard_pipeline_mut().set_pipeline_shading(shading);
    }

    pub fn clear_color(&self) -> Vec4<f32> {
        self.standard_pipeline().clear_color().clone()
    }

    pub fn set_clear_color(&mut self, clear_color: Vec4<f32>) {
        self.standard_pipeline_mut().set_clear_color(clear_color);
    }

    pub fn gamma_correction_enabled(&self) -> bool {
        self.standard_pipeline().gamma_correction_enabled()
    }

    pub fn enable_gamma_correction(&mut self) {
        self.standard_pipeline_mut().enable_gamma_correction();
    }

    pub fn disable_gamma_correction(&mut self) {
        self.standard_pipeline_mut().disable_gamma_correction();
    }

    pub fn gamma(&self) -> f32 {
        self.standard_pipeline().gamma()
    }

    pub fn set_gamma(&mut self, gamma: f32) {
        self.standard_pipeline_mut().set_gamma(gamma);
    }

    pub fn multisamples(&self) -> Option<i32> {
        self.standard_pipeline().multisamples()
    }

    pub fn set_multisamples(&mut self, samples: Option<i32>) {
        self.standard_pipeline_mut().set_multisamples(samples);
    }

    pub fn hdr_supported(&mut self) -> bool {
        self.render_mut().hdr_supported()
    }

    pub fn hdr_enabled(&self) -> bool {
        self.standard_pipeline().hdr_enabled()
    }

    pub fn enable_hdr(&mut self) {
        self.standard_pipeline_mut().enable_hdr();
    }

    pub fn disable_hdr(&mut self) {
        self.standard_pipeline_mut().disable_hdr();
    }

    pub fn hdr_tone_mapping_type(&self) -> HdrToneMappingType {
        self.standard_pipeline().hdr_tone_mapping_type()
    }

    pub fn set_hdr_tone_mapping_type(&mut self, hdr_tone_mapping_type: HdrToneMappingType) {
        self.standard_pipeline_mut()
            .set_hdr_tone_mapping_type(hdr_tone_mapping_type);
    }

    pub fn bloom_enabled(&self) -> bool {
        self.standard_pipeline().bloom_enabled()
    }

    pub fn enable_bloom(&mut self) {
        self.standard_pipeline_mut().enable_bloom();
    }

    pub fn disable_bloom(&mut self) {
        self.standard_pipeline_mut().disable_bloom();
    }

    pub fn bloom_blur_epoch(&self) -> usize {
        self.standard_pipeline().bloom_blur_epoch()
    }

    pub fn set_bloom_blur_epoch(&mut self, epoch: usize) {
        self.standard_pipeline_mut().set_bloom_blur_epoch(epoch);
    }

    pub fn add_controller<C>(&mut self, mut controller: C)
    where
        C: Controller + 'static,
    {
        controller.on_add(self);
        self.controllers_mut().push(Box::new(controller));
    }

    pub fn remove_controller(&mut self, index: usize) -> Option<Box<dyn Controller>> {
        if index > self.controllers().len() - 1 {
            return None;
        }

        let mut controller = self.controllers_mut().remove(index);

        controller.on_remove(self);
        Some(controller)
    }

    pub fn pick_entity(
        &mut self,
        window_position_x: i32,
        window_position_y: i32,
    ) -> Result<Option<RefMut<'_, Entity>>, Error> {
        unsafe {
            let mut scene = self.scene_mut();
            let mut camera = self.camera_mut();
            let mut render = self.render_mut();
            let mut pipeline = self.standard_pipeline_mut();
            let status = self.status_mut();
            let previous_pipeline_shading = pipeline.pipeline_shading();
            pipeline.set_pipeline_shading(StandardPipelineShading::Picking);
            render.render(&mut *pipeline, &mut *camera, &mut *scene, status.timestamp)?;
            pipeline.set_pipeline_shading(previous_pipeline_shading);

            let Some(id) = pipeline.pick_entity_id(window_position_x, window_position_y)? else {
                return Ok(None);
            };

            let entity = RefMut::map(scene, |scene| {
                scene.entity_container_mut().entity_mut(&id).unwrap()
            });
            Ok(Some(entity))
        }
    }

    pub fn pick_position(
        &mut self,
        window_position_x: i32,
        window_position_y: i32,
    ) -> Result<Option<Vec3>, Error> {
        unsafe {
            let mut scene = self.scene_mut();
            let mut camera = self.camera_mut();
            let mut render = self.render_mut();
            let mut pipeline = self.standard_pipeline_mut();
            let status = self.status_mut();
            let previous_pipeline_shading = pipeline.pipeline_shading();
            pipeline.set_pipeline_shading(StandardPipelineShading::Picking);
            render.render(&mut *pipeline, &mut *camera, &mut *scene, status.timestamp)?;
            pipeline.set_pipeline_shading(previous_pipeline_shading);

            let position = pipeline.pick_position(window_position_x, window_position_y)?;
            Ok(position)
        }
    }

    pub fn render_frame(&mut self) -> Result<(), Error> {
        let mut scene = self.scene_mut();
        let mut camera = self.camera_mut();
        let mut render = self.render_mut();
        let mut pipeline = self.standard_pipeline_mut();
        let status = self.status_mut();

        render.render(&mut *pipeline, &mut *camera, &mut *scene, status.timestamp)?;

        Ok(())
    }

    pub fn start_render_loop(&mut self) {
        if self.status().render_loop.is_some() {
            return;
        }

        let me = self.downgrade();
        self.status_mut().render_loop_stopping = false;
        self.status_mut().render_loop = Some(Closure::new(move |timestamp| {
            let Some(mut me) = me.upgrade() else {
                return;
            };

            me.status_mut().timestamp = timestamp;

            if me.status().render_loop_stopping {
                me.status_mut().render_loop = None;
                me.status_mut().render_loop_stopping = false;
                return;
            }

            let render_loop_interrupted_when_error = me.status().render_loop_interrupted_when_error;
            let result = me.render_frame();
            if let Err(err) = result {
                error!("error occurred during rendering {err}");
                if render_loop_interrupted_when_error {
                    me.status_mut().render_loop = None;
                    me.status_mut().render_loop_stopping = false;
                    return;
                }
            }

            if let Some(render_loop) = me.clone().status().render_loop.as_ref() {
                request_animation_frame(render_loop);
            } else {
                me.status_mut().render_loop_stopping = false;
            }
        }));
        request_animation_frame(self.status().render_loop.as_ref().unwrap());
    }

    pub fn stop_render_loop(&mut self) {
        self.status_mut().render_loop_stopping = true;
    }
}

#[derive(Clone)]
pub struct ViewerWeak {
    scene: Weak<RefCell<Scene>>,
    camera: Weak<RefCell<dyn Camera>>,
    render: Weak<RefCell<WebGL2Render>>,
    standard_pipeline: Weak<RefCell<StandardPipeline>>,
    controllers: Weak<RefCell<Vec<Box<dyn Controller>>>>,
    status: Weak<RefCell<Status>>,
}

impl ViewerWeak {
    pub fn upgrade(&self) -> Option<Viewer> {
        let scene = self.scene.upgrade()?;
        let camera = self.camera.upgrade()?;
        let render = self.render.upgrade()?;
        let standard_pipeline = self.standard_pipeline.upgrade()?;
        let controllers = self.controllers.upgrade()?;
        let status = self.status.upgrade()?;
        Some(Viewer {
            scene,
            camera,
            render,
            standard_pipeline,
            controllers,
            status,
        })
    }
}
