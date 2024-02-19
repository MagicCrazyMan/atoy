use std::{
    any::Any,
    cell::RefCell,
    collections::VecDeque,
    iter::FromIterator,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use gl_matrix4rust::mat4::Mat4;
use indexmap::IndexMap;
use log::warn;
use uuid::Uuid;

use crate::{
    bounding::{merge_bounding_volumes, CullingBoundingVolume},
    error::Error,
    geometry::Geometry,
    material::webgl::StandardMaterial,
    readonly::Readonly,
    renderer::webgl::{
        attribute::AttributeValue,
        uniform::{UniformBlockValue, UniformValue},
    },
};

pub trait EntityBase {
    fn model_matrix(&self) -> Readonly<'_, Mat4>;

    fn geometry(&self) -> Option<&dyn Geometry>;

    fn geometry_mut(&mut self) -> Option<&mut dyn Geometry>;

    fn material(&self) -> Option<&dyn StandardMaterial>;

    fn material_mut(&mut self) -> Option<&mut dyn StandardMaterial>;

    fn attribute_value(&self, name: &str) -> Option<Readonly<'_, AttributeValue>>;

    fn uniform_value(&self, name: &str) -> Option<Readonly<'_, UniformValue>>;

    fn uniform_block_value(&self, name: &str) -> Option<Readonly<'_, UniformBlockValue>>;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct SimpleEntityBase {
    model_matrix: Mat4,
    geometry: Option<Box<dyn Geometry>>,
    material: Option<Box<dyn StandardMaterial>>,
}

impl SimpleEntityBase {
    pub fn new() -> Self {
        Self {
            model_matrix: Mat4::<f64>::new_identity(),
            geometry: None,
            material: None,
        }
    }

    pub fn set_model_matrix(&mut self, model_matrix: Mat4) {
        self.model_matrix = model_matrix;
    }

    pub fn set_geometry<G>(&mut self, geometry: Option<G>)
    where
        G: Geometry + 'static,
    {
        self.geometry = geometry.map(|geometry| Box::new(geometry) as Box<dyn Geometry>);
    }

    pub fn set_material<M>(&mut self, material: Option<M>)
    where
        M: StandardMaterial + 'static,
    {
        self.material = material.map(|material| Box::new(material) as Box<dyn StandardMaterial>);
    }
}

impl EntityBase for SimpleEntityBase {
    fn model_matrix(&self) -> Readonly<'_, Mat4> {
        Readonly::Borrowed(&self.model_matrix)
    }

    fn geometry(&self) -> Option<&dyn Geometry> {
        self.geometry.as_deref()
    }

    fn geometry_mut(&mut self) -> Option<&mut dyn Geometry> {
        match self.geometry.as_mut() {
            Some(geometry) => Some(&mut **geometry),
            None => None,
        }
    }

    fn material(&self) -> Option<&dyn StandardMaterial> {
        self.material.as_deref()
    }

    fn material_mut(&mut self) -> Option<&mut dyn StandardMaterial> {
        match self.material.as_mut() {
            Some(material) => Some(&mut **material),
            None => None,
        }
    }

    fn attribute_value(&self, _: &str) -> Option<Readonly<'_, AttributeValue>> {
        None
    }

    fn uniform_value(&self, _: &str) -> Option<Readonly<'_, UniformValue>> {
        None
    }

    fn uniform_block_value(&self, _: &str) -> Option<Readonly<'_, UniformBlockValue>> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct GroupOptions {
    model_matrix: Mat4,
    entities: Vec<Box<dyn EntityBase>>,
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

    pub fn entities(&self) -> &Vec<Box<dyn EntityBase>> {
        self.entities.as_ref()
    }

    pub fn entities_mut(&mut self) -> &mut Vec<Box<dyn EntityBase>> {
        &mut self.entities
    }

    pub fn sub_groups(&self) -> &Vec<GroupOptions> {
        self.subgroups.as_ref()
    }

    pub fn sub_groups_mut(&mut self) -> &mut Vec<GroupOptions> {
        &mut self.subgroups
    }
}

