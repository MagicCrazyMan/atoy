use std::{
    any::Any,
    cell::RefCell,
    collections::VecDeque,
    rc::{Rc, Weak},
};

use gl_matrix4rust::mat4::Mat4;
use indexmap::IndexMap;
use uuid::Uuid;

use crate::{
    bounding::{merge_bounding_volumes, CullingBoundingVolume},
    clock::Tick,
    geometry::Geometry,
    material::webgl::StandardMaterial,
    readonly::Readonly,
    renderer::webgl::{
        attribute::AttributeValue,
        uniform::{UniformBlockValue, UniformValue},
    },
    share::{Share, WeakShare},
};

pub trait Entity {
    fn id(&self) -> &Uuid;

    fn group(&self) -> Option<Share<dyn Group>>;

    fn mount(&mut self, group: &Share<dyn Group>);

    fn umount(&mut self);

    fn compose_model_matrix(&self) -> &Mat4;

    fn compose_normal_matrix(&self) -> &Mat4;

    fn bounding_volume(&self) -> Option<&CullingBoundingVolume>;

    fn geometry(&self) -> Option<&dyn Geometry>;

    fn geometry_mut(&mut self) -> Option<&mut dyn Geometry>;

    fn material(&self) -> Option<&dyn StandardMaterial>;

    fn material_mut(&mut self) -> Option<&mut dyn StandardMaterial>;

    fn attribute_value(&self, name: &str) -> Option<Readonly<'_, AttributeValue>>;

    fn uniform_value(&self, name: &str) -> Option<Readonly<'_, UniformValue>>;

    fn uniform_block_value(&self, name: &str) -> Option<Readonly<'_, UniformBlockValue>>;

    fn tick(&mut self, tick: &Tick);

    fn sync(&mut self, group: &dyn Group);

    fn set_resync(&mut self);

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait Group {
    fn id(&self) -> &Uuid;

    fn parent(&self) -> Option<Share<dyn Group>>;

    fn mount(&mut self, group: &Share<dyn Group>);

    fn umount(&mut self);

    fn compose_model_matrix(&self) -> &Mat4;

    fn bounding_volume(&self) -> Option<&CullingBoundingVolume>;

    fn entity(&self, id: &Uuid) -> Option<Share<dyn Entity>>;

    fn entities(&self) -> Box<dyn Iterator<Item = Share<dyn Entity>> + '_>;

    fn entities_hierarchy(&self) -> HierarchyEntitiesIter
    where
        Self: Sized,
    {
        HierarchyEntitiesIter::new(self)
    }

    fn sub_group(&self, id: &Uuid) -> Option<Share<dyn Group>>;

    fn sub_groups(&self) -> Box<dyn Iterator<Item = Share<dyn Group>> + '_>;

    fn sub_groups_hierarchy(&self) -> HierarchyGroupsIter
    where
        Self: Sized,
    {
        HierarchyGroupsIter::new(self)
    }

    fn tick(&mut self, tick: &Tick);

    fn sync(&mut self, parent: Option<&dyn Group>);

    fn set_resync(&mut self, resync_entities: bool, resync_sub_groups: bool);

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct HierarchyGroupsIter {
    groups: VecDeque<Share<dyn Group>>,
}

impl HierarchyGroupsIter {
    pub fn new(group: &dyn Group) -> Self {
        Self {
            groups: group.sub_groups().collect(),
        }
    }
}

impl Iterator for HierarchyGroupsIter {
    type Item = Share<dyn Group>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.groups.pop_front() {
            Some(group) => {
                self.groups.extend(group.borrow().sub_groups());
                Some(group)
            }
            None => None,
        }
    }
}

pub struct HierarchyEntitiesIter {
    groups: HierarchyGroupsIter,
    entities: VecDeque<Share<dyn Entity>>,
}

impl HierarchyEntitiesIter {
    pub fn new(group: &dyn Group) -> Self {
        Self {
            groups: HierarchyGroupsIter::new(group),
            entities: group.entities().collect(),
        }
    }
}

impl Iterator for HierarchyEntitiesIter {
    type Item = Share<dyn Entity>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.entities.pop_front() {
            Some(entity) => Some(entity),
            None => match self.groups.next() {
                Some(group) => {
                    self.entities.extend(group.borrow().entities());
                    match self.entities.pop_front() {
                        Some(entity) => Some(entity),
                        None => None,
                    }
                }
                None => None,
            },
        }
    }
}

pub struct SimpleEntity {
    id: Uuid,
    group: Option<WeakShare<dyn Group>>,

    model_matrix: Mat4,
    compose_model_matrix: Mat4,
    compose_normal_matrix: Mat4,

    geometry: Option<Box<dyn Geometry>>,
    material: Option<Box<dyn StandardMaterial>>,

    enable_bounding: bool,
    bounding_volume: Option<CullingBoundingVolume>,

    should_sync: bool,
}

