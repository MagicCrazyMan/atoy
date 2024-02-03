use std::{any::Any, collections::VecDeque, iter::FromIterator};

use gl_matrix4rust::mat4::Mat4;
use hashbrown::HashMap;
use indexmap::IndexMap;
use uuid::Uuid;

use crate::{
    bounding::{merge_bounding_volumes, CullingBoundingVolume},
    error::Error,
    geometry::Geometry,
    material::StandardMaterial,
    notify::{Notifiee, Notifying},
    render::webgl::{
        attribute::AttributeValue,
        uniform::{UniformBlockValue, UniformValue},
    },
};

pub struct EntityOptions {
    model_matrix: Mat4,
    attribute_values: HashMap<String, AttributeValue>,
    uniform_values: HashMap<String, UniformValue>,
    uniform_blocks_values: HashMap<String, UniformBlockValue>,
    properties: HashMap<String, Box<dyn Any>>,
    geometry: Option<Box<dyn Geometry>>,
    material: Option<Box<dyn StandardMaterial>>,
}

impl EntityOptions {
    pub fn new() -> Self {
        Self {
            model_matrix: Mat4::<f64>::new_identity(),
            attribute_values: HashMap::new(),
            uniform_values: HashMap::new(),
            uniform_blocks_values: HashMap::new(),
            properties: HashMap::new(),
            geometry: None,
            material: None,
        }
    }

    pub fn model_matrix(&self) -> &Mat4 {
        &self.model_matrix
    }

    pub fn set_model_matrix(&mut self, model_matrix: Mat4) {
        self.model_matrix = model_matrix;
    }

    pub fn geometry(&self) -> Option<&dyn Geometry> {
        self.geometry.as_deref()
    }

    pub fn geometry_mut(&mut self) -> Option<&mut dyn Geometry> {
        match &mut self.geometry {
            Some(geometry) => Some(&mut **geometry),
            None => None,
        }
    }

    pub fn set_geometry<G>(&mut self, geometry: Option<G>)
    where
        G: Geometry + 'static,
    {
        self.geometry = geometry.map(|geometry| Box::new(geometry) as Box<dyn Geometry>);
    }

    pub fn material(&self) -> Option<&dyn StandardMaterial> {
        self.material.as_deref()
    }

    pub fn material_mut(&mut self) -> Option<&mut dyn StandardMaterial> {
        match &mut self.material {
            Some(material) => Some(&mut **material),
            None => None,
        }
    }

