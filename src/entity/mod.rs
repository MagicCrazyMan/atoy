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
    subgroups: Vec<GroupOptions>,
}

impl GroupOptions {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            subgroups: Vec::new(),
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
        self.subgroups.as_ref()
    }

    pub fn sub_groups_mut(&mut self) -> &mut Vec<GroupOptions> {
        &mut self.subgroups
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
    compose_model_matrix: Mat4,
    compose_normal_matrix: Mat4,

    dirty_id: usize,
    dirty_id_updated: usize,
}

struct EntityChangeNotifiee(*mut Entity);

impl Notifiee<()> for EntityChangeNotifiee {
    fn notify(&mut self, _: &()) {
        unsafe {
            (*self.0).set_dirty();
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
            compose_model_matrix: Mat4::<f64>::new_identity(),
            compose_normal_matrix: Mat4::<f64>::new_identity(),

            dirty_id: 1,
            dirty_id_updated: 0,
        }));
        entity.set_geometry_boxed(options.geometry);
        entity.set_material_boxed(options.material);
        entity
    }

    fn to_options(mut self) -> EntityOptions {
        let geometry = self.take_geometry();
        let material = self.take_material();
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
            self.dirty_id = self.dirty_id.wrapping_add(1);
        }
    }

    pub fn is_dirty(&self) -> bool {
        unsafe { self.dirty_id != self.dirty_id_updated }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn geometry(&self) -> Option<&dyn Geometry> {
        match self.geometry.as_ref() {
            Some((geometry, _, _)) => Some(geometry.as_ref()),
            None => None,
        }
    }

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
            let notifying = geometry
                .notifier()
                .register(EntityChangeNotifiee(std::ptr::addr_of_mut!(*self)));
            let bounding = geometry
                .bounding_volume()
                .map(|bounding| bounding.as_ref().transform(self.compose_model_matrix))
                .map(|bounding| CullingBoundingVolume::new(bounding));
            self.geometry = Some((geometry, bounding, notifying));
        }

        self.set_dirty();

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
                .register(EntityChangeNotifiee(std::ptr::addr_of_mut!(*self)));
            self.material = Some((material, notifying));
        }

        self.set_dirty();

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

    pub fn model_matrix(&self) -> &Mat4 {
        &self.model_matrix
    }

    pub fn set_model_matrix(&mut self, model_matrix: Mat4) {
        self.model_matrix = model_matrix;
        self.set_dirty();
    }

    pub fn bounding(&self) -> Option<&CullingBoundingVolume> {
        self.geometry
            .as_ref()
            .and_then(|(_, bounding, _)| bounding.as_ref())
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
            self.update_matrices();
            self.update_bounding();
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

pub struct Group {
    id: Uuid,
    parent: Option<*mut Group>,
    container: NonDropContainer,
    entities: IndexMap<Uuid, *mut Entity>,
    subgroups: IndexMap<Uuid, *mut Group>,

    model_matrix: Mat4,
    compose_model_matrix: Mat4,
    bounding: Option<CullingBoundingVolume>,

    dirty_id: usize,
    dirty_id_updated: usize,
}

impl Group {
    fn new(id: Uuid, parent: Option<*mut Group>, container: NonDropContainer) -> Self {
        Self {
            id,
            parent,
            container,
            entities: IndexMap::new(),
            subgroups: IndexMap::new(),

            model_matrix: Mat4::<f64>::new_identity(),
            compose_model_matrix: Mat4::<f64>::new_identity(),
            bounding: None,

            dirty_id: 1,
            dirty_id_updated: 0,
        }
    }

    pub fn set_dirty(&mut self) {
        unsafe {
            self.dirty_id = self.dirty_id.wrapping_add(1);
            Container::from(self.container).set_dirty();
            // let mut subgroups = self.subgroups.values().collect::<VecDeque<_>>();
            // while let Some(subgroup) = subgroups.pop_front() {
            //     (**subgroup).dirty = true;
            //     (**subgroup).dirty_id.wrapping_add(1);
            //     subgroups.extend((**subgroup).subgroups.values());
            // }
        }
    }

    pub fn is_dirty(&self) -> bool {
        unsafe { self.dirty_id != self.dirty_id_updated }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn parent(&self) -> Option<&Group> {
        unsafe { self.parent.map(|parent| &*parent) }
    }

    pub fn entities_len(&self) -> usize {
        self.entities.len()
    }

    pub fn entities_hierarchy_len(&self) -> usize {
        unsafe {
            let mut entities_len = self.entities.len();
            let mut groups = self.subgroups.values().collect::<VecDeque<_>>();
            while let Some(group) = groups.pop_front() {
                entities_len += (**group).entities.len();
                groups.extend((**group).subgroups.values());
            }
            entities_len
        }
    }

    pub fn model_matrix(&self) -> &Mat4 {
        &self.model_matrix
    }

    pub fn set_model_matrix(&mut self, model_matrix: Mat4) {
        self.model_matrix = model_matrix;
        self.set_dirty();
    }

    pub fn compose_model_matrix(&self) -> &Mat4 {
        &self.compose_model_matrix
    }

    pub fn bounding(&self) -> Option<&CullingBoundingVolume> {
        self.bounding.as_ref()
    }

    pub fn add_entity(&mut self, entity_options: EntityOptions) -> &mut Entity {
        unsafe {
            let id = Uuid::new_v4();
            let entity = Entity::from_options(entity_options, id, self);
            self.entities.insert(id, entity);
            (*self.container.entities).insert(id, entity);
            self.set_dirty();
            &mut *entity
        }
    }

    pub fn remove_entity(&mut self, id: &Uuid) -> Option<EntityOptions> {
        unsafe {
            let entity = self.entities.swap_remove(id)?;
            let entity = Box::from_raw(entity);
            (*self.container.entities).swap_remove(id);
            self.set_dirty();
            Some(entity.to_options())
        }
    }

    pub fn entity(&self, id: &Uuid) -> Option<&Entity> {
        self.entities.get(id).map(|entity| unsafe { &**entity })
    }

    pub fn entity_mut(&mut self, id: &Uuid) -> Option<&mut Entity> {
        self.entities
            .get_mut(id)
            .map(|entity| unsafe { &mut **entity })
    }

    pub fn entities_iter(&self) -> impl Iterator<Item = &Entity> {
        self.entities.values().map(|entity| unsafe { &**entity })
    }

    pub fn entities_iter_mut(&mut self) -> impl Iterator<Item = &mut Entity> {
        self.entities
            .values_mut()
            .map(|entity| unsafe { &mut **entity })
    }

    pub fn add_subgroup(&mut self, group_options: GroupOptions) -> Result<(), Error> {
        unsafe {
            let mut rollings: VecDeque<(GroupOptions, *mut Self)> =
                VecDeque::from_iter([(group_options, self as *mut Self)]);
            while let Some((group_options, parent)) = rollings.pop_front() {
                let group_id = Uuid::new_v4();
                let group = Box::leak(Box::new(Self::new(
                    group_id,
                    Some(parent),
                    (*parent).container,
                )));
                group.set_model_matrix(group_options.model_matrix);
                (*parent).subgroups.insert(group_id, group);
                (*self.container.groups).insert(group_id, group);

                for entity_options in group_options.entities {
                    let entity_id = Uuid::new_v4();
                    let entity = Entity::from_options(entity_options, entity_id, group);
                    group.entities.insert(entity_id, entity);
                    (*self.container.entities).insert(entity_id, entity);
                }

                rollings.extend(
                    group_options
                        .subgroups
                        .into_iter()
                        .map(|subgroup_options| (subgroup_options, group as *mut Self)),
                )
            }

            self.set_dirty();

            Ok(())
        }
    }

    pub fn create_subgroup(&mut self) -> &mut Self {
        unsafe {
            let id = Uuid::new_v4();
            let subgroup = Box::leak(Box::new(Self::new(id, Some(self), self.container)));
            self.subgroups.insert(id, subgroup);
            (*self.container.groups).insert(id, subgroup);
            self.set_dirty();
            subgroup
        }
    }

    pub fn remove_subgroup(&mut self, id: &Uuid) -> Option<GroupOptions> {
        unsafe {
            let Some(group) = self.subgroups.swap_remove(id) else {
                return None;
            };

            let mut out = GroupOptions::new();

            let mut rollings = VecDeque::from_iter([(group, &mut out as *mut GroupOptions)]);
            while let Some((group, out)) = rollings.pop_front() {
                let group = Box::from_raw(group);
                (*self.container.groups).swap_remove(&group.id);

                for (entity_id, entity) in group.entities {
                    (*self.container.entities).swap_remove(&entity_id);
                    (*out).entities.push(Box::from_raw(entity).to_options());
                }

                (*out).model_matrix = group.model_matrix;

                for (_, subgroup) in group.subgroups {
                    (*out).subgroups.push(GroupOptions::new());
                    rollings.push_back((subgroup, (*out).subgroups.last_mut().unwrap()));
                }
            }

            self.set_dirty();

            Some(out)
        }
    }

    pub fn remove_subgroup_flatten(&mut self, id: &Uuid) -> Option<Vec<EntityOptions>> {
        unsafe {
            let Some(group) = self.subgroups.swap_remove(id) else {
                return None;
            };
            let mut group = Box::from_raw(group);

            // removes from parent
            if let Some(parent) = self.parent {
                (*parent).subgroups.swap_remove(id);
            }

            // iterates and removes subgroups and entities
            let mut entity_options = Vec::new();
            let mut groups = group.subgroups.drain(..).collect::<VecDeque<_>>();
            while let Some((group_id, group)) = groups.pop_front() {
                (*self.container.groups).swap_remove(&group_id);
                let mut group = Box::from_raw(group);
                groups.extend(group.subgroups);

                for (entity_id, entity) in group.entities.drain(..) {
                    (*self.container.entities).swap_remove(&entity_id);
                    let entity = Box::from_raw(entity);
                    entity_options.push(entity.to_options());
                }
            }

            self.set_dirty();

            Some(entity_options)
        }
    }

    pub fn decompose(mut self) {
        unsafe {
            let Some(parent) = self.parent else {
                return;
            };
            let me = (*parent).subgroups.swap_remove(&self.id).unwrap();

            self.subgroups.values_mut().for_each(|subgroup| {
                (**subgroup).parent = Some(parent);
            });
            self.entities.values_mut().for_each(|entity| {
                (**entity).group = parent;
            });
            (*parent).entities.extend(self.entities);
            (*parent).subgroups.extend(self.subgroups);

            drop(Box::from(me));
        }
    }

    pub fn subgroup(&self, id: &Uuid) -> Option<&Group> {
        self.subgroups.get(id).map(|group| unsafe { &**group })
    }

    pub fn subgroup_mut(&mut self, id: &Uuid) -> Option<&mut Group> {
        self.subgroups
            .get_mut(id)
            .map(|group| unsafe { &mut **group })
    }

    pub fn subgroups_iter(&self) -> impl Iterator<Item = &Group> {
        self.subgroups.values().map(|group| unsafe { &**group })
    }

    pub fn subgroups_iter_mut(&mut self) -> impl Iterator<Item = &mut Group> {
        self.subgroups
            .values_mut()
            .map(|group| unsafe { &mut **group })
    }

    pub fn update(&mut self) {
        let mut boundings = Vec::new();

        for entity in self.entities_iter_mut() {
            entity.update();
            if let Some(bounding) = entity.bounding() {
                boundings.push(bounding.bounding());
            }
        }

        for group in self.subgroups_iter_mut() {
            group.update();
            if let Some(bounding) = group.bounding() {
                boundings.push(bounding.bounding());
            }
        }

        self.bounding =
            merge_bounding_volumes(boundings).map(|bounding| CullingBoundingVolume::new(bounding));
    }
}

pub struct Container {
    entities: *mut IndexMap<Uuid, *mut Entity>,
    groups: *mut IndexMap<Uuid, *mut Group>,
    root_group: *mut Group,

    dirty_id: *mut usize,
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
            drop(Box::from_raw(self.dirty_id));
        }
    }
}

