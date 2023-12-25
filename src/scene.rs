use gl_matrix4rust::vec3::{AsVec3, Vec3};
use log::warn;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::js_sys::Float32Array;

use crate::{
    camera::{universal::UniversalCamera, Camera},
    entity::collection::EntityCollection,
    light::{
        ambient::Ambient,
        diffuse::{Diffuse, MAX_DIFFUSE_LIGHTS},
        specular::{Specular, MAX_SPECULAR_LIGHTS},
    },
    render::{
        pp::Stuff,
        webgl::{
            buffer::{BufferDescriptor, BufferSource, BufferUsage, MemoryPolicy},
            conversion::{GLsizeiptr, GLuint},
        },
    },
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
    diffuse_lights: Vec<Box<dyn Diffuse>>,
    diffuse_lights_descriptor: BufferDescriptor,
    specular_lights: Vec<Box<dyn Specular>>,
    specular_lights_descriptor: BufferDescriptor,
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
        let diffuse_lights_descriptor = BufferDescriptor::with_memory_policy(
            BufferSource::preallocate(64 * MAX_DIFFUSE_LIGHTS as GLsizeiptr),
            BufferUsage::DynamicDraw,
            MemoryPolicy::Unfree,
        );
        let specular_lights_descriptor = BufferDescriptor::with_memory_policy(
            BufferSource::preallocate(64 * MAX_SPECULAR_LIGHTS as GLsizeiptr),
            BufferUsage::DynamicDraw,
            MemoryPolicy::Unfree,
        );

        Self {
            active_camera: Self::create_camera(),
            ambient_light: None,
            diffuse_lights: Vec::new(),
            diffuse_lights_descriptor,
            specular_lights: Vec::new(),
            specular_lights_descriptor,
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

    /// Adds a diffuse light.
    pub fn add_diffuse_light<L>(&mut self, light: L)
    where
        L: Diffuse + 'static,
    {
        if self.diffuse_lights.len() == MAX_DIFFUSE_LIGHTS {
            warn!("only {} diffuse lights available", MAX_DIFFUSE_LIGHTS);
            return;
        }

        let buffer = Float32Array::new_with_length(12);
        let index = self.specular_lights.len() as u32;
        // color
        let color = light.color().to_gl();
        buffer.set_index(0, color[0]);
        buffer.set_index(1, color[1]);
        buffer.set_index(2, color[2]);
        buffer.set_index(3, 0.0);
        // position
        let position = light.position().to_gl();
        buffer.set_index(4, position[0]);
        buffer.set_index(5, position[1]);
        buffer.set_index(6, position[2]);
        buffer.set_index(7, 0.0);
        // attenuations
        let attenuations = light.attenuations().to_gl();
        buffer.set_index(8, attenuations[0]);
        buffer.set_index(9, attenuations[1]);
        buffer.set_index(10, attenuations[2]);
        // enabled
        buffer.set_index(11, 1.0);

        self.diffuse_lights_descriptor.buffer_sub_data(
            BufferSource::from_float32_array(buffer, 0, 12),
            (index * 48) as GLsizeiptr,
        );

        self.diffuse_lights.push(Box::new(light));
    }

    /// Removes a diffuse light by index.
    pub fn remove_diffuse_light(&mut self, index: usize) -> Box<dyn Diffuse> {
        self.diffuse_lights.remove(index)
    }

    /// Adds a specular light.
    pub fn add_specular_light<L>(&mut self, light: L)
    where
        L: Specular + 'static,
    {
        if self.specular_lights.len() == MAX_SPECULAR_LIGHTS {
            warn!("only {} specular lights available", MAX_SPECULAR_LIGHTS);
            return;
        }

        let buffer = Float32Array::new_with_length(16);
        let index = self.specular_lights.len() as u32;
        // color
        let color = light.color().to_gl();
        buffer.set_index(0, color[0]);
        buffer.set_index(1, color[1]);
        buffer.set_index(2, color[2]);
        buffer.set_index(3, 0.0);
        // position
        let position = light.position().to_gl();
        buffer.set_index(4, position[0]);
        buffer.set_index(5, position[1]);
        buffer.set_index(6, position[2]);
        buffer.set_index(7, 0.0);
        // attenuations
        let attenuations = light.attenuations().to_gl();
        buffer.set_index(8, attenuations[0]);
        buffer.set_index(9, attenuations[1]);
        buffer.set_index(10, attenuations[2]);
        // shininess
        let shininess = light.shininess();
        buffer.set_index(11, shininess);
        // enabled
        buffer.set_index(12, 1.0);

        self.specular_lights_descriptor.buffer_sub_data(
            BufferSource::from_float32_array(buffer, 0, 16),
            (index * 64) as GLsizeiptr,
        );

        self.specular_lights.push(Box::new(light));
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

    fn enable_lighting(&self) -> bool {
        true
    }

    fn diffuse_lights(&self) -> Vec<&dyn Diffuse> {
        self.scene
            .diffuse_lights
            .iter()
            .map(|light| light.as_ref())
            .collect()
    }

    fn ambient_light(&self) -> Option<&dyn Ambient> {
        self.scene.ambient_light()
    }

    fn diffuse_lights_descriptor(&self) -> BufferDescriptor {
        self.scene.diffuse_lights_descriptor.clone()
    }

    fn specular_lights(&self) -> Vec<&dyn Specular> {
        self.scene
            .specular_lights
            .iter()
            .map(|light| light.as_ref())
            .collect()
    }

    fn specular_lights_descriptor(&self) -> BufferDescriptor {
        self.scene.specular_lights_descriptor.clone()
    }
}