    pub fn set_material<M>(&mut self, material: Option<M>)
    where
        M: StandardMaterial + 'static,
    {
        self.material = material.map(|material| Box::new(material) as Box<dyn StandardMaterial>);
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

    pub fn add_attribute_value<S>(&mut self, name: S, value: AttributeValue)
    where
        S: Into<String>,
    {
        let name = name.into();
        self.attribute_values.insert(name.clone(), value);
    }

    pub fn add_uniform_value<S>(&mut self, name: S, value: UniformValue)
    where
        S: Into<String>,
    {
        let name = name.into();
        self.uniform_values.insert(name.clone(), value);
    }

    pub fn add_uniform_block_value<S>(&mut self, name: S, value: UniformBlockValue)
    where
        S: Into<String>,
    {
        let name = name.into();
        self.uniform_blocks_values.insert(name.clone(), value);
    }

    pub fn add_property<S, T>(&mut self, name: S, value: T)
    where
        S: Into<String>,
        T: 'static,
    {
        let name = name.into();
        self.properties.insert(name.clone(), Box::new(value));
    }

    pub fn remove_attribute_value(&mut self, name: &str) -> Option<(String, AttributeValue)> {
        if let Some(entry) = self.attribute_values.remove_entry(name) {
            Some(entry)
        } else {
            None
        }
    }

    pub fn remove_uniform_value(&mut self, name: &str) -> Option<(String, UniformValue)> {
        if let Some(entry) = self.uniform_values.remove_entry(name) {
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
            Some(entry)
        } else {
            None
        }
    }

    pub fn remove_property(&mut self, name: &str) -> Option<(String, Box<dyn Any>)> {
        if let Some(entry) = self.properties.remove_entry(name) {
            Some(entry)
        } else {
            None
        }
    }

    pub fn clear_attribute_values(&mut self) {
        self.attribute_values.clear();
    }

    pub fn clear_uniform_values(&mut self) {
        self.uniform_blocks_values.clear();
    }

    pub fn clear_uniform_blocks_values(&mut self) {
        self.uniform_blocks_values.clear();
    }

    pub fn clear_properties(&mut self) {
        self.properties.clear();
    }
}

pub struct GroupOptions {
    model_matrix: Mat4,
    entities: Vec<EntityOptions>,
    sub_groups: Vec<GroupOptions>,
}

impl GroupOptions {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            sub_groups: Vec::new(),
            model_matrix: Mat4::<f64>::new_identity(),
        }
    }

    pub fn model_matrix(&self) -> &Mat4 {
        &self.model_matrix
    }

    pub fn set_model_matrix(&mut self, model_matrix: Mat4) {
        self.model_matrix = model_matrix;
    }

    pub fn entities(&self) -> &[EntityOptions] {
        self.entities.as_ref()
    }

    pub fn entities_mut(&mut self) -> &mut Vec<EntityOptions> {
        &mut self.entities
    }

    pub fn sub_groups(&self) -> &[GroupOptions] {
        self.sub_groups.as_ref()
    }

    pub fn sub_groups_mut(&mut self) -> &mut Vec<GroupOptions> {
        &mut self.sub_groups
    }
}

pub struct Entity {
    id: Uuid,
    model_matrix: Mat4,
    attribute_values: HashMap<String, AttributeValue>,
    uniform_values: HashMap<String, UniformValue>,
    uniform_blocks_values: HashMap<String, UniformBlockValue>,
    properties: HashMap<String, Box<dyn Any>>,
    geometry: Option<(
        Box<dyn Geometry>,
        Option<CullingBoundingVolume>,
        Notifying<()>,
    )>,
    material: Option<(Box<dyn StandardMaterial>, Notifying<()>)>,

    group: *mut Group,
    dirty: *mut bool,
    compose_model_matrix: Mat4,
    compose_normal_matrix: Mat4,
}

struct EntityDirtyNotifiee {
    group: *mut Group,
    dirty: *mut bool,
}

impl Notifiee<()> for EntityDirtyNotifiee {
    fn notify(&mut self, _: &()) {
        unsafe {
            *self.dirty = true;
            (*self.group).set_dirty();
        }
    }
}

impl Entity {
    fn from_options(options: EntityOptions, id: Uuid, group: *mut Group) -> *mut Entity {
        let entity = Box::leak(Box::new(Entity {
            id,
            model_matrix: options.model_matrix,
            attribute_values: options.attribute_values,
            uniform_values: options.uniform_values,
            uniform_blocks_values: options.uniform_blocks_values,
            properties: options.properties,
            geometry: None,
            material: None,

            group,
            dirty: Box::leak(Box::new(true)),
            compose_model_matrix: Mat4::<f64>::new_identity(),
            compose_normal_matrix: Mat4::<f64>::new_identity(),
        }));
        entity.set_geometry_boxed(options.geometry);
        entity.set_material_boxed(options.material);
        entity
    }

    fn to_options(mut self) -> EntityOptions {
        let geometry = self.take_geometry();
        let material = self.take_material();
        unsafe { drop(Box::from_raw(self.dirty)) };
        EntityOptions {
            model_matrix: self.model_matrix,
            attribute_values: self.attribute_values,
            uniform_values: self.uniform_values,
            uniform_blocks_values: self.uniform_blocks_values,
            properties: self.properties,
            geometry,
            material,
        }
    }

    pub fn set_dirty(&mut self) {
        unsafe {
            *self.dirty = true;
            (*self.group).set_dirty();
        }
    }

