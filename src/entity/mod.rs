pub mod collection;

use std::{
    any::Any,
    collections::HashMap,
    rc::{Rc, Weak},
};

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

use self::collection::EntityCollectionWeak;

struct Inner {
    id: Uuid,
    model_matrix: Mat4,
    attribute_values: HashMap<String, AttributeValue>,
    uniform_values: HashMap<String, UniformValue>,
    uniform_blocks_values: HashMap<String, UniformBlockValue>,
    properties: HashMap<String, Box<dyn Any>>,
    geometry: Option<Box<dyn Geometry>>,
    material: Option<Box<dyn Material>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum DelegateKind {
    Geometry,
    Material,
}

struct Runtime {
    dirty: bool,
    collection: Option<EntityCollectionWeak>,
    compose_model_matrix: Mat4,
    compose_normal_matrix: Mat4,
    bounding: Option<CullingBoundingVolume>,
    changed_event: EventAgency<Event>,
    delegated_geometry_event: Option<Uuid>,
    delegated_material_event: Option<Uuid>,
}

#[derive(Clone)]
pub struct Entity {
    marker: Rc<()>,
    inner: *mut Inner,
    runtime: *mut Runtime,
}

impl Entity {
    pub fn new() -> Self {
        let inner = Inner {
            id: Uuid::new_v4(),
            model_matrix: Mat4::new_identity(),
            attribute_values: HashMap::new(),
            uniform_values: HashMap::new(),
            uniform_blocks_values: HashMap::new(),
            properties: HashMap::new(),
            geometry: None,
            material: None,
        };
        let runtime = Runtime {
            dirty: true,
            collection: None,
            compose_model_matrix: Mat4::new_identity(),
            compose_normal_matrix: Mat4::new_identity(),
            bounding: None,
            changed_event: EventAgency::new(),
            delegated_geometry_event: None,
            delegated_material_event: None,
        };

        Self {
            marker: Rc::new(()),
            inner: Box::leak(Box::new(inner)),
            runtime: Box::leak(Box::new(runtime)),
        }
    }

    #[inline]
    fn inner(&self) -> &Inner {
        unsafe { &*self.inner }
    }

    #[inline]
    fn inner_mut(&mut self) -> &mut Inner {
        unsafe { &mut *self.inner }
    }

    #[inline]
    fn runtime(&self) -> &Runtime {
        unsafe { &*self.runtime }
    }

    #[inline]
    fn runtime_mut(&mut self) -> &mut Runtime {
        unsafe { &mut *self.runtime }
    }

    fn undelegate_event(&mut self, kind: DelegateKind) {
        match kind {
            DelegateKind::Geometry => {
                let Some(listener) = self.runtime_mut().delegated_geometry_event.take() else {
                    return;
                };
                let Some(geometry) = self.inner().geometry.as_deref() else {
                    return;
                };
                geometry.changed_event().off(&listener);
            }
            DelegateKind::Material => {
                let Some(listener) = self.runtime_mut().delegated_material_event.take() else {
                    return;
                };
                let Some(material) = self.inner().material.as_deref() else {
                    return;
                };
                material.changed_event().off(&listener);
            }
        }
    }

    fn delegate_event(&mut self, kind: DelegateKind) {
        match kind {
            DelegateKind::Geometry => {
                let Some(geometry) = self.inner().geometry.as_deref() else {
                    return;
                };
                let me = self.weak();
                let changed_event = self.runtime().changed_event.clone();
                let listener = geometry.changed_event().on(move |_| {
                    let Some(mut me) = me.upgrade() else {
                        return;
                    };
                    me.runtime_mut().dirty = true;
                    changed_event.raise(Event::new(EventKind::Geometry, &me));
                });
                self.runtime_mut().delegated_geometry_event = Some(listener);
            }
            DelegateKind::Material => {
                let Some(material) = self.inner().material.as_deref() else {
                    return;
                };
                let me = self.weak();
                let changed_event = self.runtime().changed_event.clone();
                let listener = material.changed_event().on(move |_| {
                    let Some(mut me) = me.upgrade() else {
                        return;
                    };
                    me.runtime_mut().dirty = true;
                    changed_event.raise(Event::new(EventKind::Material, &me));
                });
                self.runtime_mut().delegated_material_event = Some(listener);
            }
        };
    }

