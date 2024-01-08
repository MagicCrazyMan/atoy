use std::{cell::RefCell, rc::Rc};

use gl_matrix4rust::vec3::Vec3;
use log::error;
use wasm_bindgen::closure::Closure;
use web_sys::Element;

use crate::{
    camera::Camera,
    controller::Controller,
    entity::Entity,
    render::{
        webgl::{
            error::Error,
            pipeline::{picking::PickingPipeline, StandardPipeline},
            WebGL2Render,
        },
        Render,
    },
    request_animation_frame,
    scene::Scene,
};

pub struct Viewer {
    mount: Rc<RefCell<Option<Element>>>,
    timestamp: Rc<RefCell<f64>>,
    controllers: Rc<RefCell<Vec<Box<dyn Controller>>>>,
    scene: Rc<RefCell<Scene>>,
    camera: Rc<RefCell<dyn Camera>>,
    render: Rc<RefCell<WebGL2Render>>,
    standard_pipeline: Rc<RefCell<StandardPipeline>>,
    picking_pipeline: Rc<RefCell<PickingPipeline>>,
    render_loop: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>>,
    stop_render_loop_when_error: Rc<RefCell<bool>>,
}

impl Viewer {
    pub fn new<C>(scene: Scene, camera: C) -> Result<Self, Error>
    where
        C: Camera + 'static,
    {
        Ok(Self {
            mount: Rc::new(RefCell::new(None)),
            timestamp: Rc::new(RefCell::new(0.0)),
            controllers: Rc::new(RefCell::new(Vec::new())),
            scene: Rc::new(RefCell::new(scene)),
            camera: Rc::new(RefCell::new(camera)),
            render: Rc::new(RefCell::new(WebGL2Render::new(None)?)),
            standard_pipeline: Rc::new(RefCell::new(StandardPipeline::new())),
            picking_pipeline: Rc::new(RefCell::new(PickingPipeline::new())),
            render_loop: Rc::new(RefCell::new(None)),
            stop_render_loop_when_error: Rc::new(RefCell::new(true)),
        })
    }

    pub fn set_mount(&mut self, mount: Option<Element>) -> Result<(), Error> {
        let mut mounting = self.mount.borrow_mut();
        let render = self.render.borrow();

        if let Some(mounting) = &*mounting {
            let _ = mounting.remove_child(render.canvas());
        }

        if let Some(mount) = &mount {
            mount
                .append_child(render.canvas())
                .or(Err(Error::MountElementFailed))?;
        }
        *mounting = mount;

        Ok(())
    }

    pub fn timestamp(&self) -> f64 {
        *self.timestamp.borrow()
    }

    pub fn scene(&self) -> &Rc<RefCell<Scene>> {
        &self.scene
    }

    pub fn render(&self) -> &Rc<RefCell<WebGL2Render>> {
        &self.render
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
    ) -> Result<Option<&mut Entity>, Error> {
        let mut picking_pipeline = self.picking_pipeline.borrow_mut();
        let timestamp = *self.timestamp.borrow_mut();
        let mut scene = self.scene.borrow_mut();
        let mut render = self.render.borrow_mut();
        let mut camera = self.camera.borrow_mut();

        render.render(&mut *picking_pipeline, &mut *camera, &mut *scene, timestamp)?;

        picking_pipeline.pick_entity(window_position_x, window_position_y)
    }

    pub fn pick_position(
        &mut self,
        window_position_x: i32,
        window_position_y: i32,
    ) -> Result<Option<Vec3>, Error> {
        let mut picking_pipeline = self.picking_pipeline.borrow_mut();
        let timestamp = *self.timestamp.borrow_mut();
        let mut scene = self.scene.borrow_mut();
        let mut render = self.render.borrow_mut();
        let mut camera = self.camera.borrow_mut();

        render.render(&mut *picking_pipeline, &mut *camera, &mut *scene, timestamp)?;

        picking_pipeline.pick_position(window_position_x, window_position_y)
    }

    pub fn render_sync(&mut self) -> Result<(), Error> {
        let timestamp = *self.timestamp.borrow_mut();
        let mut scene = self.scene.borrow_mut();
        let mut render = self.render.borrow_mut();
        let mut camera = self.camera.borrow_mut();
        let mut standard_pipeline = self.standard_pipeline.borrow_mut();

        let start = crate::window().performance().unwrap().now();
        let result = render.render(
            &mut *standard_pipeline,
            &mut *camera,
            &mut *scene,
            timestamp,
        );
        let end = crate::window().performance().unwrap().now();
        crate::document()
            .get_element_by_id("total")
            .unwrap()
            .set_inner_html(&format!("{:.2}", end - start));
        result
    }

    pub fn start_render_loop(&mut self) {
        if self.render_loop.borrow().is_some() {
            return;
        }

        let me = self.clone();

        let render_loop: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
        let render_loop_cloned = Rc::clone(&render_loop);
        *(*render_loop_cloned).borrow_mut() = Some(Closure::new(move |timestamp| {
            let mut me = me.clone();
            *me.timestamp.borrow_mut() = timestamp;

            if let Err(err) = me.render_sync() {
                error!("error occurred during rendering {err}");
                if *me.stop_render_loop_when_error.borrow() {
                    return;
                }
            }

            let render_loop = render_loop.borrow();
            if let Some(render_loop) = render_loop.as_ref() {
                request_animation_frame(render_loop);
            }
        }));
        request_animation_frame(render_loop_cloned.borrow().as_ref().unwrap());

        self.render_loop = render_loop_cloned;
    }

    pub fn stop_render_loop(&mut self) {
        *self.render_loop.borrow_mut() = None;
    }
}

impl Clone for Viewer {
    fn clone(&self) -> Self {
        Self {
            mount: self.mount.clone(),
            timestamp: self.timestamp.clone(),
            controllers: self.controllers.clone(),
            scene: self.scene.clone(),
            camera: self.camera.clone(),
            render: self.render.clone(),
            standard_pipeline: self.standard_pipeline.clone(),
            picking_pipeline: self.picking_pipeline.clone(),
            render_loop: self.render_loop.clone(),
            stop_render_loop_when_error: self.stop_render_loop_when_error.clone(),
        }
    }
}