    #[inline]
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    #[inline]
    pub fn geometry(&self) -> Option<&dyn Geometry> {
        match self.geometry.as_ref() {
            Some((geometry, _, _)) => Some(geometry.as_ref()),
            None => None,
        }
    }

    #[inline]
    pub fn geometry_mut(&mut self) -> Option<&mut dyn Geometry> {
        match self.geometry.as_mut() {
            Some((geometry, _, _)) => Some(geometry.as_mut()),
            None => None,
        }
    }

    pub fn set_geometry<G>(&mut self, geometry: Option<G>) -> Option<Box<dyn Geometry>>
    where
        G: Geometry + 'static,
    {
        self.set_geometry_boxed(geometry.map(|geometry| Box::new(geometry) as Box<dyn Geometry>))
    }

    pub fn set_geometry_boxed(
        &mut self,
        geometry: Option<Box<dyn Geometry>>,
    ) -> Option<Box<dyn Geometry>> {
        let old = self.take_geometry();

        if let Some(mut geometry) = geometry {
            let notifying = geometry.notifier().register(EntityDirtyNotifiee {
                group: self.group,
                dirty: self.dirty,
            });
            let bounding = geometry
                .bounding_volume()
                .map(|bounding| bounding.as_ref().transform(self.compose_model_matrix))
                .map(|bounding| CullingBoundingVolume::new(bounding));
            self.geometry = Some((geometry, bounding, notifying));
        }

        old
    }

    pub fn take_geometry(&mut self) -> Option<Box<dyn Geometry>> {
        let Some((geometry, _, notifying)) = self.geometry.take() else {
            return None;
        };
        notifying.unregister();
        self.set_dirty();
        Some(geometry)
    }

    pub fn material(&self) -> Option<&dyn StandardMaterial> {
        match self.material.as_ref() {
            Some((material, _)) => Some(material.as_ref()),
            None => None,
        }
    }

    #[inline]
    pub fn material_mut(&mut self) -> Option<&mut dyn StandardMaterial> {
        match self.material.as_mut() {
            Some((material, _)) => Some(material.as_mut()),
            None => None,
        }
    }

    pub fn set_material<M>(&mut self, material: Option<M>) -> Option<Box<dyn StandardMaterial>>
    where
        M: StandardMaterial + 'static,
    {
        self.set_material_boxed(
            material.map(|material| Box::new(material) as Box<dyn StandardMaterial>),
        )
    }

    pub fn set_material_boxed(
        &mut self,
        material: Option<Box<dyn StandardMaterial>>,
    ) -> Option<Box<dyn StandardMaterial>> {
        let old = self.take_material();

        if let Some(material) = material {
            let notifying = material
                .notifier()
                .borrow_mut()
                .register(EntityDirtyNotifiee {
                    group: self.group,
                    dirty: self.dirty,
                });
            self.material = Some((material, notifying));
        }

        old
    }

    pub fn take_material(&mut self) -> Option<Box<dyn StandardMaterial>> {
        let Some((material, notifying)) = self.material.take() else {
            return None;
        };
        notifying.unregister();
        self.set_dirty();
        Some(material)
    }

    #[inline]
    pub fn model_matrix(&self) -> &Mat4 {
        &self.model_matrix
    }

    pub fn set_model_matrix(&mut self, model_matrix: Mat4) {
        self.model_matrix = model_matrix;
        self.set_dirty();
    }

    #[inline]
    pub fn bounding(&self) -> Option<&CullingBoundingVolume> {
        self.geometry
            .as_ref()
            .and_then(|(_, bounding, _)| bounding.as_ref())
    }

    #[inline]
    pub fn compose_model_matrix(&self) -> &Mat4 {
        &self.compose_model_matrix
    }

    #[inline]
    pub fn compose_normal_matrix(&self) -> &Mat4 {
        &self.compose_normal_matrix
    }

    #[inline]
    pub fn attribute_values(&self) -> &HashMap<String, AttributeValue> {
        &self.attribute_values
    }

    #[inline]
    pub fn uniform_values(&self) -> &HashMap<String, UniformValue> {
        &self.uniform_values
    }

