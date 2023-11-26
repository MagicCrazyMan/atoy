use gl_matrix4rust::vec3::{AsVec3, Vec3};
use wasm_bindgen::{closure::Closure, prelude::wasm_bindgen, JsCast};
use wasm_bindgen_test::console_log;
use web_sys::{HtmlCanvasElement, HtmlElement, ResizeObserver, ResizeObserverEntry};

use crate::{
    camera::{perspective::PerspectiveCamera, Camera},
    document,
    entity::Entity,
    error::Error,
    utils::set_panic_hook,
};

#[wasm_bindgen(typescript_custom_section)]
const SCENE_OPTIONS_TYPE: &'static str = r#"
export type SceneOptions {
    /**
     * Mounts target.
     */
    mount?: string;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "SceneOptions")]
    pub type SceneOptionsObject;
}

/// Scene options
#[derive(Default)]
pub struct SceneOptions {
    /// Mounts target.
    mount: Option<String>,
    /// Default camera
    camera: Option<Box<dyn Camera>>,
}

impl SceneOptions {
    pub fn new() -> Self {
        Self {
            mount: None,
            camera: None,
        }
    }

    pub fn with_mount<S: Into<String>>(mut self, mount: S) -> Self {
        self.mount = Some(mount.into());
        self
    }

    pub fn without_mount(mut self) -> Self {
        self.mount = None;
        self
    }

    pub fn with_default_camera<C: Camera + 'static>(mut self, camera: C) -> Self {
        self.camera = Some(Box::new(camera));
        self
    }

    pub fn without_default_camera(mut self) -> Self {
        self.camera = None;
        self
    }

    pub fn mount(&self) -> Option<&str> {
        self.mount.as_ref().map(|x| x.as_str())
    }

    pub fn camera(&self) -> Option<&Box<dyn Camera>> {
        self.camera.as_ref()
    }

    fn take_mount(&mut self) -> Option<String> {
        self.mount.take()
    }

    fn take_camera(&mut self) -> Option<Box<dyn Camera>> {
        self.camera.take()
    }
}

#[wasm_bindgen]
pub struct Scene {
    mount: Option<HtmlElement>,
    canvas: HtmlCanvasElement,
    active_camera: Box<dyn Camera>,
    root_entity: Entity,
    // require for storing callback closure function
    resize_observer: (ResizeObserver, Closure<dyn FnMut(Vec<ResizeObserverEntry>)>),
}

// #[wasm_bindgen]
// impl Scene {
//     #[wasm_bindgen(constructor)]
//     pub fn new_constructor(options: Option<SceneOptionsObject>) -> Result<Scene, Error> {
//         let options = match options {
//             Some(options) => serde_wasm_bindgen::from_value::<SceneOptions>(options.obj)
//                 .or(Err(Error::ParseObjectFailure))?,
//             None => SceneOptions::default(),
//         };

//         Self::with_options(options)
//     }
// }

impl Scene {
    /// Constructs a new scene using initialization options.
    pub fn new() -> Result<Self, Error> {
        Self::new_inner(None)
    }

    /// Constructs a new scene using initialization options.
    pub fn with_options(options: SceneOptions) -> Result<Self, Error> {
        Self::new_inner(Some(options))
    }

    fn new_inner(mut options: Option<SceneOptions>) -> Result<Self, Error> {
        set_panic_hook();

        let canvas = Self::create_canvas()?;
        let mut active_camera = options
            .as_mut()
            .and_then(|opts| opts.take_camera())
            .unwrap_or(Self::create_camera(&canvas));
        let _resize_observer = Self::observer_canvas_size(&canvas, &mut active_camera);

        let mut scene = Self {
            mount: None,
            canvas,
            active_camera,
            resize_observer: _resize_observer,
            root_entity: Entity::new(),
        };

        // init mount target
        Self::set_mount(&mut scene, options.as_ref().and_then(|opts| opts.mount()))?;

        Ok(scene)
    }

    fn create_canvas() -> Result<HtmlCanvasElement, Error> {
        let canvas = document()
            .create_element("canvas")
            .ok()
            .and_then(|ele| ele.dyn_into::<HtmlCanvasElement>().ok())
            .ok_or(Error::CreateCanvasFailure)?;

        canvas.style().set_css_text("width: 100%; height: 100%;");
        Ok(canvas)
    }

