pub mod collection;

use std::{any::Any, collections::HashMap, ptr::NonNull};

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

use self::collection::EntityCollection;

pub struct EntityRaw {
    id: Uuid,
    model_matrix: Mat4,
    attribute_values: HashMap<String, AttributeValue>,
    uniform_values: HashMap<String, UniformValue>,
    uniform_blocks_values: HashMap<String, UniformBlockValue>,
    properties: HashMap<String, Box<dyn Any>>,
    geometry: Option<Box<dyn Geometry>>,
    material: Option<Box<dyn Material>>,
    bounding: Option<CullingBoundingVolume>,
}

impl EntityRaw {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            model_matrix: Mat4::new_identity(),
            attribute_values: HashMap::new(),
            uniform_values: HashMap::new(),
            uniform_blocks_values: HashMap::new(),
            properties: HashMap::new(),
            geometry: None,
            material: None,
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
        self.geometry.as_ref().map(|geom| geom.as_ref())
    }

    pub fn geometry_mut(&mut self) -> Option<&mut dyn Geometry> {
        match self.geometry.as_mut() {
            Some(geom) => Some(geom.as_mut()),
            None => None,
        }
    }

    pub fn material(&self) -> Option<&dyn Material> {
        self.material.as_ref().map(|geom| geom.as_ref())
    }

    pub fn material_mut(&mut self) -> Option<&mut dyn Material> {
        match self.material.as_mut() {
            Some(material) => Some(material.as_mut()),
            None => None,
        }
    }

    pub fn set_model_matrix(&mut self, model_matrix: Mat4) {
        self.model_matrix = model_matrix;
        self.bounding = self
            .geometry
            .as_ref()
            .and_then(|geom| geom.bounding_volume())
            .map(|bounding| bounding.transform(&self.model_matrix))
            .map(|bounding| CullingBoundingVolume::new(bounding));
    }

    pub fn set_geometry<G: Geometry + 'static>(&mut self, geometry: Option<G>) {
        match geometry {
            Some(geometry) => {
                self.geometry = Some(Box::new(geometry));
                self.bounding = self
                    .geometry
                    .as_ref()
                    .and_then(|geom| geom.bounding_volume())
                    .map(|bounding| bounding.transform(&self.model_matrix))
                    .map(|bounding| CullingBoundingVolume::new(bounding));
            }
            None => {
                self.geometry = None;
                self.bounding = None;
            }
        };
    }

    pub fn set_material<M: Material + 'static>(&mut self, material: Option<M>) {
        match material {
            Some(material) => {
                self.material = Some(Box::new(material));
            }
            None => {
                self.material = None;
            }
        };
    }

    pub fn add_attribute_value<S: Into<String>>(&mut self, name: S, value: AttributeValue) {
        let name = name.into();
        self.attribute_values.insert(name.clone(), value);
    }

    pub fn add_uniform_value<S: Into<String>>(&mut self, name: S, value: UniformValue) {
        let name = name.into();
        self.uniform_values.insert(name, value);
    }

    pub fn add_uniform_block_value<S: Into<String>>(&mut self, name: S, value: UniformBlockValue) {
        let name = name.into();
        self.uniform_blocks_values.insert(name, value);
    }

    pub fn add_property<S: Into<String>, T: 'static>(&mut self, name: S, value: T) {
        let name = name.into();
        self.properties.insert(name, Box::new(value));
    }

    pub fn remove_attribute_value(&mut self, name: &str) -> Option<(String, AttributeValue)> {
        self.attribute_values.remove_entry(name)
    }

    pub fn remove_uniform_value(&mut self, name: &str) -> Option<(String, UniformValue)> {
        self.uniform_values.remove_entry(name)
    }

    pub fn remove_uniform_block_value(
        &mut self,
        name: &str,
    ) -> Option<(String, UniformBlockValue)> {
        self.uniform_blocks_values.remove_entry(name)
    }

    pub fn remove_property(&mut self, name: &str) -> Option<(String, Box<dyn Any>)> {
        self.properties.remove_entry(name)
    }

    pub fn clear_attribute_values(&mut self) {
        self.attribute_values.clear();
    }

    pub fn clear_uniform_values(&mut self) {
        self.uniform_values.clear();
    }

    pub fn clear_uniform_blocks_values(&mut self) {
        self.uniform_blocks_values.clear();
    }

    pub fn clear_properties(&mut self) {
        self.properties.clear();
    }
}

pub struct Entity {
    raw: EntityRaw,
    event: EventAgency<Event>,
    dirty: bool,
    collection: *const EntityCollection,
    compose_model_matrix: Mat4,
    compose_normal_matrix: Mat4,
}

impl Entity {
    pub(self) fn new(raw: EntityRaw, collection: *const EntityCollection) -> Self {
        Self {
            raw,
            event: EventAgency::new(),
            dirty: true,
            collection,
            compose_model_matrix: Mat4::new_identity(),
            compose_normal_matrix: Mat4::new_identity(),
        }
    }

    pub fn id(&self) -> &Uuid {
        self.raw.id()
    }

    pub fn collection(&self) -> Option<&EntityCollection> {
        if self.collection.is_null() {
            None
        } else {
            unsafe { Some(&*self.collection) }
        }
    }

    pub fn bounding(&self) -> Option<&CullingBoundingVolume> {
        self.raw.bounding()
    }

    pub fn model_matrix(&self) -> &Mat4 {
        self.raw.model_matrix()
    }

    pub fn compose_model_matrix(&self) -> &Mat4 {
        &self.compose_model_matrix
    }

    pub fn compose_normal_matrix(&self) -> &Mat4 {
        &self.compose_normal_matrix
    }

    pub fn attribute_values(&self) -> &HashMap<String, AttributeValue> {
        self.raw.attribute_values()
    }