impl Container {
    pub fn new() -> Self {
        let entities = Box::leak(Box::new(IndexMap::new()));
        let groups = Box::leak(Box::new(IndexMap::new()));
        let dirty = Box::leak(Box::new(true));
        let dirty_id = Box::leak(Box::new(0));
        let mut container = NonDropContainer {
            entities,
            groups,
            root_group: std::ptr::null_mut(),
            dirty_id,
        };
        let root_group = Box::leak(Box::new(Group::new(Uuid::new_v4(), None, container)));
        container.root_group = root_group;

        Self {
            entities,
            groups,
            root_group,

            dirty_id,
        }
    }

    pub fn set_dirty(&mut self) {
        unsafe {
            (*self.dirty_id).wrapping_add(1);
        }
    }

    pub fn dirty_id(&self) -> usize {
        unsafe { *self.dirty_id }
    }

    pub fn entities_len(&self) -> usize {
        unsafe { (*self.entities).len() }
    }

    pub fn groups_len(&self) -> usize {
        unsafe { (*self.groups).len() }
    }

    pub fn entity(&self, id: &Uuid) -> Option<&Entity> {
        unsafe { (*self.entities).get(id).map(|entity| &**entity) }
    }

    pub fn entity_mut(&mut self, id: &Uuid) -> Option<&mut Entity> {
        unsafe { (*self.entities).get_mut(id).map(|entity| &mut **entity) }
    }

    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        unsafe { (*self.entities).values().map(|entity| &**entity) }
    }

    pub fn entities_mut(&mut self) -> impl Iterator<Item = &mut Entity> {
        unsafe { (*self.entities).values_mut().map(|entity| &mut **entity) }
    }

    pub fn entities_raw(&mut self) -> &mut IndexMap<Uuid, *mut Entity> {
        unsafe { &mut *self.entities }
    }

    pub fn root_group(&self) -> &Group {
        unsafe { &*self.root_group }
    }

    pub fn root_group_mut(&self) -> &mut Group {
        unsafe { &mut *self.root_group }
    }

    pub fn group(&self, id: &Uuid) -> Option<&Group> {
        unsafe { (*self.groups).get(id).map(|group| &**group) }
    }

    pub fn group_mut(&mut self, id: &Uuid) -> Option<&mut Group> {
        unsafe { (*self.groups).get_mut(id).map(|group| &mut **group) }
    }

    pub fn groups_raw(&mut self) -> &mut IndexMap<Uuid, *mut Group> {
        unsafe { &mut *self.groups }
    }

    pub fn groups(&self) -> impl Iterator<Item = &Group> {
        unsafe { (*self.groups).values().map(|group| &**group) }
    }

    pub fn groups_mut(&mut self) -> impl Iterator<Item = &mut Group> {
        unsafe { (*self.groups).values_mut().map(|group| &mut **group) }
    }

    pub fn move_entity_to_root(&mut self, entity_id: &Uuid) -> Result<(), Error> {
        unsafe {
            let Some(entity) = (*self.entities).get(entity_id) else {
                return Err(Error::NoSuchEntity);
            };
            let entity = &mut **entity;

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

    pub fn move_group_to_root(&mut self, group_id: &Uuid) -> Result<(), Error> {
        unsafe {
            let Some(group) = (*self.groups).get(group_id) else {
                return Err(Error::NoSuchGroup);
            };
            let group = &mut **group;
            let parent = group.parent.unwrap();

            if parent == self.root_group {
                return Ok(());
            }

            (*parent).subgroups.swap_remove(&group.id);
            (*self.root_group).subgroups.insert(group.id, group);
            group.parent = Some(self.root_group);
            self.set_dirty();

            Ok(())
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
            let parent = group.parent.unwrap();

            if parent == *to_group {
                return Ok(());
            }

            let to_group = &mut **to_group;

            (*parent).subgroups.swap_remove(group_id);
            to_group.subgroups.insert(*group_id, group);
            group.parent = Some(to_group);
            self.set_dirty();
        }
        Ok(())
    }

    pub fn update(&mut self) {
        unsafe {
            (*self.root_group).update();
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct NonDropContainer {
    entities: *mut IndexMap<Uuid, *mut Entity>,
    groups: *mut IndexMap<Uuid, *mut Group>,
    root_group: *mut Group,

    dirty_id: *mut usize,
}

impl From<NonDropContainer> for Container {
    fn from(value: NonDropContainer) -> Self {
        Container {
            entities: value.entities,
            groups: value.groups,
            root_group: value.root_group,
            dirty_id: value.dirty_id,
        }
    }
}