    #[inline]
    pub fn uniform_blocks_values(&self) -> &HashMap<String, UniformBlockValue> {
        &self.uniform_blocks_values
    }

    #[inline]
    pub fn properties(&self) -> &HashMap<String, Box<dyn Any>> {
        &self.properties
    }

    pub fn add_attribute_value<S>(&mut self, name: S, value: AttributeValue)
    where
        S: Into<String>,
    {
        let name = name.into();
        self.attribute_values.insert(name.clone(), value);
        self.set_dirty();
    }

    pub fn add_uniform_value<S>(&mut self, name: S, value: UniformValue)
    where
        S: Into<String>,
    {
        let name = name.into();
        self.uniform_values.insert(name.clone(), value);
        self.set_dirty();
    }

    pub fn add_uniform_block_value<S>(&mut self, name: S, value: UniformBlockValue)
    where
        S: Into<String>,
    {
        let name = name.into();
        self.uniform_blocks_values.insert(name.clone(), value);
        self.set_dirty();
    }

    pub fn add_property<S, T>(&mut self, name: S, value: T)
    where
        S: Into<String>,
        T: 'static,
    {
        let name = name.into();
        self.properties.insert(name.clone(), Box::new(value));
        self.set_dirty();
    }

    pub fn remove_attribute_value(&mut self, name: &str) -> Option<(String, AttributeValue)> {
        if let Some(entry) = self.attribute_values.remove_entry(name) {
            self.set_dirty();
            Some(entry)
        } else {
            None
        }
    }

    pub fn remove_uniform_value(&mut self, name: &str) -> Option<(String, UniformValue)> {
        if let Some(entry) = self.uniform_values.remove_entry(name) {
            self.set_dirty();
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
            self.set_dirty();
            Some(entry)
        } else {
            None
        }
    }

    pub fn remove_property(&mut self, name: &str) -> Option<(String, Box<dyn Any>)> {
        if let Some(entry) = self.properties.remove_entry(name) {
            self.set_dirty();
            Some(entry)
        } else {
            None
        }
    }

    pub fn clear_attribute_values(&mut self) {
        self.attribute_values.clear();
        self.set_dirty();
    }

    pub fn clear_uniform_values(&mut self) {
        self.uniform_blocks_values.clear();
        self.set_dirty();
    }

    pub fn clear_uniform_blocks_values(&mut self) {
        self.uniform_blocks_values.clear();
        self.set_dirty();
    }

    pub fn clear_properties(&mut self) {
        self.properties.clear();
        self.set_dirty();
    }

    pub fn update(&mut self) {
        unsafe {
            if *self.dirty {
                self.update_matrices();
                self.update_bounding();
                *self.dirty = false;
            }
        }
    }

    unsafe fn update_matrices(&mut self) {
        self.compose_model_matrix = (*self.group).model_matrix.clone() * self.model_matrix;
        self.compose_normal_matrix = self.compose_model_matrix.clone();
        self.compose_normal_matrix
            .invert_in_place()
            .expect("invert a matrix with zero determinant is not allowed");
        self.compose_normal_matrix.transpose_in_place();
    }

    fn update_bounding(&mut self) {
        let Some((geometry, bounding, _)) = self.geometry.as_mut() else {
            return;
        };

        let compose_model_matrix = self.compose_model_matrix;
        *bounding = geometry.bounding_volume().map(|bounding| {
            CullingBoundingVolume::new(bounding.as_ref().transform(compose_model_matrix))
        });
    }
}

enum GroupOwner {
    Container(*mut bool),
    Group(*mut Group),
}

pub struct Group {
    id: Uuid,
    owner: GroupOwner,
    entities: IndexMap<Uuid, *mut Entity>,
    sub_groups: IndexMap<Uuid, *mut Group>,
    model_matrix: Mat4,

    bounding: Option<CullingBoundingVolume>,

    dirty: bool,
}

