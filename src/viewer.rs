use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

use gl_matrix4rust::{vec3::Vec3, vec4::Vec4};
use log::{error, warn};
use wasm_bindgen::closure::Closure;
use web_sys::Element;

use crate::{
    camera::Camera,
    cancel_animation_frame,
    controller::Controller,
    entity::Entity,
    error::Error,
    pipeline::webgl::{HdrToneMappingType, StandardPipeline, StandardPipelineShading},
    renderer::{webgl::WebGL2Renderer, Renderer},
    request_animation_frame,
    scene::Scene,
};

// pub const DEFAULT_RENDER_WHEN_NEEDED: bool = false;
pub const DEFAULT_RENDER_LOOP_INTERRUPTED_WHEN_ERROR: bool = true;

pub struct Viewer {
    mount: Option<Element>,
    scene: Rc<RefCell<Scene>>,
    camera: Rc<RefCell<dyn Camera + 'static>>,
    renderer: Rc<RefCell<WebGL2Renderer>>,
    controllers: Rc<RefCell<Vec<Box<dyn Controller>>>>,

    timestamp: *mut f64,
    standard_pipeline: *mut StandardPipeline,
    render_loop: *mut Option<Closure<dyn FnMut(f64)>>,
    render_loop_animation_handle: *mut Option<i32>,
    render_loop_interrupted_when_error: *mut bool,
}

impl Drop for Viewer {
    fn drop(&mut self) {
        unsafe {
            log::info!("1111");
            self.stop_render_loop();
            drop(Box::from_raw(self.timestamp));
            drop(Box::from_raw(self.standard_pipeline));
            drop(Box::from_raw(self.render_loop));
            drop(Box::from_raw(self.render_loop_animation_handle));
            drop(Box::from_raw(self.render_loop_interrupted_when_error));
        }
    }
}

impl Viewer {
    pub fn new<C>(scene: Scene, camera: C) -> Result<Self, Error>
    where
        C: Camera + 'static,
    {
        let render = Rc::new(RefCell::new(WebGL2Renderer::new(
            scene.canvas().clone(),
            None,
        )?));
        let scene = Rc::new(RefCell::new(scene));
        let camera = Rc::new(RefCell::new(camera));
        let controllers = Rc::new(RefCell::new(Vec::new()));

        Ok(Self {
            mount: None,
            scene,
            camera,
            renderer: render,
            controllers,

            timestamp: Box::leak(Box::new(0.0)),
            standard_pipeline: Box::leak(Box::new(StandardPipeline::new())),
            render_loop: Box::leak(Box::new(None)),
            render_loop_animation_handle: Box::leak(Box::new(None)),
            render_loop_interrupted_when_error: Box::leak(Box::new(
                DEFAULT_RENDER_LOOP_INTERRUPTED_WHEN_ERROR,
            )),
        })
    }

    pub fn mount(&self) -> Option<&Element> {
        self.mount.as_ref()
    }

    pub fn set_mount(&mut self, mount: Option<Element>) -> Result<(), Error> {
        let scene = self.scene.borrow();

        if let Some(mounting) = self.mount.take() {
            let _ = mounting.remove_child(scene.canvas());
        }

        if let Some(mount) = &mount {
            mount
                .append_child(scene.canvas())
                .or(Err(Error::MountElementFailure))?;
        }
        self.mount = mount;

        Ok(())
    }

    pub fn timestamp(&self) -> f64 {
        unsafe { *self.timestamp }
    }

