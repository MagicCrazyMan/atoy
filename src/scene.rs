use gl_matrix4rust::vec3::{AsVec3, Vec3};
use log::warn;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::js_sys::Uint8Array;

use crate::{
    camera::{universal::UniversalCamera, Camera},
    entity::collection::EntityCollection,
    light::{
        ambient::Ambient,
        diffuse::{
            Diffuse, DIFFUSE_LIGHTS_UNIFORM_BLOCK_BYTES_SIZE,
            DIFFUSE_LIGHTS_UNIFORM_BLOCK_STRUCT_BYTES_SIZE_PER_LIGHT, MAX_DIFFUSE_LIGHTS,
        },
    },
    render::{
        pp::Stuff,
        webgl::buffer::{BufferDescriptor, BufferSource, BufferUsage, MemoryPolicy},
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
            BufferSource::preallocate(DIFFUSE_LIGHTS_UNIFORM_BLOCK_BYTES_SIZE as i32),
            BufferUsage::DynamicDraw,
            MemoryPolicy::Unfree,
        );

        Self {
            active_camera: Self::create_camera(),
            ambient_light: None,
            diffuse_lights: Vec::new(),
            diffuse_lights_descriptor,
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
            warn!("only {} diffuse lights accepts", MAX_DIFFUSE_LIGHTS);
            return;
        }

        let bytes_per_light = DIFFUSE_LIGHTS_UNIFORM_BLOCK_STRUCT_BYTES_SIZE_PER_LIGHT as u32;
        let count_buffer = Uint8Array::new_with_length(16);
        let light_buffer = Uint8Array::new_with_length(bytes_per_light);
        let index = self.diffuse_lights.len() as u32;
        let count = (index + 1).to_ne_bytes();
        count_buffer.set_index(0, count[0]);
        count_buffer.set_index(1, count[1]);
        count_buffer.set_index(2, count[2]);
        count_buffer.set_index(3, count[3]);

        // enabled
        light_buffer.set_index(0, 1);
        // color
        let color = light.color().to_gl_binary();
        light_buffer.set_index(16, color[0]);
        light_buffer.set_index(17, color[1]);
        light_buffer.set_index(18, color[2]);
        light_buffer.set_index(19, color[3]);
        light_buffer.set_index(20, color[4]);
        light_buffer.set_index(21, color[5]);
        light_buffer.set_index(22, color[6]);
        light_buffer.set_index(23, color[7]);
        light_buffer.set_index(24, color[8]);
        light_buffer.set_index(25, color[9]);
        light_buffer.set_index(26, color[10]);
        light_buffer.set_index(27, color[11]);
        // position
        let position = light.position().to_gl_binary();
        light_buffer.set_index(32, position[0]);
        light_buffer.set_index(33, position[1]);
        light_buffer.set_index(34, position[2]);
        light_buffer.set_index(35, position[3]);
        light_buffer.set_index(36, position[4]);
        light_buffer.set_index(37, position[5]);
        light_buffer.set_index(38, position[6]);
        light_buffer.set_index(39, position[7]);
        light_buffer.set_index(40, position[8]);
        light_buffer.set_index(41, position[9]);
        light_buffer.set_index(42, position[10]);
        light_buffer.set_index(43, position[11]);
        // attenuations
        let attenuations = light.attenuations().to_gl_binary();
        light_buffer.set_index(48, attenuations[0]);
        light_buffer.set_index(49, attenuations[1]);
        light_buffer.set_index(50, attenuations[2]);
        light_buffer.set_index(51, attenuations[3]);
        light_buffer.set_index(52, attenuations[4]);
        light_buffer.set_index(53, attenuations[5]);
        light_buffer.set_index(54, attenuations[6]);
        light_buffer.set_index(55, attenuations[7]);
        light_buffer.set_index(56, attenuations[8]);
        light_buffer.set_index(57, attenuations[9]);
        light_buffer.set_index(58, attenuations[10]);
        light_buffer.set_index(59, attenuations[11]);

        self.diffuse_lights_descriptor
            .buffer_sub_data(BufferSource::from_uint8_array(count_buffer, 0, 16), 0);
        self.diffuse_lights_descriptor.buffer_sub_data(
            BufferSource::from_uint8_array(light_buffer, 0, bytes_per_light),
            (16 + index * bytes_per_light) as i32,
        );

        self.diffuse_lights.push(Box::new(light));
    }

    /// Removes a diffuse light by index.
    pub fn remove_diffuse_light(&mut self, index: usize) -> Box<dyn Diffuse> {
        self.diffuse_lights.remove(index)
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
        log::info!("3333");
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
}