impl Group {
    fn new(id: Uuid, owner: GroupOwner) -> Self {
        Self {
            id,
            owner,
            entities: IndexMap::new(),
            sub_groups: IndexMap::new(),

            model_matrix: Mat4::<f64>::new_identity(),
            bounding: None,

            dirty: true,
        }
    }

    pub fn set_dirty(&mut self) {
        unsafe {
            self.dirty = true;
            match self.owner {
                GroupOwner::Container(dirty) => *dirty = true,
                GroupOwner::Group(super_group) => (*super_group).set_dirty(),
            }
        }
    }

    #[inline]
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    #[inline]
    pub fn entities_len(&self) -> usize {
        self.entities.len()
    }

    #[inline]
    pub fn model_matrix(&self) -> &Mat4 {
        &self.model_matrix
    }

    pub fn set_model_matrix(&mut self, model_matrix: Mat4) {
        self.model_matrix = model_matrix;
        self.set_dirty();
    }

    #[inline]
    pub fn bounding(&self) -> Option<&CullingBoundingVolume> {
        self.bounding.as_ref()
    }

    #[inline]
    pub fn super_group(&self) -> Option<&Group> {
        unsafe {
            match self.owner {
                GroupOwner::Container(_) => None,
                GroupOwner::Group(super_group) => Some(&*super_group),
            }
        }
    }

    #[inline]
    pub fn entity(&self, id: &Uuid) -> Option<&Entity> {
        self.entities.get(id).map(|entity| unsafe { &**entity })
    }

    #[inline]
    pub fn entity_mut(&mut self, id: &Uuid) -> Option<&mut Entity> {
        self.entities
            .get_mut(id)
            .map(|entity| unsafe { &mut **entity })
    }

    #[inline]
    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.values().map(|entity| unsafe { &**entity })
    }

    #[inline]
    pub fn entities_mut(&mut self) -> impl Iterator<Item = &mut Entity> {
        self.entities
            .values_mut()
            .map(|entity| unsafe { &mut **entity })
    }

    #[inline]
    pub fn subgroup(&self, id: &Uuid) -> Option<&Group> {
        self.sub_groups.get(id).map(|group| unsafe { &**group })
    }

    #[inline]
    pub fn subgroup_mut(&mut self, id: &Uuid) -> Option<&mut Group> {
        self.sub_groups
            .get_mut(id)
            .map(|group| unsafe { &mut **group })
    }

    #[inline]
    pub fn subgroups(&self) -> impl Iterator<Item = &Group> {
        self.sub_groups.values().map(|group| unsafe { &**group })
    }

    #[inline]
    pub fn subgroups_mut(&mut self) -> impl Iterator<Item = &mut Group> {
        self.sub_groups
            .values_mut()
            .map(|group| unsafe { &mut **group })
    }

    pub fn update(&mut self) {
        if self.dirty {
            let mut boundings = Vec::new();

            for entity in self.entities_mut() {
                entity.update();
                if let Some(bounding) = entity.bounding() {
                    boundings.push(bounding.bounding());
                }
            }

            for group in self.subgroups_mut() {
                group.update();
                if let Some(bounding) = group.bounding() {
                    boundings.push(bounding.bounding());
                }
            }

            self.bounding = merge_bounding_volumes(boundings)
                .map(|bounding| CullingBoundingVolume::new(bounding));

            self.dirty = false;
        }
    }
}

pub struct Container {
    entities: *mut IndexMap<Uuid, *mut Entity>,
    groups: *mut IndexMap<Uuid, *mut Group>,
    root_group: *mut Group,

    dirty: *mut bool,
}

impl Drop for Container {
    fn drop(&mut self) {
        unsafe {
            let mut entities = Box::from_raw(self.entities);
            let mut groups = Box::from_raw(self.groups);
            for (_, entity) in entities.drain(..) {
                drop(Box::from_raw(entity).to_options());
            }
            for (_, group) in groups.drain(..) {
                drop(Box::from_raw(group));
            }
            drop(Box::from_raw(self.root_group));
            drop(Box::from_raw(self.dirty));
        }
    }
}