impl SimpleEntity {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            group: None,
            model_matrix: Mat4::<f64>::new_identity(),
            compose_model_matrix: Mat4::<f64>::new_identity(),
            compose_normal_matrix: Mat4::<f64>::new_identity(),
            geometry: None,
            material: None,
            enable_bounding: true,
            bounding_volume: None,
            should_sync: true,
        }
    }

    pub fn model_matrix(&self) -> &Mat4 {
        &self.model_matrix
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

    pub fn bounding_enabled(&self) -> bool {
        self.enable_bounding
    }

    pub fn enable_bounding(&mut self) {
        self.enable_bounding = true;
        self.bounding_volume = None;
        self.set_resync();
    }

    pub fn disable_bounding(&mut self) {
        self.enable_bounding = false;
        self.bounding_volume = None;
        self.set_resync();
    }

    fn sync_matrices(&mut self, group: &dyn Group) {
        self.compose_model_matrix = *group.compose_model_matrix() * *self.model_matrix();
        self.compose_normal_matrix = self.compose_model_matrix.clone();
        self.compose_normal_matrix
            .invert_in_place()
            .expect("invert a matrix with zero determinant is not allowed");
        self.compose_normal_matrix.transpose_in_place();
    }

    fn sync_bounding_volume(&mut self) {
        if !self.enable_bounding {
            self.bounding_volume = None;
            return;
        }

        let compose_model_matrix = self.compose_model_matrix;
        self.bounding_volume =
            self.geometry()
                .and_then(|geom| geom.bounding_volume())
                .map(|bounding| {
                    CullingBoundingVolume::new(bounding.as_ref().transform(compose_model_matrix))
                });
    }
}

impl Entity for SimpleEntity {
    fn id(&self) -> &Uuid {
        &self.id
    }

    fn group(&self) -> Option<Share<dyn Group>> {
        self.group.as_ref().and_then(|group| group.upgrade())
    }

    fn mount(&mut self, group: &Share<dyn Group>) {
        self.group = Some(Rc::downgrade(group));
    }

    fn umount(&mut self) {
        self.group = None;
    }

    fn compose_model_matrix(&self) -> &Mat4 {
        &self.compose_model_matrix
    }

    fn compose_normal_matrix(&self) -> &Mat4 {
        &self.compose_normal_matrix
    }