const ENTITY_DIRTY_FIELD_CLEAR: u8 = 0b00000000;
const ENTITY_DIRTY_FIELD_MODEL_MATRIX_DIRTY: u8 = 0b00000001;
const ENTITY_DIRTY_FIELD_MATERIAL_DIRTY: u8 = 0b00000010;
const ENTITY_DIRTY_FIELD_GEOMETRY_DIRTY: u8 = 0b00000100;

pub struct Entity {
    id: Uuid,
    base: Box<dyn EntityBase>,

    compose_model_matrix: Mat4,
    compose_normal_matrix: Mat4,
    bounding: Option<CullingBoundingVolume>,

    group: *mut Group,
    container: *mut ContainerInner,
    me: Rc<RefCell<*mut Self>>,

    dirty_field: u8,
}

impl Deref for Entity {
    type Target = Box<dyn EntityBase>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for Entity {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Entity {
    fn new(id: Uuid, base: Box<dyn EntityBase>, group: &mut Group) -> *mut Entity {
        let entity = Box::leak(Box::new(Entity {
            id,
            base,

            compose_model_matrix: Mat4::<f64>::new_identity(),
            compose_normal_matrix: Mat4::<f64>::new_identity(),
            bounding: None,

            group,
            container: group.container,
            me: Rc::new(RefCell::new(std::ptr::null_mut())),

            dirty_field: ENTITY_DIRTY_FIELD_MODEL_MATRIX_DIRTY
                | ENTITY_DIRTY_FIELD_MATERIAL_DIRTY
                | ENTITY_DIRTY_FIELD_GEOMETRY_DIRTY,
        }));
        *entity.me.borrow_mut() = entity;
        entity
    }

    fn take(self: Box<Self>) -> Box<dyn EntityBase> {
        *self.me.borrow_mut() = std::ptr::null_mut();
        self.base
    }

    pub(crate) fn me(&self) -> &Rc<RefCell<*mut Self>> {
        &self.me
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn group(&self) -> &Group {
        unsafe { &*self.group }
    }

    pub fn base(&self) -> &dyn EntityBase {
        self.base.as_ref()
    }

    pub fn base_mut(&mut self) -> &mut dyn EntityBase {
        self.base.as_mut()
    }

    pub fn compose_model_matrix(&self) -> &Mat4 {
        &self.compose_model_matrix
    }

    pub fn compose_normal_matrix(&self) -> &Mat4 {
        &self.compose_normal_matrix
    }

    pub fn bounding(&self) -> Option<&CullingBoundingVolume> {
        self.bounding.as_ref()
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty_field != ENTITY_DIRTY_FIELD_CLEAR
    }

    pub fn mark_model_matrix_dirty(&mut self) {
        self.mark_dirty_inner(ENTITY_DIRTY_FIELD_MODEL_MATRIX_DIRTY);
    }

    pub fn mark_material_dirty(&mut self) {
        self.mark_dirty_inner(ENTITY_DIRTY_FIELD_MATERIAL_DIRTY);
    }

    pub fn mark_geometry_dirty(&mut self) {
        self.mark_dirty_inner(ENTITY_DIRTY_FIELD_GEOMETRY_DIRTY);
    }

    fn mark_dirty_inner(&mut self, field: u8) {
        unsafe {
            self.dirty_field |= field;

            if field & (ENTITY_DIRTY_FIELD_MODEL_MATRIX_DIRTY | ENTITY_DIRTY_FIELD_GEOMETRY_DIRTY)
                != 0
            {
                (*self.group).mark_bounding_volume_dirty();
            }

            (*self.container).mark_dirty();
        }
    }

    fn refresh(&mut self) {
        unsafe {
            if !self.is_dirty() {
                return;
            }

            self.refresh_compose_matrices();
            self.refresh_bounding_volume();

            self.dirty_field = ENTITY_DIRTY_FIELD_CLEAR;
        }
    }

    unsafe fn refresh_compose_matrices(&mut self) {
        if self.dirty_field & ENTITY_DIRTY_FIELD_MODEL_MATRIX_DIRTY == 0 {
            return;
        }

        self.compose_model_matrix = (*self.group).compose_model_matrix * *self.base.model_matrix();
        self.compose_normal_matrix = self.compose_model_matrix.clone();
        self.compose_normal_matrix
            .invert_in_place()
            .expect("invert a matrix with zero determinant is not allowed");
        self.compose_normal_matrix.transpose_in_place();
    }

    fn refresh_bounding_volume(&mut self) {
        if self.dirty_field
            & (ENTITY_DIRTY_FIELD_MODEL_MATRIX_DIRTY | ENTITY_DIRTY_FIELD_GEOMETRY_DIRTY)
            == 0
        {
            return;
        }

        let compose_model_matrix = self.compose_model_matrix;
        self.bounding = self
            .base
            .geometry()
            .and_then(|geom| geom.bounding_volume())
            .map(|bounding| {
                CullingBoundingVolume::new(bounding.as_ref().transform(compose_model_matrix))
            });
    }
}

const GROUP_DIRTY_FIELD_CLEAR: u8 = 0b00000000;
const GROUP_DIRTY_FIELD_MODEL_MATRIX_DIRTY: u8 = 0b00000001;
const GROUP_DIRTY_FIELD_BOUNDING_VOLUME_DIRTY: u8 = 0b00000010;

pub struct Group {
    id: Uuid,
    parent: Option<*mut Group>,
    container: *mut ContainerInner,
    entities: IndexMap<Uuid, *mut Entity>,
    subgroups: IndexMap<Uuid, *mut Group>,

    model_matrix: Mat4,
    compose_model_matrix: Mat4,

    enable_bounding: bool,
    bounding: Option<CullingBoundingVolume>,

    dirty_field: u8,
}

impl Group {
    fn new(id: Uuid, parent: Option<*mut Group>, container: *mut ContainerInner) -> Self {
        Self {
            id,
            parent,
            container,
            entities: IndexMap::new(),
            subgroups: IndexMap::new(),

            model_matrix: Mat4::<f64>::new_identity(),
            compose_model_matrix: Mat4::<f64>::new_identity(),

            enable_bounding: false,
            bounding: None,

            dirty_field: GROUP_DIRTY_FIELD_MODEL_MATRIX_DIRTY
                | GROUP_DIRTY_FIELD_BOUNDING_VOLUME_DIRTY,
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn parent(&self) -> Option<&Group> {
        unsafe { self.parent.map(|parent| &*parent) }
    }

    pub fn model_matrix(&self) -> &Mat4 {
        &self.model_matrix
    }

    pub fn set_model_matrix(&mut self, model_matrix: Mat4) {
        self.model_matrix = model_matrix;
        self.mark_model_matrix_dirty();
    }

    pub fn compose_model_matrix(&self) -> &Mat4 {
        &self.compose_model_matrix
    }

    pub fn bounding_enabled(&self) -> bool {
        self.enable_bounding
    }

    pub fn enable_bounding(&mut self) {
        self.enable_bounding = true;
        self.bounding = None;
        self.mark_bounding_volume_dirty();
    }

    pub fn disable_bounding(&mut self) {
        self.enable_bounding = false;
        self.bounding = None;
        self.mark_bounding_volume_dirty();
    }

    pub fn bounding(&self) -> Option<&CullingBoundingVolume> {
        self.bounding.as_ref()
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

    pub fn add_entity<E>(&mut self, entity_base: E) -> &mut Entity
    where
        E: EntityBase + 'static,
    {
        self.add_entity_boxed(Box::new(entity_base))
    }

    pub fn add_entity_boxed(&mut self, entity_base: Box<dyn EntityBase>) -> &mut Entity {
        unsafe {
            let id = Uuid::new_v4();
            let entity = Entity::new(id, entity_base, self);
            self.entities.insert(id, entity);
            (*self.container).entities.insert(id, entity);
            self.mark_bounding_volume_dirty();
            &mut *entity
        }
    }

    pub fn remove_entity(&mut self, id: &Uuid) -> Option<Box<dyn EntityBase>> {
        unsafe {
            let entity = self.entities.swap_remove(id)?;
            (*self.container).entities.swap_remove(id);
            self.mark_bounding_volume_dirty();
            Some(Box::from_raw(entity).take())
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

    pub fn subgroups_len(&self) -> usize {
        self.subgroups.len()
    }

    pub fn subgroups_hierarchy_len(&self) -> usize {
        unsafe {
            let mut subgroups_len = self.subgroups.len();
            let mut groups = self.subgroups.values().collect::<VecDeque<_>>();
            while let Some(group) = groups.pop_front() {
                subgroups_len += (**group).subgroups.len();
                groups.extend((**group).subgroups.values());
            }
            subgroups_len
        }
    }

    pub fn add_subgroup(&mut self, group_options: GroupOptions) -> &mut Self {
        unsafe {
            let mut subgroup: *mut Group = std::ptr::null_mut();
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
                (*self.container).groups.insert(group_id, group);

                for entity_options in group_options.entities {
                    let entity_id = Uuid::new_v4();
                    let entity = Entity::new(entity_id, entity_options, group);
                    group.entities.insert(entity_id, entity);
                    (*self.container).entities.insert(entity_id, entity);
                }

                if subgroup.is_null() {
                    subgroup = group;
                }

                rollings.extend(
                    group_options
                        .subgroups
                        .into_iter()
                        .map(|subgroup_options| (subgroup_options, group as *mut Self)),
                )
            }

            self.mark_bounding_volume_dirty();

            &mut *subgroup
        }
    }

    pub fn create_subgroup(&mut self) -> &mut Self {
        unsafe {
            let id = Uuid::new_v4();
            let subgroup = Box::leak(Box::new(Self::new(id, Some(self), self.container)));
            self.subgroups.insert(id, subgroup);
            (*self.container).groups.insert(id, subgroup);
            self.mark_bounding_volume_dirty();
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
                (*self.container).groups.swap_remove(&group.id);

                for (entity_id, entity) in group.entities {
                    (*self.container).entities.swap_remove(&entity_id);
                    (*out).entities.push(Box::from_raw(entity).take());
                }

                (*out).model_matrix = group.model_matrix;

                for (_, subgroup) in group.subgroups {
                    (*out).subgroups.push(GroupOptions::new());
                    rollings.push_back((subgroup, (*out).subgroups.last_mut().unwrap()));
                }
            }

            self.mark_bounding_volume_dirty();

            Some(out)
        }
    }

    pub fn remove_subgroup_flatten(&mut self, id: &Uuid) -> Option<Vec<Box<dyn EntityBase>>> {
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
                (*self.container).groups.swap_remove(&group_id);
                let mut group = Box::from_raw(group);
                groups.extend(group.subgroups);

                for (entity_id, entity) in group.entities.drain(..) {
                    (*self.container).entities.swap_remove(&entity_id);
                    entity_options.push(Box::from_raw(entity).take());
                }
            }

            self.mark_bounding_volume_dirty();

            Some(entity_options)
        }
    }

    pub fn decompose(mut self) {
        unsafe {
            let Some(parent) = self.parent else {
                warn!(
                    target: "Group",
                    "decompose root group has no effect"
                );
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

    pub fn is_dirty(&self) -> bool {
        self.dirty_field != GROUP_DIRTY_FIELD_CLEAR
    }

    fn mark_model_matrix_dirty(&mut self) {
        self.mark_dirty_inner(GROUP_DIRTY_FIELD_MODEL_MATRIX_DIRTY)
    }

    fn mark_bounding_volume_dirty(&mut self) {
        self.mark_dirty_inner(GROUP_DIRTY_FIELD_BOUNDING_VOLUME_DIRTY)
    }

    fn mark_dirty_inner(&mut self, field: u8) {
        unsafe {
            self.dirty_field |= field;

            // updates all children if compose model matrix changed
            if field & GROUP_DIRTY_FIELD_MODEL_MATRIX_DIRTY != 0 {
                for entity in self.entities_iter_mut() {
                    entity.dirty_field |= ENTITY_DIRTY_FIELD_MODEL_MATRIX_DIRTY;
                }

                let mut subgroups = self.subgroups.values_mut().collect::<VecDeque<_>>();
                while let Some(subgroup) = subgroups.pop_front() {
                    (**subgroup).dirty_field |= GROUP_DIRTY_FIELD_MODEL_MATRIX_DIRTY;
                    for entity in (**subgroup).entities_iter_mut() {
                        entity.dirty_field |= ENTITY_DIRTY_FIELD_MODEL_MATRIX_DIRTY;
                    }

                    subgroups.extend((**subgroup).subgroups.values_mut());
                }
            }

            if field
                & (GROUP_DIRTY_FIELD_MODEL_MATRIX_DIRTY | GROUP_DIRTY_FIELD_BOUNDING_VOLUME_DIRTY)
                != 0
            {
                let mut parent = self.parent;
                while let Some(p) = parent.take() {
                    (*p).dirty_field |= GROUP_DIRTY_FIELD_BOUNDING_VOLUME_DIRTY;
                    parent = (*p).parent;
                }
            }

            (*self.container).mark_dirty();
        }
    }

    fn refresh(&mut self) {
        if self.is_dirty() {
            self.refresh_compose_matrices();

            if self.enable_bounding {
                self.refresh_children_and_bounding_volume();
            } else {
                self.refresh_children();
                self.bounding = None;
            }

            self.dirty_field = GROUP_DIRTY_FIELD_CLEAR;
        } else {
            self.refresh_children();
        }
    }

    fn refresh_compose_matrices(&mut self) {
        if self.dirty_field & GROUP_DIRTY_FIELD_MODEL_MATRIX_DIRTY == 0 {
            return;
        }

        self.compose_model_matrix = match self.parent() {
            Some(parent) => parent.compose_model_matrix * self.model_matrix,
            None => self.model_matrix.clone(),
        };
    }

    fn refresh_children_and_bounding_volume(&mut self) {
        if self.dirty_field
            & (GROUP_DIRTY_FIELD_MODEL_MATRIX_DIRTY | GROUP_DIRTY_FIELD_BOUNDING_VOLUME_DIRTY)
            == 0
        {
            return;
        }

        let mut boundings = Vec::new();
        for entity in self.entities_iter_mut() {
            entity.refresh();
            if let Some(bounding) = entity.bounding() {
                boundings.push(bounding.bounding());
            }
        }
        for subgroup in self.subgroups_iter_mut() {
            subgroup.refresh();
            if let Some(bounding) = subgroup.bounding() {
                boundings.push(bounding.bounding());
            }
        }
        self.bounding =
            merge_bounding_volumes(boundings).map(|bounding| CullingBoundingVolume::new(bounding));
    }

    fn refresh_children(&mut self) {
        for entity in self.entities_iter_mut() {
            entity.refresh();
        }
        for subgroup in self.subgroups_iter_mut() {
            subgroup.refresh();
        }
    }
}

struct ContainerInner {
    entities: IndexMap<Uuid, *mut Entity>,
    groups: IndexMap<Uuid, *mut Group>,
    root_group: *mut Group,

    dirty: bool,
}

impl Drop for ContainerInner {
    fn drop(&mut self) {
        unsafe {
            for entity in self.entities.values() {
                drop(Box::from_raw(*entity));
            }
            for group in self.groups.values() {
                drop(Box::from_raw(*group));
            }
            drop(Box::from_raw(self.root_group));
        }
    }
}

impl ContainerInner {
    fn new() -> *mut Self {
        let me = Box::leak(Box::new(Self {
            entities: IndexMap::new(),
            groups: IndexMap::new(),
            root_group: std::ptr::null_mut(),

            dirty: false,
        }));
        let root_group = Box::leak(Box::new(Group::new(Uuid::new_v4(), None, me)));
        me.root_group = root_group;
        me
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

pub struct Container(*mut ContainerInner);

impl Drop for Container {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.0));
        }
    }
}

impl Container {
    pub fn new() -> Self {
        Self(ContainerInner::new())
    }

    pub fn entities_len(&self) -> usize {
        unsafe { (*self.0).entities.len() }
    }

    pub fn groups_len(&self) -> usize {
        unsafe { (*self.0).groups.len() }
    }

    pub fn entity(&self, id: &Uuid) -> Option<&Entity> {
        unsafe { (*self.0).entities.get(id).map(|entity| &**entity) }
    }

    pub fn entity_mut(&mut self, id: &Uuid) -> Option<&mut Entity> {
        unsafe { (*self.0).entities.get_mut(id).map(|entity| &mut **entity) }
    }

    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        unsafe { (*self.0).entities.values().map(|entity| &**entity) }
    }

    pub fn entities_mut(&mut self) -> impl Iterator<Item = &mut Entity> {
        unsafe { (*self.0).entities.values_mut().map(|entity| &mut **entity) }
    }

    pub fn entities_raw(&mut self) -> &mut IndexMap<Uuid, *mut Entity> {
        unsafe { &mut (*self.0).entities }
    }

    pub fn root_group(&self) -> &Group {
        unsafe { &(*(*self.0).root_group) }
    }

    pub fn root_group_mut(&self) -> &mut Group {
        unsafe { &mut (*(*self.0).root_group) }
    }

    pub fn group(&self, id: &Uuid) -> Option<&Group> {
        unsafe { (*self.0).groups.get(id).map(|group| &**group) }
    }

    pub fn group_mut(&mut self, id: &Uuid) -> Option<&mut Group> {
        unsafe { (*self.0).groups.get_mut(id).map(|group| &mut **group) }
    }

    pub fn groups_raw(&mut self) -> &mut IndexMap<Uuid, *mut Group> {
        unsafe { &mut (*self.0).groups }
    }

    pub fn groups(&self) -> impl Iterator<Item = &Group> {
        unsafe { (*self.0).groups.values().map(|group| &**group) }
    }

    pub fn groups_mut(&mut self) -> impl Iterator<Item = &mut Group> {
        unsafe { (*self.0).groups.values_mut().map(|group| &mut **group) }
    }

    pub fn move_entity_to_root(&mut self, entity_id: &Uuid) -> Result<(), Error> {
        unsafe {
            let Some(entity) = (*self.0).entities.get(entity_id) else {
                return Err(Error::NoSuchEntity);
            };
            let entity = &mut **entity;

            if entity.group == (*self.0).root_group {
                Ok(())
            } else {
                let group = &mut *entity.group;
                group.entities.swap_remove(entity_id);
                (*(*self.0).root_group).entities.insert(*entity_id, entity);
                entity.group = (*self.0).root_group;
                self.mark_dirty();
                Ok(())
            }
        }
    }

    pub fn move_entity_to_group(&mut self, entity_id: &Uuid, group_id: &Uuid) -> Result<(), Error> {
        unsafe {
            let Some(entity) = (*self.0).entities.get(entity_id) else {
                return Err(Error::NoSuchEntity);
            };
            let Some(to_group) = (*self.0).groups.get(group_id) else {
                return Err(Error::NoSuchGroup);
            };
            let entity = &mut **entity;
            let from_group = &mut *entity.group;
            let to_group = &mut **to_group;

            from_group.entities.swap_remove(entity_id);
            to_group.entities.insert(*entity_id, entity);
            entity.group = to_group;
            self.mark_dirty();
        }

        Ok(())
    }

    pub fn move_group_to_root(&mut self, group_id: &Uuid) -> Result<(), Error> {
        unsafe {
            let Some(group) = (*self.0).groups.get(group_id) else {
                return Err(Error::NoSuchGroup);
            };
            let group = &mut **group;
            let parent = group.parent.unwrap();

            if parent == (*self.0).root_group {
                return Ok(());
            }

            (*parent).subgroups.swap_remove(&group.id);
            (*(*self.0).root_group).subgroups.insert(group.id, group);
            group.parent = Some((*self.0).root_group);
            self.mark_dirty();

            Ok(())
        }
    }

    pub fn move_group_to_group(
        &mut self,
        group_id: &Uuid,
        to_group_id: &Uuid,
    ) -> Result<(), Error> {
        unsafe {
            let Some(group) = (*self.0).groups.get(group_id) else {
                return Err(Error::NoSuchGroup);
            };
            let Some(to_group) = (*self.0).groups.get(to_group_id) else {
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
            self.mark_dirty();
        }
        Ok(())
    }

    pub fn mark_dirty(&mut self) {
        unsafe {
            (*self.0).mark_dirty();
        }
    }

    pub fn is_dirty(&self) -> bool {
        unsafe { (*self.0).dirty }
    }

    pub fn refresh(&mut self) {
        unsafe {
            (*(*self.0).root_group).refresh();
            (*self.0).dirty = false;
        }
    }
}