impl Container {
    pub fn new() -> Self {
        let mut container = Self {
            entities: Box::leak(Box::new(IndexMap::new())),
            groups: Box::leak(Box::new(IndexMap::new())),
            root_group: std::ptr::null_mut(),

            dirty: Box::leak(Box::new(true)),
        };
        container.root_group = Box::leak(Box::new(Group::new(
            Uuid::new_v4(),
            GroupOwner::Container(container.dirty),
        )));
        container
    }

    #[inline]
    pub fn set_dirty(&mut self) {
        unsafe {
            *self.dirty = true;
        }
    }

    #[inline]
    pub fn dirty(&self) -> bool {
        unsafe { *self.dirty }
    }

    #[inline]
    pub fn entities_len(&self) -> usize {
        unsafe { (*self.entities).len() }
    }

    #[inline]
    pub fn groups_len(&self) -> usize {
        unsafe { (*self.groups).len() }
    }

    pub fn add_entity(&mut self, entity_options: EntityOptions) -> &mut Entity {
        unsafe {
            let id = Uuid::new_v4();
            let entity = Entity::from_options(entity_options, id, self.root_group);
            (*self.entities).insert(id, entity);
            (*self.root_group).entities.insert(id, entity);
            self.set_dirty();
            &mut *entity
        }
    }

    pub fn add_entity_to_group(
        &mut self,
        entity_options: EntityOptions,
        group_id: &Uuid,
    ) -> Result<&mut Entity, Error> {
        unsafe {
            let Some(group) = (*self.groups).get_mut(group_id).map(|group| *group) else {
                return Err(Error::NoSuchGroup);
            };

            let id = Uuid::new_v4();
            let entity = Entity::from_options(entity_options, id, group);
            (*self.entities).insert(id, entity);
            (*group).entities.insert(id, entity);
            self.set_dirty();
            Ok(&mut *entity)
        }
    }

    pub fn remove_entity(&mut self, id: &Uuid) -> Option<EntityOptions> {
        unsafe {
            let entity = (*self.entities).swap_remove(id)?;
            let entity = Box::from_raw(entity);
            (*entity.group).entities.swap_remove(id);
            self.set_dirty();
            Some(entity.to_options())
        }
    }

    #[inline]
    pub fn entity(&self, id: &Uuid) -> Option<&Entity> {
        unsafe { (*self.entities).get(id).map(|entity| &**entity) }
    }

    #[inline]
    pub fn entity_mut(&mut self, id: &Uuid) -> Option<&mut Entity> {
        unsafe { (*self.entities).get_mut(id).map(|entity| &mut **entity) }
    }

