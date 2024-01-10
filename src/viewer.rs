use std::rc::{Rc, Weak};

use gl_matrix4rust::vec3::Vec3;
use log::error;
use uuid::Uuid;
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

struct Inner {
    mount: Option<Element>,
    timestamp: f64,
    controllers: Vec<Box<dyn Controller>>,
    scene: Scene,
    camera: Box<dyn Camera>,
    render: WebGL2Render,
    standard_pipeline: StandardPipeline,
    picking_pipeline: PickingPipeline,

    render_loop: Option<Closure<dyn FnMut(f64)>>,
    render_next: bool,
    stopping_render_loop: bool,
    stop_render_loop_when_error: bool,

    entities_changed_listener: Option<Uuid>,
}

#[derive(Clone)]
pub struct Viewer {
    marker: Rc<()>,
    inner: *mut Inner,
}

impl Viewer {
    pub fn new<C>(scene: Scene, camera: C) -> Result<Self, Error>
    where
        C: Camera + 'static,
    {
        let inner = Inner {
            mount: None,
            timestamp: 0.0,
            controllers: Vec::new(),
            scene,
            camera: Box::new(camera),
            render: WebGL2Render::new(None)?,
            standard_pipeline: StandardPipeline::new(),
            picking_pipeline: PickingPipeline::new(),
            render_loop: None,
            render_next: true,
            stopping_render_loop: false,
            stop_render_loop_when_error: true,
            entities_changed_listener: None,
        };
        let mut instance = Self {
            marker: Rc::new(()),
            inner: Box::leak(Box::new(inner)),
        };

        instance.register_event();

        Ok(instance)
    }

    #[inline]
    fn inner(&self) -> &Inner {
        unsafe { &*self.inner }
    }

    #[inline]
    fn inner_mut(&mut self) -> &mut Inner {
        unsafe { &mut *self.inner }
    }

    fn register_event(&mut self) {
        let me = self.weak();
        let listener = self
            .scene_mut()
            .entity_collection_mut()
            .changed_event()
            .on(move |_| {
                let Some(mut viewer) = me.upgrade() else {
                    return;
                };

                if viewer.inner().render_loop.is_some() {
                    viewer.should_render_next();
                } else {
                    if let Err(err) = viewer.render_frame() {
                        error!("error occurred during rendering {err}");
                    }
                }
            });
        self.inner_mut().entities_changed_listener = Some(listener);
    }

    pub fn mount(&self) -> Option<&Element> {
        self.inner().mount.as_ref()
    }

    pub fn set_mount(&mut self, mount: Option<Element>) -> Result<(), Error> {
        let inner = self.inner_mut();

        if let Some(mounting) = inner.mount.take() {
            let _ = mounting.remove_child(inner.render.canvas());
        }

        if let Some(mount) = &mount {
            mount
                .append_child(inner.render.canvas())
                .or(Err(Error::MountElementFailed))?;
        }
        inner.mount = mount;

        Ok(())
    }

    pub fn weak(&self) -> ViewerWeak {
        ViewerWeak {
            marker: Rc::downgrade(&self.marker),
            inner: self.inner,
        }
    }

    pub fn timestamp(&self) -> f64 {
        self.inner().timestamp
    }

    pub fn camera(&self) -> &dyn Camera {
        self.inner().camera.as_ref()
    }

    pub fn camera_mut(&mut self) -> &mut dyn Camera {
        self.inner_mut().camera.as_mut()
    }

    pub fn scene(&self) -> &Scene {
        &self.inner().scene
    }

    pub fn scene_mut(&mut self) -> &mut Scene {
        &mut self.inner_mut().scene
    }

    pub fn render(&self) -> &WebGL2Render {
        &self.inner().render
    }

    pub fn render_mut(&mut self) -> &mut WebGL2Render {
        &mut self.inner_mut().render
    }

    pub fn controllers(&self) -> &[Box<dyn Controller>] {
        &self.inner().controllers
    }

