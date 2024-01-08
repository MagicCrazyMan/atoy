pub mod collection;

use std::{any::Any, collections::HashMap};

use gl_matrix4rust::mat4::{AsMat4, Mat4};
use uuid::Uuid;

use crate::{
    bounding::CullingBoundingVolume,
    event::EventAgency,
    geometry::Geometry,
    material::Material,
    render::webgl::{
        attribute::AttributeValue,
        uniform::{UniformBlockValue, UniformValue},
    },
};

pub struct Entity {
    id: Uuid,
    model_matrix: Mat4,
    parent_model_matrix: Option<Mat4>,
    attribute_values: HashMap<String, AttributeValue>,
    uniform_values: HashMap<String, UniformValue>,
    uniform_blocks_values: HashMap<String, UniformBlockValue>,
    properties: HashMap<String, Box<dyn Any>>,
    geometry: Option<Box<dyn Geometry>>,
    material: Option<Box<dyn Material>>,
    dirty: bool,
    compose_model_matrix: Mat4,
    compose_normal_matrix: Mat4,
    bounding: Option<CullingBoundingVolume>,

    entity_changed_event: EventAgency<EntityChangedEvent>,
    geometry_event: EventAgency<EntityChangedEvent>,
    material_event: EventAgency<EntityChangedEvent>,
    model_matrix_event: EventAgency<EntityChangedEvent>,
    add_attribute_value_event: EventAgency<EntityChangedEvent>,
    add_uniform_value_event: EventAgency<EntityChangedEvent>,
    add_uniform_block_value_event: EventAgency<EntityChangedEvent>,
    add_property_event: EventAgency<EntityChangedEvent>,
    remove_attribute_value_event: EventAgency<EntityChangedEvent>,
    remove_uniform_value_event: EventAgency<EntityChangedEvent>,
    remove_uniform_block_value_event: EventAgency<EntityChangedEvent>,
    remove_property_event: EventAgency<EntityChangedEvent>,
    clear_attributes_value_event: EventAgency<EntityChangedEvent>,
    clear_uniform_values_event: EventAgency<EntityChangedEvent>,
    clear_uniform_block_values_event: EventAgency<EntityChangedEvent>,
    clear_properties_event: EventAgency<EntityChangedEvent>,
}

impl Entity {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            model_matrix: Mat4::new_identity(),
            parent_model_matrix: None,
            attribute_values: HashMap::new(),
            uniform_values: HashMap::new(),
            uniform_blocks_values: HashMap::new(),
            properties: HashMap::new(),
            geometry: None,
            material: None,
            dirty: true,
            compose_model_matrix: Mat4::new_identity(),
            compose_normal_matrix: Mat4::new_identity(),
            bounding: None,

            entity_changed_event: EventAgency::new(),
            geometry_event: EventAgency::new(),
            material_event: EventAgency::new(),
            model_matrix_event: EventAgency::new(),
            add_attribute_value_event: EventAgency::new(),
            add_uniform_value_event: EventAgency::new(),
            add_uniform_block_value_event: EventAgency::new(),
            add_property_event: EventAgency::new(),
            remove_attribute_value_event: EventAgency::new(),
            remove_uniform_value_event: EventAgency::new(),
            remove_uniform_block_value_event: EventAgency::new(),
            remove_property_event: EventAgency::new(),
            clear_attributes_value_event: EventAgency::new(),
            clear_uniform_values_event: EventAgency::new(),
            clear_uniform_block_values_event: EventAgency::new(),
            clear_properties_event: EventAgency::new(),
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn bounding(&self) -> Option<&CullingBoundingVolume> {
        self.bounding.as_ref()
    }

    pub fn model_matrix(&self) -> &Mat4 {
        &self.model_matrix
    }

    pub fn compose_model_matrix(&self) -> &Mat4 {
        &self.compose_model_matrix
    }

    pub fn compose_normal_matrix(&self) -> &Mat4 {
        &self.compose_normal_matrix
    }

    pub fn attribute_values(&self) -> &HashMap<String, AttributeValue> {
        &self.attribute_values
    }

    pub fn uniform_values(&self) -> &HashMap<String, UniformValue> {
        &self.uniform_values
    }

    pub fn uniform_blocks_values(&self) -> &HashMap<String, UniformBlockValue> {
        &self.uniform_blocks_values
    }

    pub fn properties(&self) -> &HashMap<String, Box<dyn Any>> {
        &self.properties
    }

    pub fn geometry(&self) -> Option<&dyn Geometry> {
        self.geometry.as_deref()
    }

