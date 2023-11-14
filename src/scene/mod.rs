use std::borrow::Cow;

use gl_matrix4rust::vec3::Vec3;
use serde::Deserialize;
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsError};
use web_sys::{HtmlCanvasElement, HtmlElement};

use crate::{
    camera::{perspective::PerspectiveCamera, Camera},
    document,
    entity::Entity,
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
#[derive(Default, Deserialize)]
pub struct SceneOptions<'a> {
    /// Mounts target.
    pub mount: Option<Cow<'a, str>>,
}

#[wasm_bindgen]
pub struct Scene {
    mount: Option<HtmlElement>,
    canvas: HtmlCanvasElement,
    active_camera: Box<dyn Camera>,
    root_entity: Box<Entity>,
}

#[wasm_bindgen]
impl Scene {
    #[wasm_bindgen(constructor)]
    pub fn new_constructor(options: Option<SceneOptionsObject>) -> Result<Scene, JsError> {
        let options = match options {
            Some(options) => serde_wasm_bindgen::from_value::<SceneOptions>(options.obj)
                .or(Err(JsError::new("failed to parse scene options")))?,
            None => SceneOptions::default(),
        };

        Self::with_options(options)
    }
}

impl Scene {
    /// Constructs a new scene using initialization options.
    pub fn new() -> Result<Self, JsError> {
        set_panic_hook();

        let canvas = Self::create_canvas()?;
        let active_camera = Self::create_camera(&canvas);
        let mut scene = Self {
            mount: None,
            canvas,
            active_camera,
            root_entity: Entity::new_boxed(),
        };

        // init mount target
        Self::set_mount(&mut scene, None)?;

        Ok(scene)
    }

    /// Constructs a new scene using initialization options.
    pub fn with_options(options: SceneOptions) -> Result<Self, JsError> {
        set_panic_hook();

        let canvas = Self::create_canvas()?;
        let active_camera = Self::create_camera(&canvas);
        let mut scene = Self {
            mount: None,
            canvas,
            active_camera,
            root_entity: Entity::new_boxed(),
        };

        // init mount target
        Self::set_mount(&mut scene, options.mount)?;

        Ok(scene)
    }

    fn create_canvas() -> Result<HtmlCanvasElement, JsError> {
        let canvas = document()
            .create_element("canvas")
            .ok()
            .and_then(|ele| ele.dyn_into::<HtmlCanvasElement>().ok())
            .ok_or(JsError::new("failed to create canvas"))?;

        canvas.style().set_css_text("width: 100%; height: 100%;");
        Ok(canvas)
    }

    fn create_camera(canvas: &HtmlCanvasElement) -> Box<dyn Camera> {
        Box::new(PerspectiveCamera::new(
            Vec3::from_values(0.0, 0.0, 2.0),
            Vec3::new(),
            Vec3::from_values(0.0, 1.0, 0.0),
            60.0f32.to_radians(),
            canvas.width() as f32 / canvas.height() as f32,
            0.5,
            None,
        ))
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
    pub fn set_mount(&mut self, mount: Option<Cow<'_, str>>) -> Result<(), JsError> {
        if let Some(mount) = mount {
            if !mount.is_empty() {
                // gets and sets mount target using `document.getElementById`
                let mount = document()
                    .get_element_by_id(&mount)
                    .and_then(|ele| ele.dyn_into::<HtmlElement>().ok())
                    .ok_or(JsError::new(&format!(
                        "mount element with id `{mount}` not found"
                    )))?;

                // mounts canvas to target (creates new if not exists)
                mount.append_child(&self.canvas).unwrap();
                self.canvas.set_width(mount.client_width() as u32);
                self.canvas.set_height(mount.client_height() as u32);

                self.mount = Some(mount);

                return Ok(());
            }
        }

        // for all other situations, removes canvas from mount target
        self.canvas.remove();
        Ok(())
    }

    /// Gets canvas element.
    pub fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }

    /// Gets root entity.
    pub fn root_entity(&self) -> &Entity {
        &self.root_entity
    }

    /// Gets mutable root entity.
    pub fn root_entity_mut(&mut self) -> &mut Box<Entity> {
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
}