    pub fn add_controller<C>(&mut self, mut controller: C)
    where
        C: Controller + 'static,
    {
        controller.on_add(self);
        self.inner_mut().controllers.push(Box::new(controller));
    }

    pub fn remove_controller(&mut self, index: usize) -> Option<Box<dyn Controller>> {
        let controllers = &mut self.inner_mut().controllers;

        if index > controllers.len() - 1 {
            return None;
        }

        let mut controller = controllers.remove(index);
        controller.on_remove(self);
        Some(controller)
    }

    pub fn pick_entity(
        &mut self,
        window_position_x: i32,
        window_position_y: i32,
    ) -> Result<Option<&mut Entity>, Error> {
        let inner = self.inner_mut();
        let picking_pipeline = &mut inner.picking_pipeline;
        if picking_pipeline.is_dirty() {
            let timestamp = inner.timestamp;
            let scene = &mut inner.scene;
            let render = &mut inner.render;
            let camera = inner.camera.as_mut();
            render.render(picking_pipeline, camera, scene, timestamp)?;
        }

        picking_pipeline.pick_entity(window_position_x, window_position_y)
    }

    pub fn pick_position(
        &mut self,
        window_position_x: i32,
        window_position_y: i32,
    ) -> Result<Option<Vec3>, Error> {
        let inner = self.inner_mut();
        let picking_pipeline = &mut inner.picking_pipeline;
        if picking_pipeline.is_dirty() {
            let timestamp = inner.timestamp;
            let scene = &mut inner.scene;
            let render = &mut inner.render;
            let camera = inner.camera.as_mut();
            render.render(picking_pipeline, camera, scene, timestamp)?;
        }

        picking_pipeline.pick_position(window_position_x, window_position_y)
    }

    pub fn render_frame(&mut self) -> Result<(), Error> {
        let inner = self.inner_mut();
        inner.render_next = false;
        let result = inner.render.render(
            &mut inner.standard_pipeline,
            &mut *inner.camera,
            &mut inner.scene,
            inner.timestamp,
        );
        inner.picking_pipeline.set_dirty();

        result
    }

    pub fn should_render_next(&mut self) {
        self.inner_mut().render_next = true;
    }

    pub fn start_render_loop(&mut self) {
        if self.inner_mut().render_loop.is_some() {
            return;
        }

        let mut me = self.clone();

        self.inner_mut().stopping_render_loop = false;
        self.inner_mut().render_loop = Some(Closure::new(move |timestamp| {
            me.inner_mut().timestamp = timestamp;

            if me.inner().stopping_render_loop {
                me.inner_mut().render_loop = None;
                me.inner_mut().stopping_render_loop = false;
                return;
            }

            if me.inner().render_next {
                if let Err(err) = me.render_frame() {
                    error!("error occurred during rendering {err}");
                    if me.inner().stop_render_loop_when_error {
                        return;
                    }
                }
            }

            if let Some(render_loop) = me.inner().render_loop.as_ref() {
                request_animation_frame(render_loop);
            }
        }));
        request_animation_frame(self.inner().render_loop.as_ref().unwrap());
    }

    pub fn stop_render_loop(&mut self) {
        self.inner_mut().stopping_render_loop = true;
    }
}

impl Drop for Viewer {
    fn drop(&mut self) {
        if Rc::strong_count(&self.marker) == 1 {
            // removes controllers
            (0..self.controllers().len()).for_each(|_| {
                self.remove_controller(0);
            });
            // removes entities changed listener
            if let Some(listener) = self.inner_mut().entities_changed_listener.take() {
                self.scene_mut()
                    .entity_collection_mut()
                    .changed_event()
                    .off(&listener);
            }
            // unmount
            let _ = self.set_mount(None);

            unsafe { drop(Box::from_raw(self.inner)) }
        }
    }
}

#[derive(Clone)]
pub struct ViewerWeak {
    marker: Weak<()>,
    inner: *mut Inner,
}

impl ViewerWeak {
    pub fn upgrade(&self) -> Option<Viewer> {
        self.marker.upgrade().map(|marker| Viewer {
            marker,
            inner: self.inner,
        })
    }
}