    pub fn geometry_mut(&mut self) -> Option<&mut dyn Geometry> {
        match &mut self.geometry {
            Some(geometry) => {
                self.dirty = true;
                Some(&mut **geometry)
            }
            None => None,
        }
    }

    pub fn material(&self) -> Option<&dyn Material> {
        self.material.as_deref()
    }

    pub fn material_mut(&mut self) -> Option<&mut dyn Material> {
        match &mut self.material {
            Some(material) => {
                self.dirty = true;
                Some(&mut **material)
            }
            None => None,
        }
    }

    pub fn set_model_matrix(&mut self, model_matrix: Mat4) {
        self.model_matrix = model_matrix;
        self.dirty = true;
        self.model_matrix_event.raise(EntityChangedEvent::new(self));
        self.entity_changed_event
            .raise(EntityChangedEvent::new(self));
    }

    pub fn set_geometry<G>(&mut self, geometry: Option<G>)
    where
        G: Geometry + 'static,
    {
        self.geometry = geometry.map(|geometry| Box::new(geometry) as Box<dyn Geometry>);
        self.dirty = true;
        self.geometry_event.raise(EntityChangedEvent::new(self));
        self.entity_changed_event
            .raise(EntityChangedEvent::new(self));
    }

    pub fn set_material<M>(&mut self, material: Option<M>)
    where
        M: Material + 'static,
    {
        self.material = material.map(|material| Box::new(material) as Box<dyn Material>);
        self.dirty = true;
        self.material_event.raise(EntityChangedEvent::new(self));
        self.entity_changed_event
            .raise(EntityChangedEvent::new(self));
    }

    pub fn add_attribute_value<S>(&mut self, name: S, value: AttributeValue)
    where
        S: Into<String>,
    {
        let name = name.into();
        self.attribute_values.insert(name.clone(), value);
        self.add_attribute_value_event
            .raise(EntityChangedEvent::new(self));
        self.entity_changed_event
            .raise(EntityChangedEvent::new(self));
    }

    pub fn add_uniform_value<S>(&mut self, name: S, value: UniformValue)
    where
        S: Into<String>,
    {
        let name = name.into();
        self.uniform_values.insert(name.clone(), value);
        self.add_uniform_block_value_event
            .raise(EntityChangedEvent::new(self));
        self.entity_changed_event
            .raise(EntityChangedEvent::new(self));
    }

    pub fn add_uniform_block_value<S>(&mut self, name: S, value: UniformBlockValue)
    where
        S: Into<String>,
    {
        let name = name.into();
        self.uniform_blocks_values.insert(name.clone(), value);
        self.add_uniform_block_value_event
            .raise(EntityChangedEvent::new(self));
        self.entity_changed_event
            .raise(EntityChangedEvent::new(self));
    }

    pub fn add_property<S, T>(&mut self, name: S, value: T)
    where
        S: Into<String>,
        T: 'static,
    {
        let name = name.into();
        self.properties.insert(name.clone(), Box::new(value));
        self.add_property_event.raise(EntityChangedEvent::new(self));
        self.entity_changed_event
            .raise(EntityChangedEvent::new(self));
    }

    pub fn remove_attribute_value(&mut self, name: &str) -> Option<(String, AttributeValue)> {
        if let Some(entry) = self.attribute_values.remove_entry(name) {
            self.remove_attribute_value_event
                .raise(EntityChangedEvent::new(self));
            self.entity_changed_event
                .raise(EntityChangedEvent::new(self));
            Some(entry)
        } else {
            None
        }
    }

    pub fn remove_uniform_value(&mut self, name: &str) -> Option<(String, UniformValue)> {
        if let Some(entry) = self.uniform_values.remove_entry(name) {
            self.remove_uniform_value_event
                .raise(EntityChangedEvent::new(self));
            self.entity_changed_event
                .raise(EntityChangedEvent::new(self));
            Some(entry)
        } else {
            None
        }
    }

    pub fn remove_uniform_block_value(
        &mut self,
        name: &str,
    ) -> Option<(String, UniformBlockValue)> {
        if let Some(entry) = self.uniform_blocks_values.remove_entry(name) {
            self.remove_uniform_block_value_event
                .raise(EntityChangedEvent::new(self));
            self.entity_changed_event
                .raise(EntityChangedEvent::new(self));
            Some(entry)
        } else {
            None
        }
    }

    pub fn remove_property(&mut self, name: &str) -> Option<(String, Box<dyn Any>)> {
        if let Some(entry) = self.properties.remove_entry(name) {
            self.remove_property_event
                .raise(EntityChangedEvent::new(self));
            self.entity_changed_event
                .raise(EntityChangedEvent::new(self));
            Some(entry)
        } else {
            None
        }
    }