    #[inline]
    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        unsafe { (*self.entities).values().map(|entity| &**entity) }
    }

    #[inline]
    pub fn entities_mut(&mut self) -> impl Iterator<Item = &mut Entity> {
        unsafe { (*self.entities).values_mut().map(|entity| &mut **entity) }
    }

    #[inline]
    pub fn entities_raw(&mut self) -> &mut IndexMap<Uuid, *mut Entity> {
        unsafe { &mut *self.entities }
    }

    #[inline]
    pub fn root_group(&self) -> &Group {
        unsafe { &*self.root_group }
    }

    #[inline]
    pub fn root_group_mut(&self) -> &mut Group {
        unsafe { &mut *self.root_group }
    }

    pub fn add_group(&mut self, group_options: GroupOptions) -> Result<(), Error> {
        unsafe {
            let mut queue: VecDeque<(GroupOptions, Option<&Uuid>)> = VecDeque::from_iter([(group_options, None)]);
            while let Some((group_options, super_group)) = queue.pop_front() {
                let group: *mut Group = match super_group {
                    Some(super_group) => self.create_group_in_group(super_group)?,
                    None => self.create_group(),
                };
                (*group).set_model_matrix(group_options.model_matrix);
                let group_id = (*group).id();

                for entity_options in group_options.entities {
                    self.add_entity_to_group(entity_options, group_id)?;
                }

                queue.extend(
                    group_options
                        .sub_groups
                        .into_iter()
                        .map(|subgroup_options| (subgroup_options, Some(group_id))),
                )
            }

            self.set_dirty()
        }

        Ok(())
    }

    pub fn create_group(&mut self) -> &mut Group {
        unsafe {
            let id = Uuid::new_v4();
            let group = Box::leak(Box::new(Group::new(id, GroupOwner::Group(self.root_group))));
            (*self.groups).insert(id, group);
            (*self.root_group).sub_groups.insert(id, group);
            self.set_dirty();

            group
        }
    }

    pub fn create_group_in_group(&mut self, group_id: &Uuid) -> Result<&mut Group, Error> {
        unsafe {
            let Some(super_group) = (*self.groups)
                .get_mut(group_id)
                .map(|super_group| *super_group)
            else {
                return Err(Error::NoSuchGroup);
            };

            let id = Uuid::new_v4();
            let group = Box::leak(Box::new(Group::new(id, GroupOwner::Group(super_group))));
            (*self.groups).insert(id, group);
            (*super_group).sub_groups.insert(id, group);
            self.set_dirty();

            Ok(group)
        }
    }

    pub fn remove_group(&mut self, id: &Uuid) -> Option<GroupOptions> {
        unsafe {
            let Some(group) = (*self.groups).swap_remove(id) else {
                return None;
            };

            let mut out = GroupOptions::new();

            let mut groups = VecDeque::from_iter([(group, &mut out as *mut GroupOptions)]);
            while let Some((group, me)) = groups.pop_front() {
                let group = Box::from_raw(group);
                (*self.groups).swap_remove(&group.id);

                for (entity_id, entity) in group.entities {
                    (*self.entities).swap_remove(&entity_id);
                    (*me).entities.push(Box::from_raw(entity).to_options());
                }
                (*me).model_matrix = group.model_matrix;

                for (_, subgroup) in group.sub_groups {
                    (*me).sub_groups.push(GroupOptions::new());
                    let subgroup_options = (*me)
                        .sub_groups
                        .get_unchecked_mut((*me).sub_groups.len() - 1);
                    groups.push_back((subgroup, subgroup_options));
                }
            }

            self.set_dirty();

            Some(out)
        }
    }

    pub fn remove_group_flatten(&mut self, id: &Uuid) -> Option<Vec<EntityOptions>> {
        unsafe {
            let Some(group) = (*self.groups).swap_remove(id) else {
                return None;
            };
            let mut group = Box::from_raw(group);
            let super_group = match group.owner {
                GroupOwner::Container(_) => unreachable!(),
                GroupOwner::Group(super_group) => &mut *super_group,
            };

            super_group.sub_groups.swap_remove(id);

            // iterates and removes subgroups
            let mut entities = group.entities.drain(..).collect::<VecDeque<_>>();
            let mut groups = group.sub_groups.drain(..).collect::<VecDeque<_>>();
            while let Some((group_id, group)) = groups.pop_front() {
                let group = Box::from_raw(group);
                entities.extend(group.entities);
                groups.extend(group.sub_groups);
                (*self.groups).swap_remove(&group_id);
            }

            // removes entities
            let mut entity_options = Vec::new();
            for (entity_id, entity) in entities {
                entity_options.push(Box::from_raw(entity).to_options());
                (*self.entities).swap_remove(&entity_id);
            }

            self.set_dirty();

            Some(entity_options)
        }
    }

    pub fn decompose_group(&mut self, id: &Uuid) {
        unsafe {
            let Some(group) = (*self.groups).swap_remove(id) else {
                return;
            };
            let mut group = Box::from_raw(group);

            let mut entities = group.entities.clone();
            let mut groups = group.sub_groups.drain(..).collect::<VecDeque<_>>();
            while let Some((group_id, group)) = groups.pop_front() {
                let group = Box::from_raw(group);
                entities.extend(group.entities);
                groups.extend(group.sub_groups);
                (*self.groups).swap_remove(&group_id);
            }

            (*self.root_group).entities.extend(entities);

            self.set_dirty();
        }
    }

    #[inline]
    pub fn group(&self, id: &Uuid) -> Option<&Group> {
        unsafe { (*self.groups).get(id).map(|group| &**group) }
    }

    #[inline]
    pub fn group_mut(&mut self, id: &Uuid) -> Option<&mut Group> {
        unsafe { (*self.groups).get_mut(id).map(|group| &mut **group) }
    }

    #[inline]
    pub fn groups_raw(&mut self) -> &mut IndexMap<Uuid, *mut Group> {
        unsafe { &mut *self.groups }
    }

    #[inline]
    pub fn groups(&self) -> impl Iterator<Item = &Group> {
        unsafe { (*self.groups).values().map(|group| &**group) }
    }

    #[inline]
    pub fn groups_mut(&mut self) -> impl Iterator<Item = &mut Group> {
        unsafe { (*self.groups).values_mut().map(|group| &mut **group) }
    }

    pub fn move_entity_out_of_group(&mut self, entity_id: &Uuid) -> Result<(), Error> {
        unsafe {
            let Some(entity) = (*self.entities).get(entity_id).map(|entity| *entity) else {
                return Err(Error::NoSuchEntity);
            };
            let entity = &mut *entity;

            if entity.group == self.root_group {
                Ok(())
            } else {
                let group = &mut *entity.group;
                group.entities.swap_remove(entity_id);
                (*self.root_group).entities.insert(*entity_id, entity);
                entity.group = self.root_group;
                self.set_dirty();
                Ok(())
            }
        }
    }

    pub fn move_entity_to_group(&mut self, entity_id: &Uuid, group_id: &Uuid) -> Result<(), Error> {
        unsafe {
            let Some(entity) = (*self.entities).get(entity_id) else {
                return Err(Error::NoSuchEntity);
            };
            let Some(to_group) = (*self.groups).get(group_id) else {
                return Err(Error::NoSuchGroup);
            };
            let entity = &mut **entity;
            let from_group = &mut *entity.group;
            let to_group = &mut **to_group;

            from_group.entities.swap_remove(entity_id);
            to_group.entities.insert(*entity_id, entity);
            entity.group = to_group;
            self.set_dirty();
        }

        Ok(())
    }

    pub fn move_group_out_of_group(&mut self, group_id: &Uuid) -> Result<(), Error> {
        unsafe {
            let Some(group) = (*self.groups).get(group_id) else {
                return Err(Error::NoSuchGroup);
            };
            let group = &mut **group;
            let super_group = match group.owner {
                GroupOwner::Container(_) => unreachable!(),
                GroupOwner::Group(super_group) => super_group,
            };

            if super_group == self.root_group {
                Ok(())
            } else {
                let super_group = &mut *super_group;
                super_group.sub_groups.swap_remove(group_id);
                (*self.root_group).sub_groups.insert(*group_id, group);
                group.owner = GroupOwner::Group(self.root_group);
                self.set_dirty();
                Ok(())
            }
        }
    }

    pub fn move_group_to_group(
        &mut self,
        group_id: &Uuid,
        to_group_id: &Uuid,
    ) -> Result<(), Error> {
        unsafe {
            let Some(group) = (*self.groups).get(group_id) else {
                return Err(Error::NoSuchGroup);
            };
            let Some(to_group) = (*self.groups).get(to_group_id) else {
                return Err(Error::NoSuchGroup);
            };
            let group = &mut **group;
            let super_group = match group.owner {
                GroupOwner::Container(_) => unreachable!(),
                GroupOwner::Group(super_group) => &mut *super_group,
            };
            let to_group = &mut **to_group;

            super_group.sub_groups.swap_remove(group_id);
            to_group.sub_groups.insert(*group_id, group);
            group.owner = GroupOwner::Group(to_group);
            self.set_dirty();
        }
        Ok(())
    }

    pub fn update(&mut self) {
        unsafe {
            (*self.root_group).update();
            (*self.dirty) = false;
        }
    }
}
