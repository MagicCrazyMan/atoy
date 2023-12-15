use gl_matrix4rust::vec3::{AsVec3, Vec3};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    camera::{universal::UniversalCamera, Camera},
    entity::collection::EntityCollection,
    error::Error,
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
    /// Default camera
    camera: Option<Box<dyn Camera>>,
}

impl SceneOptions {
    pub fn new() -> Self {
        Self { camera: None }
    }

    pub fn with_camera<C: Camera + 'static>(mut self, camera: C) -> Self {
        self.camera = Some(Box::new(camera));
        self
    }

    pub fn without_camera(mut self) -> Self {
        self.camera = None;
        self
    }

    pub fn camera(&self) -> Option<&Box<dyn Camera>> {
        self.camera.as_ref()
    }

    fn take_camera(&mut self) -> Option<Box<dyn Camera>> {
        self.camera.take()
    }
}

#[wasm_bindgen]
pub struct Scene {
    active_camera: Box<dyn Camera>,
    entity_collection: EntityCollection,
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
        let active_camera = options
            .as_mut()
            .and_then(|opts| opts.take_camera())
            .unwrap_or_else(|| Self::create_camera());

        let scene = Self {
            active_camera,
            entity_collection: EntityCollection::new(),
        };

        Ok(scene)
    }

    fn create_camera() -> Box<dyn Camera> {
        Box::new(UniversalCamera::new(
            Vec3::from_values(0.0, 0.0, 2.0),
            Vec3::new(),
            Vec3::from_values(0.0, 1.0, 0.0),
            60.0f64.to_radians(),
            1.0,
            0.5,
            None,
        ))
    }
}

impl Scene {
    /// Gets root root entities collection.
    pub fn entity_collection(&self) -> &EntityCollection {
        &self.entity_collection
    }

    /// Gets mutable root entities collection.
    pub fn entity_collection_mut(&mut self) -> &mut EntityCollection {
        &mut self.entity_collection
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