    pub fn clear_attribute_values(&mut self) {
        self.attribute_values.clear();
        self.clear_attributes_value_event
            .raise(EntityChangedEvent::new(self));
        self.entity_changed_event
            .raise(EntityChangedEvent::new(self));
    }

    pub fn clear_uniform_values(&mut self) {
        self.uniform_blocks_values.clear();
        self.clear_uniform_values_event
            .raise(EntityChangedEvent::new(self));
        self.entity_changed_event
            .raise(EntityChangedEvent::new(self));
    }

    pub fn clear_uniform_blocks_values(&mut self) {
        self.uniform_blocks_values.clear();
        self.clear_uniform_block_values_event
            .raise(EntityChangedEvent::new(self));
        self.entity_changed_event
            .raise(EntityChangedEvent::new(self));
    }

    pub fn clear_properties(&mut self) {
        self.properties.clear();
        self.clear_properties_event
            .raise(EntityChangedEvent::new(self));
        self.entity_changed_event
            .raise(EntityChangedEvent::new(self));
    }

    pub fn update(&mut self) {
        if !self.dirty {
            return;
        }

        self.update_matrix();
        self.update_bounding();
        self.dirty = false;
    }

    fn update_matrix(&mut self) {
        self.compose_model_matrix = match self.parent_model_matrix {
            Some(parent_model_matrix) => parent_model_matrix * self.model_matrix,
            None => self.model_matrix,
        };
        self.compose_normal_matrix = self
            .compose_model_matrix
            .invert()
            .expect("matrix with zero determinant is not allowed")
            .transpose();
    }

    fn update_bounding(&mut self) {
        self.bounding = self
            .geometry
            .as_ref()
            .and_then(|geom| geom.bounding_volume())
            .map(|bounding| bounding.transform(&self.compose_model_matrix))
            .map(|bounding| CullingBoundingVolume::new(bounding));
    }

    pub fn entity_changed_event(&mut self) -> &mut EventAgency<EntityChangedEvent> {
        &mut self.entity_changed_event
    }

    pub fn geometry_event(&mut self) -> &mut EventAgency<EntityChangedEvent> {
        &mut self.geometry_event
    }

    pub fn material_event(&mut self) -> &mut EventAgency<EntityChangedEvent> {
        &mut self.material_event
    }

    pub fn model_matrix_event(&mut self) -> &mut EventAgency<EntityChangedEvent> {
        &mut self.model_matrix_event
    }

    pub fn add_attribute_value_event(&mut self) -> &mut EventAgency<EntityChangedEvent> {
        &mut self.add_attribute_value_event
    }

    pub fn add_uniform_value_event(&mut self) -> &mut EventAgency<EntityChangedEvent> {
        &mut self.add_uniform_value_event
    }

    pub fn add_uniform_block_value_event(&mut self) -> &mut EventAgency<EntityChangedEvent> {
        &mut self.add_uniform_block_value_event
    }

    pub fn add_property_event(&mut self) -> &mut EventAgency<EntityChangedEvent> {
        &mut self.add_property_event
    }

    pub fn remove_attribute_value_event(&mut self) -> &mut EventAgency<EntityChangedEvent> {
        &mut self.remove_attribute_value_event
    }

    pub fn remove_uniform_value_event(&mut self) -> &mut EventAgency<EntityChangedEvent> {
        &mut self.remove_uniform_value_event
    }

    pub fn remove_uniform_block_value_event(&mut self) -> &mut EventAgency<EntityChangedEvent> {
        &mut self.remove_uniform_block_value_event
    }

    pub fn remove_property_event(&mut self) -> &mut EventAgency<EntityChangedEvent> {
        &mut self.remove_property_event
    }

    pub fn clear_attributes_value_event(&mut self) -> &mut EventAgency<EntityChangedEvent> {
        &mut self.clear_attributes_value_event
    }

    pub fn clear_uniform_values_event(&mut self) -> &mut EventAgency<EntityChangedEvent> {
        &mut self.clear_uniform_values_event
    }

    pub fn clear_uniform_block_values_event(&mut self) -> &mut EventAgency<EntityChangedEvent> {
        &mut self.clear_uniform_block_values_event
    }

    pub fn clear_properties_event(&mut self) -> &mut EventAgency<EntityChangedEvent> {
        &mut self.clear_properties_event
    }
}

pub struct EntityChangedEvent(*const Entity);

impl EntityChangedEvent {
    fn new(entity: &Entity) -> Self {
        Self(entity)
    }

    pub fn entity(&self) -> &Entity {
        unsafe { &*self.0 }
    }
}
