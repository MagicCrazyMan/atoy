use gl_matrix4rust::vec3::Vec3;
use log::warn;

use crate::{
    entity::collection::EntityCollection,
    light::{
        ambient_light::AmbientLight,
        area_light::{AreaLight, MAX_AREA_LIGHTS},
        directional_light::{DirectionalLight, MAX_DIRECTIONAL_LIGHTS},
        point_light::{PointLight, MAX_POINT_LIGHTS},
        spot_light::{SpotLight, MAX_SPOT_LIGHTS},
    },
};

pub struct Scene {
    entity_collection: EntityCollection,

    lighting_enabled: bool,
    light_attenuations: Vec3,
    ambient_light: Option<AmbientLight>,
    point_lights: Vec<PointLight>,
    directional_lights: Vec<DirectionalLight>,
    spot_lights: Vec<SpotLight>,
    area_lights: Vec<AreaLight>,
}

impl Scene {
    /// Constructs a new scene using initialization options.
    pub fn new() -> Self {
        Self {
            entity_collection: EntityCollection::new(),

            lighting_enabled: true,
            light_attenuations: Vec3::from_values(0.0, 1.0, 0.0),
            ambient_light: None,
            point_lights: Vec::new(),
            directional_lights: Vec::new(),
            spot_lights: Vec::new(),
            area_lights: Vec::new(),
        }
    }

    /// Returns root entities collection.
    pub fn entity_collection(&self) -> &EntityCollection {
        &self.entity_collection
    }

    /// Returns mutable root entities collection.
    pub fn entity_collection_mut(&mut self) -> &mut EntityCollection {
        &mut self.entity_collection
    }

    /// Returns `true` if enable lighting.
    /// Diffuse color of material used directly if lighting is disabled.
    pub fn lighting_enabled(&self) -> bool {
        self.lighting_enabled
    }

    /// Enables lighting.
    pub fn enable_lighting(&mut self) {
        self.lighting_enabled = true;
    }

    /// Disables lighting.
    pub fn disable_lighting(&mut self) {
        self.lighting_enabled = false;
    }

    /// Returns ambient light.
    pub fn ambient_light(&self) -> Option<&AmbientLight> {
        self.ambient_light.as_ref()
    }

    /// Returns mutable ambient light.
    pub fn ambient_light_mut(&mut self) -> Option<&mut AmbientLight> {
        self.ambient_light.as_mut()
    }

    /// Sets ambient light.
    pub fn set_ambient_light(&mut self, light: Option<AmbientLight>) {
        self.ambient_light = light;
    }

    /// Returns lighting attenuation.
    pub fn light_attenuations(&self) -> &Vec3 {
        &self.light_attenuations
    }

    /// Sets lighting attenuation.
    pub fn set_light_attenuations(&mut self, attenuations: Vec3) {
        self.light_attenuations = attenuations;
    }

    /// Adds a point light.
    pub fn add_point_light(&mut self, light: PointLight) {
        if self.point_lights.len() == MAX_POINT_LIGHTS {
            warn!(
                "only {} point lights are available, ignored",
                MAX_POINT_LIGHTS
            );
            return;
        }

        self.point_lights.push(light);
    }

    /// Removes a point light by index.
    pub fn remove_point_light(&mut self, index: usize) -> Option<PointLight> {
        if index < self.point_lights.len() {
            return None;
        }

        Some(self.point_lights.remove(index))
    }

    /// Returns point lights.
    pub fn point_lights(&self) -> &[PointLight] {
        &self.point_lights
    }

    /// Returns a point light by index.
    pub fn point_light(&self, index: usize) -> Option<&PointLight> {
        self.point_lights.get(index)
    }

    /// Returns a mutable point light by index.
    pub fn point_light_mut(&mut self, index: usize) -> Option<&mut PointLight> {
        self.point_lights.get_mut(index)
    }

    /// Adds a directional light.
    pub fn add_directional_light(&mut self, light: DirectionalLight) {
        if self.directional_lights.len() == MAX_DIRECTIONAL_LIGHTS {
            warn!(
                "only {} directional lights are available, ignored",
                MAX_DIRECTIONAL_LIGHTS
            );
            return;
        }

        self.directional_lights.push(light);
    }

    /// Removes a directional light by index.
    pub fn remove_directional_light(&mut self, index: usize) -> Option<DirectionalLight> {
        if index < self.directional_lights.len() {
            return None;
        }

        Some(self.directional_lights.remove(index))
    }

    /// Returns directional lights.
    pub fn directional_lights(&self) -> &[DirectionalLight] {
        &self.directional_lights
    }

    /// Returns a directional light by index.
    pub fn directional_light(&self, index: usize) -> Option<&DirectionalLight> {
        self.directional_lights.get(index)
    }

    /// Returns a mutable directional light by index.
    pub fn directional_light_mut(&mut self, index: usize) -> Option<&mut DirectionalLight> {
        self.directional_lights.get_mut(index)
    }

    /// Adds a spot light.
    pub fn add_spot_light(&mut self, light: SpotLight) {
        if self.spot_lights.len() == MAX_SPOT_LIGHTS {
            warn!(
                "only {} spot lights are available, ignored",
                MAX_SPOT_LIGHTS
            );
            return;
        }

        self.spot_lights.push(light);
    }

    /// Removes a spot light by index.
    pub fn remove_spot_light(&mut self, index: usize) -> Option<SpotLight> {
        if index < self.spot_lights.len() {
            return None;
        }

        Some(self.spot_lights.remove(index))
    }

    /// Returns spot lights.
    pub fn spot_lights(&self) -> &[SpotLight] {
        &self.spot_lights
    }

    /// Returns a spot light by index.
    pub fn spot_light(&self, index: usize) -> Option<&SpotLight> {
        self.spot_lights.get(index)
    }

    /// Returns a mutable spot light by index.
    pub fn spot_light_mut(&mut self, index: usize) -> Option<&mut SpotLight> {
        self.spot_lights.get_mut(index)
    }

    /// Adds a area light.
    pub fn add_area_light(&mut self, light: AreaLight) {
        if self.spot_lights.len() == MAX_AREA_LIGHTS {
            warn!(
                "only {} area lights are available, ignored",
                MAX_AREA_LIGHTS
            );
            return;
        }

        self.area_lights.push(light);
    }

    /// Removes a area light by index.
    pub fn remove_area_light(&mut self, index: usize) -> Option<AreaLight> {
        if index < self.area_lights.len() {
            return None;
        }

        Some(self.area_lights.remove(index))
    }

    /// Returns area lights.
    pub fn area_lights(&self) -> &[AreaLight] {
        &self.area_lights
    }

    /// Returns a area light by index.
    pub fn area_light(&self, index: usize) -> Option<&AreaLight> {
        self.area_lights.get(index)
    }

    /// Returns a mutable area light by index.
    pub fn area_light_mut(&mut self, index: usize) -> Option<&mut AreaLight> {
        self.area_lights.get_mut(index)
    }
}