    fn bounding_volume(&self) -> Option<&CullingBoundingVolume> {
        self.bounding_volume.as_ref()
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

    fn tick(&mut self, tick: &Tick) {
        let mut mutated = false;

        if let Some(geometry) = self.geometry.as_mut() {
            mutated = mutated | geometry.tick(tick);
        }
        if let Some(material) = self.material.as_mut() {
            mutated = mutated | material.tick(tick);
        }

        if mutated {
            self.set_resync();
        }
    }

    fn sync(&mut self, group: &dyn Group) {
        if !self.should_sync {
            return;
        }

        self.sync_matrices(group);
        self.sync_bounding_volume();

        self.should_sync = false;
    }

    fn set_resync(&mut self) {
        self.should_sync = true;

        let Some(group) = self.group.as_ref().and_then(|group| group.upgrade()) else {
            return;
        };
        let Ok(mut group) = group.try_borrow_mut() else {
            return;
        };
        group.set_resync(false, false);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct SimpleGroup {
    id: Uuid,
    me: WeakShare<Self>,
    parent: Option<WeakShare<dyn Group>>,

    model_matrix: Mat4,
    compose_model_matrix: Mat4,

    entities: IndexMap<Uuid, Share<dyn Entity>>,
    sub_groups: IndexMap<Uuid, Share<dyn Group>>,

    enable_bounding: bool,
    bounding_volume: Option<CullingBoundingVolume>,

    should_sync: bool,
}

impl SimpleGroup {
    pub fn new() -> Share<Self> {
        Rc::new_cyclic(|me| {
            RefCell::new(Self {
                id: Uuid::new_v4(),
                me: Weak::clone(me),
                parent: None,
                model_matrix: Mat4::<f64>::new_identity(),
                compose_model_matrix: Mat4::<f64>::new_identity(),
                entities: IndexMap::new(),
                sub_groups: IndexMap::new(),
                enable_bounding: true,
                bounding_volume: None,
                should_sync: true,
            })
        })
    }

    pub fn model_matrix(&self) -> &Mat4 {
        &self.model_matrix
    }

    pub fn add_entity<E>(&mut self, entity: E)
    where
        E: Entity + 'static,
    {
        self.add_entity_shared(Rc::new(RefCell::new(entity)))
    }

    pub fn add_entity_shared(&mut self, entity: Share<dyn Entity>) {
        let id = entity.borrow().id().clone();
        let me: Rc<RefCell<dyn Group>> = self.me.upgrade().unwrap();
        entity.borrow_mut().mount(&me);
        entity.borrow_mut().set_resync();
        self.entities.insert(id, entity);
        self.set_resync(false, false);
    }

    pub fn remove_entity(&mut self, id: &Uuid) -> Option<Share<dyn Entity>> {
        match self.entities.swap_remove(id) {
            Some(entity) => {
                entity.borrow_mut().umount();
                entity.borrow_mut().set_resync();
                self.set_resync(false, false);
                Some(entity)
            }
            None => None,
        }
    }

    pub fn add_sub_group<G>(&mut self, group: G)
    where
        G: Group + 'static,
    {
        self.add_sub_group_shared(Rc::new(RefCell::new(group)))
    }

    pub fn add_sub_group_shared(&mut self, group: Share<dyn Group>) {
        let id = group.borrow().id().clone();
        let me: Rc<RefCell<dyn Group>> = self.me.upgrade().unwrap();
        group.borrow_mut().mount(&me);
        group.borrow_mut().set_resync(true, true);
        self.sub_groups.insert(id, group);
        self.set_resync(false, false);
    }

    pub fn remove_sub_group(&mut self, id: &Uuid) -> Option<Share<dyn Group>> {
        match self.sub_groups.swap_remove(id) {
            Some(sub_group) => {
                sub_group.borrow_mut().umount();
                sub_group.borrow_mut().set_resync(true, true);
                self.set_resync(false, false);
                Some(sub_group)
            }
            None => None,
        }
    }

    pub fn should_sync(&self) -> bool {
        self.should_sync
    }

    fn sync_matrices(&mut self, parent: Option<&dyn Group>) {
        let parent_model_matrix = parent.map(|parent| *parent.compose_model_matrix());
        self.compose_model_matrix = match parent_model_matrix {
            Some(parent) => parent * *self.model_matrix(),
            None => self.model_matrix().clone(),
        };
    }

    fn sync_children_and_bounding_volume(&mut self) {
        let mut bounding_volumes = Vec::new();

        for entity in self.entities() {
            entity.borrow_mut().sync(self);

            if let Some(bounding) = entity.borrow().bounding_volume() {
                bounding_volumes.push(bounding.bounding_volume());
            }
        }

        for sub_group in self.sub_groups() {
            sub_group.borrow_mut().sync(Some(self));

            if let Some(bounding) = sub_group.borrow().bounding_volume() {
                bounding_volumes.push(bounding.bounding_volume());
            }
        }

        self.bounding_volume = merge_bounding_volumes(bounding_volumes)
            .map(|bounding| CullingBoundingVolume::new(bounding));
    }

    fn sync_children(&mut self) {
        for entity in self.entities() {
            entity.borrow_mut().sync(self);
        }

        for sub_group in self.sub_groups() {
            sub_group.borrow_mut().sync(Some(self));
        }
    }
}

impl Group for SimpleGroup {
    fn id(&self) -> &Uuid {
        &self.id
    }

    fn parent(&self) -> Option<Share<dyn Group>> {
        self.parent.as_ref().and_then(|parent| parent.upgrade())
    }

    fn mount(&mut self, group: &Share<dyn Group>) {
        self.parent = Some(Rc::downgrade(group));
    }

    fn umount(&mut self) {
        self.parent = None;
    }

    fn compose_model_matrix(&self) -> &Mat4 {
        &self.compose_model_matrix
    }

    fn bounding_volume(&self) -> Option<&CullingBoundingVolume> {
        self.bounding_volume.as_ref()
    }

    fn entity(&self, id: &Uuid) -> Option<Share<dyn Entity>> {
        self.entities.get(id).map(|entity| Rc::clone(entity))
    }

    fn entities(&self) -> Box<dyn Iterator<Item = Share<dyn Entity>> + '_> {
        Box::new(self.entities.values().map(|entity| Rc::clone(entity)))
    }

    fn sub_group(&self, id: &Uuid) -> Option<Share<dyn Group>> {
        self.sub_groups
            .get(id)
            .map(|sub_group| Rc::clone(sub_group))
    }

    fn sub_groups(&self) -> Box<dyn Iterator<Item = Share<dyn Group>> + '_> {
        Box::new(
            self.sub_groups
                .values()
                .map(|sub_group| Rc::clone(sub_group)),
        )
    }

    fn tick(&mut self, tick: &Tick) {
        for (_, entity) in &mut self.entities {
            entity.borrow_mut().tick(tick);
        }

        for (_, sub_group) in &mut self.sub_groups {
            sub_group.borrow_mut().tick(tick);
        }
    }

    fn sync(&mut self, parent: Option<&dyn Group>) {
        if self.should_sync {
            self.sync_matrices(parent);

            if self.enable_bounding {
                self.sync_children_and_bounding_volume();
            } else {
                self.sync_children();
                self.bounding_volume = None;
            }
            self.should_sync = false;
        } else {
            self.sync_children();
        }
    }

    fn set_resync(&mut self, resync_entities: bool, resync_sub_groups: bool) {
        self.should_sync = true;

        if resync_sub_groups {
            for sub_group in self.sub_groups_hierarchy() {
                let Ok(mut sub_group) = sub_group.try_borrow_mut() else {
                    continue;
                };
                sub_group.set_resync(resync_entities, resync_sub_groups);
            }
        }

        if resync_entities {
            for entity in self.entities_hierarchy() {
                let Ok(mut entity) = entity.try_borrow_mut() else {
                    continue;
                };
                entity.set_resync();
            }
        }

        let Some(parent) = self.parent.as_ref().and_then(|parent| parent.upgrade()) else {
            return;
        };
        let Ok(mut parent) = parent.try_borrow_mut() else {
            return;
        };
        parent.set_resync(false, false);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