    fn create_camera(canvas: &HtmlCanvasElement) -> Box<dyn Camera> {
        Box::new(PerspectiveCamera::new(
            Vec3::from_values(0.0, 0.0, 2.0),
            Vec3::new(),
            Vec3::from_values(0.0, 1.0, 0.0),
            60.0f64.to_radians(),
            canvas.width() as f64 / canvas.height() as f64,
            0.5,
            None,
        ))
    }

    fn observer_canvas_size(
        canvas: &HtmlCanvasElement,
        camera: &mut Box<dyn Camera>,
    ) -> (ResizeObserver, Closure<dyn FnMut(Vec<ResizeObserverEntry>)>) {
        // create observer observing size change event of canvas
        let camera_ptr: *mut dyn Camera = camera.as_mut();
        let resize_observer_callback = Closure::new(move |entries: Vec<ResizeObserverEntry>| {
            // should have only one entry
            let Some(target) = entries.get(0).map(|entry| entry.target()) else {
                return;
            };
            let Some(canvas) = target.dyn_ref::<HtmlCanvasElement>() else {
                return;
            };

            canvas.set_width(canvas.client_width() as u32);
            canvas.set_height(canvas.client_height() as u32);

            unsafe {
                let camera = &mut *camera_ptr;
                if let Some(camera) = camera.as_any_mut().downcast_mut::<PerspectiveCamera>() {
                    camera.set_aspect(canvas.width() as f64 / canvas.height() as f64);
                }
            }
        });

        let resize_observer =
            ResizeObserver::new(resize_observer_callback.as_ref().unchecked_ref()).unwrap();
        resize_observer.observe(canvas);

        (resize_observer, resize_observer_callback)
    }
}

impl Scene {
    /// Gets mount target.
    pub fn mount(&self) -> Option<&HtmlElement> {
        match &self.mount {
            Some(mount) => Some(mount),
            None => None,
        }
    }

    /// Sets the mount target.
    pub fn set_mount(&mut self, mount: Option<&str>) -> Result<(), Error> {
        if let Some(mount) = mount {
            if !mount.is_empty() {
                // gets and sets mount target using `document.getElementById`
                let mount = document()
                    .get_element_by_id(&mount)
                    .and_then(|ele| ele.dyn_into::<HtmlElement>().ok())
                    .ok_or(Error::MountElementNotFound)?;

                // mounts canvas to target (creates new if not exists)
                mount.append_child(&self.canvas).unwrap();
                let width = mount.client_width() as u32;
                let height = mount.client_height() as u32;
                self.canvas.set_width(width);
                self.canvas.set_height(height);

                // try set aspect if camera is a PerspectiveCamera
                if let Some(camera) = self
                    .active_camera
                    .as_any_mut()
                    .downcast_mut::<PerspectiveCamera>()
                {
                    camera.set_aspect(width as f64 / height as f64);
                };

                self.mount = Some(mount);

                return Ok(());
            }
        }

        // for all other situations, removes canvas from mount target
        self.canvas.remove();
        self.mount = None;
        Ok(())
    }

    /// Gets canvas element.
    pub fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }

    /// Gets root entity.
    pub(crate) fn root_entity(&self) -> &Entity {
        &self.root_entity
    }

    /// Gets mutable root entity.
    pub(crate) fn root_entity_mut(&mut self) -> &mut Entity {
        &mut self.root_entity
    }

    /// Gets mutable root entity.
    pub(crate) fn root_entity_raw(&mut self) -> *mut Entity {
        &mut self.root_entity
    }

    /// Gets current active camera.
    pub fn active_camera(&self) -> &dyn Camera {
        self.active_camera.as_ref()
    }

    /// Gets current active camera.
    pub fn active_camera_mut(&mut self) -> &mut dyn Camera {
        self.active_camera.as_mut()
    }

    pub(crate) fn active_camera_raw(&mut self) -> *mut dyn Camera {
        self.active_camera.as_mut()
    }
}

impl Drop for Scene {
    fn drop(&mut self) {
        console_log!("scene dropping");
        // cleanups observers
        self.resize_observer.0.disconnect();
        self.canvas.remove();
        console_log!("scene dropped");
    }
}
