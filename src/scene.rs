use gl_matrix4rust::vec3::Vec3;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    camera::{universal::UniversalCamera, Camera},
    entity::collection::EntityCollection,
    light::ambient::Ambient,
    render::pp::Stuff,
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

#[wasm_bindgen]
pub struct Scene {
    active_camera: Box<dyn Camera>,
    ambient_light: Option<Box<dyn Ambient>>,
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
    pub fn new() -> Self {
        Self {
            active_camera: Self::create_camera(),
            ambient_light: None,
            entity_collection: EntityCollection::new(),
        }
    }

    // /// Constructs a new scene using initialization options.
    // pub fn new() -> Result<Self, Error> {
    //     Self::new_inner(None)
    // }

    // /// Constructs a new scene using initialization options.
    // pub fn with_options(options: SceneOptions) -> Result<Self, Error> {
    //     Self::new_inner(Some(options))
    // }

    // fn new_inner(mut options: Option<SceneOptions>) -> Result<Self, Error> {
    //     let active_camera = options
    //         .as_mut()
    //         .and_then(|opts| opts.take_camera())
    //         .unwrap_or_else(|| Self::create_camera());

    //     let scene = Self {
    //         active_camera,
    //         ambient_light: None,
    //         entity_collection: EntityCollection::new(),
    //     };

    //     Ok(scene)
    // }

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
    /// Returns root entities collection.
    pub fn entity_collection(&self) -> &EntityCollection {
        &self.entity_collection
    }

    /// Returns mutable root entities collection.
    pub fn entity_collection_mut(&mut self) -> &mut EntityCollection {
        &mut self.entity_collection
    }

    /// Returns current active camera.
    pub fn active_camera(&self) -> &dyn Camera {
        self.active_camera.as_ref()
    }

    /// Returns current active camera.
    pub fn active_camera_mut(&mut self) -> &mut dyn Camera {
        self.active_camera.as_mut()
    }

    /// Sets active camera.
    pub fn set_active_camera<C>(&mut self, camera: C)
    where
        C: Camera + 'static,
    {
        self.active_camera = Box::new(camera);
    }

    /// Returns ambient light.
    pub fn ambient_light(&self) -> Option<&dyn Ambient> {
        match self.ambient_light.as_ref() {
            Some(light) => Some(&**light),
            None => None,
        }
    }

    /// Returns mutable ambient light.
    pub fn ambient_light_mut(&mut self) -> Option<&mut dyn Ambient> {
        match self.ambient_light.as_mut() {
            Some(light) => Some(&mut **light),
            None => None,
        }
    }

    /// Sets ambient light.
    pub fn set_ambient_light<L>(&mut self, light: Option<L>)
    where
        L: Ambient + 'static,
    {
        self.ambient_light = match light {
            Some(light) => Some(Box::new(light)),
            None => None,
        };
    }
}

/// A [`Stuff`] source from [`Scene`].
pub struct SceneStuff<'a> {
    scene: &'a mut Scene,
}

impl<'a> SceneStuff<'a> {
    pub fn new(scene: &'a mut Scene) -> Self {
        Self { scene }
    }
}

impl<'a> Stuff for SceneStuff<'a> {
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

    fn ambient_light(&self) -> Option<&dyn Ambient> {
        self.scene.ambient_light()
    }
}