    pub fn camera(&self) -> &Rc<RefCell<dyn Camera + 'static>> {
        &self.camera
    }

    pub fn scene(&self) -> &Rc<RefCell<Scene>> {
        &self.scene
    }

    pub fn renderer(&self) -> &Rc<RefCell<WebGL2Renderer>> {
        &self.renderer
    }

    // pub fn controllers(&self) -> &Rc<RefCell<Vec<Box<dyn Controller>>>> {
    //     &self.controllers
    // }

    // pub fn render_when_needed(&self) -> bool {
    //     self.render_when_needed
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
        unsafe { (*self.standard_pipeline).culling_enabled() }
    }

    pub fn enable_culling(&mut self) {
        unsafe {
            (*self.standard_pipeline).enable_culling();
        }
    }

    pub fn disable_culling(&mut self) {
        unsafe {
            (*self.standard_pipeline).disable_culling();
        }
    }

    pub fn lighting_enabled(&self) -> bool {
        unsafe { (*self.standard_pipeline).lighting_enabled() }
    }

    pub fn enable_lighting(&mut self) {
        unsafe {
            (*self.standard_pipeline).enable_lighting();
        }
    }

    pub fn disable_lighting(&mut self) {
        unsafe {
            (*self.standard_pipeline).disable_lighting();
        }
    }

    /// Returns `true` if entity distance sorting enabled.
    pub fn distance_sorting_enabled(&self) -> bool {
        unsafe { (*self.standard_pipeline).distance_sorting_enabled() }
    }

    pub fn enable_distance_sorting(&mut self) {
        unsafe {
            (*self.standard_pipeline).enable_distance_sorting();
        }
    }

    pub fn disable_distance_sorting(&mut self) {
        unsafe {
            (*self.standard_pipeline).disable_distance_sorting();
        }
    }

    pub fn pipeline_shading(&self) -> StandardPipelineShading {
        unsafe { (*self.standard_pipeline).pipeline_shading() }
    }

    pub fn set_pipeline_shading(&mut self, shading: StandardPipelineShading) {
        unsafe {
            if let StandardPipelineShading::Picking = shading {
                warn!("manually setting pipeline shading to picking is not allowed");
                return;
            } else if let StandardPipelineShading::DeferredShading = shading {
                if self.hdr_supported() {
                    (*self.standard_pipeline).set_pipeline_shading(shading);
                } else {
                    warn!("deferred shading if not supported, fallback to forward shading");
                    (*self.standard_pipeline)
                        .set_pipeline_shading(StandardPipelineShading::ForwardShading);
                }
            } else {
                (*self.standard_pipeline).set_pipeline_shading(shading);
            }
        }
    }

    pub fn clear_color(&self) -> Vec4<f32> {
        unsafe { (*self.standard_pipeline).clear_color().clone() }
    }

    pub fn set_clear_color(&mut self, clear_color: Vec4<f32>) {
        unsafe {
            (*self.standard_pipeline).set_clear_color(clear_color);
        }
    }

    pub fn gamma_correction_enabled(&self) -> bool {
        unsafe { (*self.standard_pipeline).gamma_correction_enabled() }
    }

    pub fn enable_gamma_correction(&mut self) {
        unsafe {
            (*self.standard_pipeline).enable_gamma_correction();
        }
    }

    pub fn disable_gamma_correction(&mut self) {
        unsafe {
            (*self.standard_pipeline).disable_gamma_correction();
        }
    }

    pub fn gamma(&self) -> f32 {
        unsafe { (*self.standard_pipeline).gamma() }
    }

    pub fn set_gamma(&mut self, gamma: f32) {
        unsafe {
            (*self.standard_pipeline).set_gamma(gamma);
        }
    }

    pub fn multisamples(&self) -> Option<i32> {
        unsafe { (*self.standard_pipeline).multisamples() }
    }

    pub fn set_multisamples(&mut self, samples: Option<i32>) {
        unsafe {
            (*self.standard_pipeline).set_multisamples(samples);
        }
    }

    pub fn hdr_supported(&self) -> bool {
        self.renderer
            .borrow()
            .capabilities()
            .color_buffer_float_supported()
    }

    pub fn hdr_enabled(&self) -> bool {
        unsafe { (*self.standard_pipeline).hdr_enabled() }
    }

    pub fn enable_hdr(&mut self) {
        unsafe {
            (*self.standard_pipeline).enable_hdr();
        }
    }

    pub fn disable_hdr(&mut self) {
        unsafe {
            (*self.standard_pipeline).disable_hdr();
        }
    }

    pub fn hdr_tone_mapping_type(&self) -> HdrToneMappingType {
        unsafe { (*self.standard_pipeline).hdr_tone_mapping_type() }
    }

    pub fn set_hdr_tone_mapping_type(&mut self, hdr_tone_mapping_type: HdrToneMappingType) {
        unsafe {
            (*self.standard_pipeline).set_hdr_tone_mapping_type(hdr_tone_mapping_type);
        }
    }

    pub fn bloom_enabled(&self) -> bool {
        unsafe { (*self.standard_pipeline).bloom_enabled() }
    }

    pub fn enable_bloom(&mut self) {
        unsafe {
            (*self.standard_pipeline).enable_bloom();
        }
    }

    pub fn disable_bloom(&mut self) {
        unsafe {
            (*self.standard_pipeline).disable_bloom();
        }
    }

    pub fn bloom_blur_epoch(&self) -> usize {
        unsafe { (*self.standard_pipeline).bloom_blur_epoch() }
    }

    pub fn set_bloom_blur_epoch(&mut self, epoch: usize) {
        unsafe {
            (*self.standard_pipeline).set_bloom_blur_epoch(epoch);
        }
    }

    pub fn add_controller<C>(&mut self, mut controller: C)
    where
        C: Controller + 'static,
    {
        controller.on_add(self);
        self.controllers.borrow_mut().push(Box::new(controller));
    }

    pub fn remove_controller(&mut self, index: usize) -> Option<Box<dyn Controller>> {
        let mut controllers = self.controllers.borrow_mut();

        if index > controllers.len() - 1 {
            return None;
        }

        let mut controller = controllers.remove(index);

        drop(controllers);

        controller.on_remove(self);
        Some(controller)
    }

    pub fn pick_entity(
        &mut self,
        window_position_x: i32,
        window_position_y: i32,
    ) -> Result<Option<RefMut<'_, Entity>>, Error> {
        unsafe {
            let timestamp = *self.timestamp;
            let mut scene = self.scene.borrow_mut();
            let mut camera = self.camera.borrow_mut();
            let mut renderer = self.renderer.borrow_mut();
            let pipeline = &mut *self.standard_pipeline;

            let previous_pipeline_shading = pipeline.pipeline_shading();
            pipeline.set_pipeline_shading(StandardPipelineShading::Picking);
            renderer.render(pipeline, &mut *camera, &mut *scene, timestamp)?;
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
            let timestamp = *self.timestamp;
            let mut scene = self.scene.borrow_mut();
            let mut camera = self.camera.borrow_mut();
            let mut renderer = self.renderer.borrow_mut();
            let pipeline = &mut *self.standard_pipeline;

            let previous_pipeline_shading = pipeline.pipeline_shading();
            pipeline.set_pipeline_shading(StandardPipelineShading::Picking);
            renderer.render(pipeline, &mut *camera, &mut *scene, timestamp)?;
            pipeline.set_pipeline_shading(previous_pipeline_shading);

            let position = pipeline.pick_position(window_position_x, window_position_y)?;
            Ok(position)
        }
    }

    pub fn render(&mut self) -> Result<(), Error> {
        unsafe {
            let timestamp = *self.timestamp;
            let mut scene = self.scene.borrow_mut();
            let mut camera = self.camera.borrow_mut();
            let mut render = self.renderer.borrow_mut();
            let pipeline = &mut *self.standard_pipeline;

            render.render(pipeline, &mut *camera, &mut scene, timestamp)?;

            Ok(())
        }
    }

    pub fn start_render_loop(&mut self) {
        unsafe {
            if (*self.render_loop).is_some() {
                return;
            }

            let timestamp = self.timestamp;
            let scene = Rc::clone(&self.scene);
            let camera = Rc::clone(&self.camera);
            let renderer = Rc::clone(&self.renderer);
            let pipeline = self.standard_pipeline;
            let render_loop = self.render_loop;
            let render_animation_handle = self.render_loop_animation_handle;
            let render_loop_interrupted_when_error = self.render_loop_interrupted_when_error;
            *self.render_loop = Some(Closure::new(move |t| {
                *timestamp = t;

                let mut scene = scene.borrow_mut();
                let mut camera = camera.borrow_mut();
                let mut renderer = renderer.borrow_mut();
                let pipeline = &mut *pipeline;
                let result = renderer.render(pipeline, &mut *camera, &mut *scene, *timestamp);
                if let Err(err) = result {
                    error!("error occurred during rendering {err}");
                    if *render_loop_interrupted_when_error {
                        *render_loop = None;
                        *render_animation_handle = None;
                        return;
                    }
                }

                *render_animation_handle =
                    Some(request_animation_frame((*render_loop).as_ref().unwrap()));
            }));

            *self.render_loop_animation_handle = Some(request_animation_frame(
                (*self.render_loop).as_ref().unwrap(),
            ));
        }
    }

    pub fn stop_render_loop(&mut self) {
        unsafe {
            if let Some(handle) = (*self.render_loop_animation_handle).take() {
                cancel_animation_frame(handle);
                *self.render_loop = None;
            }
        }
    }
}