    pub fn uniform_values(&self) -> &HashMap<String, UniformValue> {
        self.raw.uniform_values()
    }

    pub fn uniform_blocks_values(&self) -> &HashMap<String, UniformBlockValue> {
        self.raw.uniform_blocks_values()
    }

    pub fn properties(&self) -> &HashMap<String, Box<dyn Any>> {
        self.raw.properties()
    }

    pub fn geometry(&self) -> Option<&dyn Geometry> {
        self.raw.geometry()
    }

    pub fn geometry_mut(&mut self) -> Option<&mut dyn Geometry> {
        self.raw.geometry_mut()
    }

    pub fn material(&self) -> Option<&dyn Material> {
        self.raw.material()
    }

    pub fn material_mut(&mut self) -> Option<&mut dyn Material> {
        self.raw.material_mut()
    }

    pub fn event(&mut self) -> &mut EventAgency<Event> {
        &mut self.event
    }

    pub fn set_model_matrix(&mut self, model_matrix: Mat4) {
        self.raw.set_model_matrix(model_matrix);
        self.dirty = true;
        self.event.raise(Event::SetModelMatrix(unsafe {
            NonNull::new_unchecked(&mut self.raw.model_matrix)
        }));
    }

    pub fn set_geometry<G: Geometry + 'static>(&mut self, geometry: Option<G>) {
        self.raw.set_geometry(geometry);
        self.dirty = true;
        self.event
            .raise(Event::SetGeometry(match self.raw.geometry.as_deref_mut() {
                Some(geom) => Some(unsafe { NonNull::new_unchecked(geom) }),
                None => None,
            }));
    }

    pub fn set_material<M: Material + 'static>(&mut self, material: Option<M>) {
        self.raw.set_material(material);
        self.dirty = true;
        self.event
            .raise(Event::SetMaterial(match self.raw.material.as_deref_mut() {
                Some(material) => Some(unsafe { NonNull::new_unchecked(material) }),
                None => None,
            }));
    }

    pub fn add_attribute_value<S: Into<String>>(&mut self, name: S, value: AttributeValue) {
        let name = name.into();
        self.raw.add_attribute_value(name.clone(), value);
        self.event.raise(Event::AddAttributeValue(name));
    }

    pub fn add_uniform_value<S: Into<String>>(&mut self, name: S, value: UniformValue) {
        let name = name.into();
        self.raw.add_uniform_value(name.clone(), value);
        self.event.raise(Event::AddUniformValue(name));
    }

    pub fn add_uniform_block_value<S: Into<String>>(&mut self, name: S, value: UniformBlockValue) {
        let name = name.into();
        self.raw.add_uniform_block_value(name.clone(), value);
        self.event.raise(Event::AddUniformBlockValue(name));
    }

    pub fn add_property<S: Into<String>, T: 'static>(&mut self, name: S, value: T) {
        let name = name.into();
        self.raw.add_property(name.clone(), value);
        self.event.raise(Event::AddProperty(name));
    }

    pub fn remove_attribute_value(&mut self, name: &str) {
        if let Some(entry) = self.raw.remove_attribute_value(name) {
            self.event.raise(Event::RemoveAttributeValue(entry));
        }
    }

    pub fn remove_uniform_value(&mut self, name: &str) {
        if let Some(entry) = self.raw.remove_uniform_value(name) {
            self.event.raise(Event::RemoveUniformValue(entry));
        }
    }

    pub fn remove_uniform_block_value(&mut self, name: &str) {
        if let Some(entry) = self.raw.remove_uniform_block_value(name) {
            self.event.raise(Event::RemoveUniformBlockValue(entry));
        }
    }

    pub fn remove_property(&mut self, name: &str) {
        if let Some((key, mut value)) = self.raw.remove_property(name) {
            self.event.raise(Event::RemoveProperty((key, unsafe {
                NonNull::new_unchecked(value.as_mut())
            })));
        }
    }

    pub fn clear_attribute_values(&mut self) {
        self.raw.clear_attribute_values();
        self.event.raise(Event::ClearAttributeValues);
    }

    pub fn clear_uniform_values(&mut self) {
        self.raw.clear_uniform_values();
        self.event.raise(Event::ClearUniformValues);
    }

    pub fn clear_uniform_blocks_values(&mut self) {
        self.raw.clear_uniform_blocks_values();
        self.event.raise(Event::ClearUniformBlockValues);
    }

    pub fn clear_properties(&mut self) {
        self.raw.clear_properties();
        self.event.raise(Event::ClearProperties);
    }

    pub fn update(&mut self) {
        if !self.dirty {
            return;
        }

        self.update_matrix();
        self.dirty = false;
    }

    fn update_matrix(&mut self) {
        if self.collection.is_null() {
            self.compose_model_matrix = self.raw.model_matrix;
        } else {
            unsafe {
                self.compose_model_matrix =
                    *(*self.collection).compose_model_matrix() * self.raw.model_matrix;
            }
        }
        self.compose_normal_matrix = self
            .compose_model_matrix
            .invert()
            .expect("matrix with zero determinant is not allowed")
            .transpose();
    }
}

#[derive(Clone)]
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

// /// An entity in rendering procedure.
// /// In this stage, geometry and material may differs from [`Entity`].
// pub struct RenderingEntity {
//     model_matrix: Mat4,
//     attribute_values: HashMap<String, AttributeValue>,
//     uniform_values: HashMap<String, UniformValue>,
//     uniform_blocks_values: HashMap<String, UniformBlockValue>,
//     properties: HashMap<String, Box<dyn Any>>,
//     geometry: Option<Box<dyn Geometry>>,
//     material: Option<Box<dyn Material>>,
// }
