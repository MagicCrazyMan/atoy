pub mod collection;

use std::{any::Any, collections::HashMap, ptr::NonNull};

use gl_matrix4rust::mat4::{AsMat4, Mat4};
use wasm_bindgen::prelude::wasm_bindgen;
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

#[wasm_bindgen]
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
    
    event: EventAgency<Event>,
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
            event: EventAgency::new(),
            dirty: true,
            compose_model_matrix: Mat4::new_identity(),
            compose_normal_matrix: Mat4::new_identity(),
            bounding: None,
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

    pub fn event(&mut self) -> &mut EventAgency<Event> {
        &mut self.event
    }

    pub fn set_model_matrix(&mut self, model_matrix: Mat4) {
        self.model_matrix = model_matrix;
        self.dirty = true;
        self.event.raise(Event::SetModelMatrix(unsafe {
            NonNull::new_unchecked(&mut self.model_matrix)
        }));
    }

    pub fn set_geometry<G>(&mut self, geometry: Option<G>)
    where
        G: Geometry + 'static,
    {
        self.geometry = geometry.map(|geometry| Box::new(geometry) as Box<dyn Geometry>);
        self.dirty = true;
        self.event.raise(Event::SetGeometry(
            match self.geometry.as_deref_mut() {
                Some(geom) => Some(unsafe { NonNull::new_unchecked(geom) }),
                None => None,
            },
        ));
    }

    pub fn set_material<M>(&mut self, material: Option<M>)
    where
        M: Material + 'static,
    {
        self.material = material.map(|material| Box::new(material) as Box<dyn Material>);
        self.dirty = true;
        self.event.raise(Event::SetMaterial(
            match self.material.as_deref_mut() {
                Some(material) => Some(unsafe { NonNull::new_unchecked(material) }),
                None => None,
            },
        ));
    }

    pub fn add_attribute_value<S>(&mut self, name: S, value: AttributeValue)
    where
        S: Into<String>,
    {
        let name = name.into();
        self.attribute_values.insert(name.clone(), value);
        self.event.raise(Event::AddAttributeValue(name));
    }

    pub fn add_uniform_value<S>(&mut self, name: S, value: UniformValue)
    where
        S: Into<String>,
    {
        let name = name.into();
        self.uniform_values.insert(name.clone(), value);
        self.event.raise(Event::AddUniformValue(name));
    }

    pub fn add_uniform_block_value<S>(&mut self, name: S, value: UniformBlockValue)
    where
        S: Into<String>,
    {
        let name = name.into();
        self.uniform_blocks_values.insert(name.clone(), value);
        self.event.raise(Event::AddUniformBlockValue(name));
    }

    pub fn add_property<S, T>(&mut self, name: S, value: T)
    where
        S: Into<String>,
        T: 'static,
    {
        let name = name.into();
        self.properties.insert(name.clone(), Box::new(value));
        self.event.raise(Event::AddProperty(name));
    }

    pub fn remove_attribute_value(&mut self, name: &str) {
        if let Some(entry) = self.attribute_values.remove_entry(name) {
            self.event.raise(Event::RemoveAttributeValue(entry));
        }
    }

    pub fn remove_uniform_value(&mut self, name: &str) {
        if let Some(entry) = self.uniform_values.remove_entry(name) {
            self.event.raise(Event::RemoveUniformValue(entry));
        }
    }

    pub fn remove_uniform_block_value(&mut self, name: &str) {
        if let Some(entry) = self.uniform_blocks_values.remove_entry(name) {
            self.event.raise(Event::RemoveUniformBlockValue(entry));
        }
    }

    pub fn remove_property(&mut self, name: &str) {
        if let Some((key, mut value)) = self.properties.remove_entry(name) {
            self.event.raise(Event::RemoveProperty((key, unsafe {
                NonNull::new_unchecked(value.as_mut())
            })));
        }
    }

    pub fn clear_attribute_values(&mut self) {
        self.attribute_values.clear();
        self.event.raise(Event::ClearAttributeValues);
    }

    pub fn clear_uniform_values(&mut self) {
        self.uniform_blocks_values.clear();
        self.event.raise(Event::ClearUniformValues);
    }

    pub fn clear_uniform_blocks_values(&mut self) {
        self.uniform_blocks_values.clear();
        self.event.raise(Event::ClearUniformBlockValues);
    }

    pub fn clear_properties(&mut self) {
        self.properties.clear();
        self.event.raise(Event::ClearProperties);
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
}

pub enum Event {
    SetGeometry(Option<NonNull<dyn Geometry>>),
    SetMaterial(Option<NonNull<dyn Material>>),
    SetModelMatrix(NonNull<Mat4>),
    AddAttributeValue(String),
    AddUniformValue(String),
    AddUniformBlockValue(String),
    AddProperty(String),
    RemoveAttributeValue((String, AttributeValue)),
    RemoveUniformValue((String, UniformValue)),
    RemoveUniformBlockValue((String, UniformBlockValue)),
    RemoveProperty((String, NonNull<dyn Any>)),
    ClearAttributeValues,
    ClearUniformValues,
    ClearUniformBlockValues,
    ClearProperties,
}
