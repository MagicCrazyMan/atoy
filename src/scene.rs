use serde::Deserialize;
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsError};
use web_sys::{HtmlCanvasElement, HtmlElement};

use crate::{document, entity::Entity, utils::set_panic_hook};

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
    pub type SceneOptionsType;
}

/// Scene options
#[derive(Default, Deserialize)]
pub struct SceneOptions {
    /// Mounts target.
    pub mount: Option<String>,
}

#[wasm_bindgen]
pub struct Scene {
    mount: Option<HtmlElement>,
    canvas: HtmlCanvasElement,
    root_entity: Box<Entity>,
}

#[wasm_bindgen]
impl Scene {
    /// Constructs a new scene using initialization options.
    #[wasm_bindgen(constructor)]
    pub fn new(options: Option<SceneOptionsType>) -> Result<Scene, JsError> {
        set_panic_hook();

        let options = match options {
            Some(options) => serde_wasm_bindgen::from_value::<SceneOptions>(options.obj)
                .or(Err(JsError::new("failed to parse scene options")))?,
            None => SceneOptions::default(),
        };

        let mut scene = Self {
            mount: None,
            canvas: Self::create_canvas()?,
            root_entity: Entity::new_boxed(),
        };

        // init mount target
        Self::set_mount(&mut scene, options.mount)?;

        Ok(scene)
    }

    fn create_canvas() -> Result<HtmlCanvasElement, JsError> {
        document()
            .create_element("canvas")
            .ok()
            .and_then(|ele| ele.dyn_into::<HtmlCanvasElement>().ok())
            .ok_or(JsError::new("failed to create canvas"))
    }

    /// Sets the mount target.
    pub fn set_mount(&mut self, mount: Option<String>) -> Result<(), JsError> {
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

                self.mount = Some(mount);

                return Ok(());
            }
        }

        // for all other situations, removes canvas from mount target
        &self.canvas.remove();
        Ok(())
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

    /// Gets canvas element.
    pub fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }

    /// Gets root entity.
    pub fn root_entity(&self) -> &Entity {
        &self.root_entity
    }
}