    pub fn weak(&self) -> EntityWeak {
        EntityWeak {
            marker: Rc::downgrade(&self.marker),
            inner: self.inner,
            runtime: self.runtime,
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.inner().id
    }

    pub fn bounding(&self) -> Option<&CullingBoundingVolume> {
        self.runtime().bounding.as_ref()
    }

    pub fn model_matrix(&self) -> &Mat4 {
        &self.inner().model_matrix
    }

    pub fn compose_model_matrix(&self) -> &Mat4 {
        &self.runtime().compose_model_matrix
    }

    pub fn compose_normal_matrix(&self) -> &Mat4 {
        &self.runtime().compose_normal_matrix
    }

    pub fn attribute_values(&self) -> &HashMap<String, AttributeValue> {
        &self.inner().attribute_values
    }

    pub fn uniform_values(&self) -> &HashMap<String, UniformValue> {
        &self.inner().uniform_values
    }

    pub fn uniform_blocks_values(&self) -> &HashMap<String, UniformBlockValue> {
        &self.inner().uniform_blocks_values
    }

    pub fn properties(&self) -> &HashMap<String, Box<dyn Any>> {
        &self.inner().properties
    }

    pub fn geometry(&self) -> Option<&dyn Geometry> {
        self.inner().geometry.as_deref()
    }

    pub fn geometry_mut(&mut self) -> Option<&mut dyn Geometry> {
        match &mut self.inner_mut().geometry {
            Some(geometry) => Some(&mut **geometry),
            None => None,
        }
    }

    pub fn material(&self) -> Option<&dyn Material> {
        self.inner().material.as_deref()
    }

    pub fn material_mut(&mut self) -> Option<&mut dyn Material> {
        match &mut self.inner_mut().material {
            Some(material) => Some(&mut **material),
            None => None,
        }
    }

    pub fn set_model_matrix(&mut self, model_matrix: Mat4) {
        self.inner_mut().model_matrix = model_matrix;
        self.runtime_mut().dirty = true;
        self.runtime()
            .changed_event
            .raise(Event::new(EventKind::ModelMatrix, self));
    }

    pub fn set_geometry<G>(&mut self, geometry: Option<G>)
    where
        G: Geometry + 'static,
    {
        self.undelegate_event(DelegateKind::Geometry);
        self.inner_mut().geometry =
            geometry.map(|geometry| Box::new(geometry) as Box<dyn Geometry>);
        self.runtime_mut().dirty = true;
        self.delegate_event(DelegateKind::Geometry);
        self.runtime()
            .changed_event
            .raise(Event::new(EventKind::Geometry, self));
    }

    pub fn set_material<M>(&mut self, material: Option<M>)
    where
        M: Material + 'static,
    {
        self.undelegate_event(DelegateKind::Material);
        self.inner_mut().material =
            material.map(|material| Box::new(material) as Box<dyn Material>);
        self.runtime_mut().dirty = true;
        self.delegate_event(DelegateKind::Material);
        self.runtime()
            .changed_event
            .raise(Event::new(EventKind::Material, self));
    }

    pub fn add_attribute_value<S>(&mut self, name: S, value: AttributeValue)
    where
        S: Into<String>,
    {
        let name = name.into();
        self.inner_mut()
            .attribute_values
            .insert(name.clone(), value);
        self.runtime_mut().dirty = true;
        self.runtime()
            .changed_event
            .raise(Event::new(EventKind::AddAttributeValue, self));
    }

    pub fn add_uniform_value<S>(&mut self, name: S, value: UniformValue)
    where
        S: Into<String>,
    {
        let name = name.into();
        self.inner_mut().uniform_values.insert(name.clone(), value);
        self.runtime_mut().dirty = true;
        self.runtime()
            .changed_event
            .raise(Event::new(EventKind::AddUniformValue, self));
    }

    pub fn add_uniform_block_value<S>(&mut self, name: S, value: UniformBlockValue)
    where
        S: Into<String>,
    {
        let name = name.into();
        self.inner_mut()
            .uniform_blocks_values
            .insert(name.clone(), value);
        self.runtime_mut().dirty = true;
        self.runtime()
            .changed_event
            .raise(Event::new(EventKind::AddUniformBlockValue, self));
    }

    pub fn add_property<S, T>(&mut self, name: S, value: T)
    where
        S: Into<String>,
        T: 'static,
    {
        let name = name.into();
        self.inner_mut()
            .properties
            .insert(name.clone(), Box::new(value));
        self.runtime_mut().dirty = true;
        self.runtime()
            .changed_event
            .raise(Event::new(EventKind::AddProperty, self));
    }

    pub fn remove_attribute_value(&mut self, name: &str) -> Option<(String, AttributeValue)> {
        if let Some(entry) = self.inner_mut().attribute_values.remove_entry(name) {
            self.runtime_mut().dirty = true;
            self.runtime()
                .changed_event
                .raise(Event::new(EventKind::RemoveAttributeValue, self));
            Some(entry)
        } else {
            None
        }
    }

    pub fn remove_uniform_value(&mut self, name: &str) -> Option<(String, UniformValue)> {
        if let Some(entry) = self.inner_mut().uniform_values.remove_entry(name) {
            self.runtime_mut().dirty = true;
            self.runtime()
                .changed_event
                .raise(Event::new(EventKind::RemoveUniformValue, self));
            Some(entry)
        } else {
            None
        }
    }

    pub fn remove_uniform_block_value(
        &mut self,
        name: &str,
    ) -> Option<(String, UniformBlockValue)> {
        if let Some(entry) = self.inner_mut().uniform_blocks_values.remove_entry(name) {
            self.runtime_mut().dirty = true;
            self.runtime()
                .changed_event
                .raise(Event::new(EventKind::RemoveUniformBlockValue, self));
            Some(entry)
        } else {
            None
        }
    }

    pub fn remove_property(&mut self, name: &str) -> Option<(String, Box<dyn Any>)> {
        if let Some(entry) = self.inner_mut().properties.remove_entry(name) {
            self.runtime_mut().dirty = true;
            self.runtime()
                .changed_event
                .raise(Event::new(EventKind::RemoveProperty, self));
            Some(entry)
        } else {
            None
        }
    }

    pub fn clear_attribute_values(&mut self) {
        self.inner_mut().attribute_values.clear();
        self.runtime_mut().dirty = true;
        self.runtime()
            .changed_event
            .raise(Event::new(EventKind::ClearAttributeValues, self));
    }

    pub fn clear_uniform_values(&mut self) {
        self.inner_mut().uniform_blocks_values.clear();
        self.runtime_mut().dirty = true;
        self.runtime()
            .changed_event
            .raise(Event::new(EventKind::ClearUniformValues, self));
    }

    pub fn clear_uniform_blocks_values(&mut self) {
        self.inner_mut().uniform_blocks_values.clear();
        self.runtime_mut().dirty = true;
        self.runtime()
            .changed_event
            .raise(Event::new(EventKind::ClearUniformBlockValues, self));
    }

    pub fn clear_properties(&mut self) {
        self.inner_mut().properties.clear();
        self.runtime_mut().dirty = true;
        self.runtime()
            .changed_event
            .raise(Event::new(EventKind::ClearProperties, self));
    }

    pub fn update(&mut self) {
        if !self.runtime().dirty {
            return;
        }

        self.update_matrix();
        self.update_bounding();
        self.runtime_mut().dirty = false;
    }

    fn update_matrix(&mut self) {
        self.runtime_mut().compose_model_matrix = match self
            .runtime()
            .collection
            .as_ref()
            .and_then(|collection| collection.upgrade())
            .map(|collection| collection.runtime().compose_model_matrix)
        {
            Some(parent_model_matrix) => parent_model_matrix * self.inner().model_matrix,
            None => self.inner().model_matrix,
        };
        self.runtime_mut().compose_normal_matrix = self
            .runtime()
            .compose_model_matrix
            .invert()
            .expect("matrix with zero determinant is not allowed")
            .transpose();
    }

    fn update_bounding(&mut self) {
        self.runtime_mut().bounding = self
            .inner()
            .geometry
            .as_ref()
            .and_then(|geom| geom.bounding_volume())
            .map(|bounding| bounding.transform(&self.runtime().compose_model_matrix))
            .map(|bounding| CullingBoundingVolume::new(bounding));
    }

    pub fn changed_event(&self) -> &EventAgency<Event> {
        &self.runtime().changed_event
    }
}

impl Drop for Entity {
    fn drop(&mut self) {
        if Rc::strong_count(&self.marker) == 1 {
            unsafe {
                drop(Box::from_raw(self.inner));
                drop(Box::from_raw(self.runtime));
            }
        }
    }
}

#[derive(Clone)]
pub struct EntityWeak {
    marker: Weak<()>,
    inner: *mut Inner,
    runtime: *mut Runtime,
}

impl EntityWeak {
    pub fn upgrade(&self) -> Option<Entity> {
        self.marker.upgrade().map(|marker| Entity {
            marker,
            inner: self.inner,
            runtime: self.runtime,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventKind {
    Geometry,
    Material,
    ModelMatrix,
    AddAttributeValue,
    AddUniformValue,
    AddUniformBlockValue,
    AddProperty,
    RemoveAttributeValue,
    RemoveUniformValue,
    RemoveUniformBlockValue,
    RemoveProperty,
    ClearAttributeValues,
    ClearUniformValues,
    ClearUniformBlockValues,
    ClearProperties,
}

pub struct Event(EventKind, *const Entity);

impl Event {
    fn new(kind: EventKind, entity: &Entity) -> Self {
        Self(kind, entity)
    }

    pub fn kind(&self) -> EventKind {
        self.0
    }

    pub fn entity(&self) -> &Entity {
        unsafe { &*self.1 }
    }
}
