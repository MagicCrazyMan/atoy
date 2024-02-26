use std::{
    any::Any,
    cell::RefCell,
    collections::VecDeque,
    ops::{Deref, DerefMut},
    rc::Rc,
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
};

pub trait Entity {
    fn id(&self) -> &Uuid;

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

    fn tick(&mut self, tick: &Tick) -> bool;

    fn sync(&mut self, group: &dyn Group);

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait Group {
    fn id(&self) -> &Uuid;

    fn compose_model_matrix(&self) -> &Mat4;

    fn bounding_volume(&self) -> Option<&CullingBoundingVolume>;

    fn entity(&self, id: &Uuid) -> Option<Rc<RefCell<dyn Entity>>>;

    fn entities(&self) -> Box<dyn Iterator<Item = Rc<RefCell<dyn Entity>>> + '_>;

    fn entities_hierarchy(&self) -> HierarchyEntitiesIter
    where
        Self: Sized,
    {
        HierarchyEntitiesIter::new(self)
    }

    fn sub_group(&self, id: &Uuid) -> Option<Rc<RefCell<dyn Group>>>;

    fn sub_groups(&self) -> Box<dyn Iterator<Item = Rc<RefCell<dyn Group>>> + '_>;

    fn sub_groups_hierarchy(&self) -> HierarchyGroupsIter
    where
        Self: Sized,
    {
        HierarchyGroupsIter::new(self)
    }

    fn tick(&mut self, tick: &Tick) -> bool;

    fn sync(&mut self, parent: Option<&dyn Group>);

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct HierarchyGroupsIter {
    groups: VecDeque<Rc<RefCell<dyn Group>>>,
}

impl HierarchyGroupsIter {
    pub fn new(group: &dyn Group) -> Self {
        Self {
            groups: group.sub_groups().collect(),
        }
    }
}

impl Iterator for HierarchyGroupsIter {
    type Item = Rc<RefCell<dyn Group>>;

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
    entities: VecDeque<Rc<RefCell<dyn Entity>>>,
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
    type Item = Rc<RefCell<dyn Entity>>;

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

    pub fn set_resync(&mut self) {
        self.should_sync = true;
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

    fn tick(&mut self, tick: &Tick) -> bool {
        let mut mutated = false;

        if let Some(geometry) = self.geometry.as_mut() {
            mutated = mutated | geometry.tick(tick);
        }
        if let Some(material) = self.material.as_mut() {
            mutated = mutated | material.tick(tick);
        }

        mutated
    }

    fn sync(&mut self, group: &dyn Group) {
        if !self.should_sync {
            return;
        }

        self.sync_matrices(group);
        self.sync_bounding_volume();

        self.should_sync = false;
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

    model_matrix: Mat4,
    compose_model_matrix: Mat4,

    entities: IndexMap<Uuid, Rc<RefCell<dyn Entity>>>,
    sub_groups: IndexMap<Uuid, Rc<RefCell<dyn Group>>>,

    enable_bounding: bool,
    bounding_volume: Option<CullingBoundingVolume>,

    should_sync: bool,
}

impl SimpleGroup {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            model_matrix: Mat4::<f64>::new_identity(),
            compose_model_matrix: Mat4::<f64>::new_identity(),
            entities: IndexMap::new(),
            sub_groups: IndexMap::new(),
            enable_bounding: true,
            bounding_volume: None,
            should_sync: true,
        }
    }

    pub fn model_matrix(&self) -> &Mat4 {
        &self.model_matrix
    }

    pub fn add_entity<E>(&mut self, entity: E)
    where
        E: Entity + 'static,
    {
        self.add_entity_boxed(Rc::new(RefCell::new(entity)))
    }

    pub fn add_entity_boxed(&mut self, entity: Rc<RefCell<dyn Entity>>) {
        let id = entity.borrow().id().clone();
        self.entities.insert(id, entity);
        self.set_resync();
    }

    pub fn remove_entity(&mut self, id: &Uuid) -> Option<Rc<RefCell<dyn Entity>>> {
        match self.entities.swap_remove(id) {
            Some(entity) => {
                self.set_resync();
                Some(entity)
            }
            None => None,
        }
    }

    pub fn add_sub_group<G>(&mut self, group: G)
    where
        G: Group + 'static,
    {
        self.add_sub_group_boxed(Rc::new(RefCell::new(group)))
    }

    pub fn add_sub_group_boxed(&mut self, group: Rc<RefCell<dyn Group>>) {
        let id = group.borrow().id().clone();
        self.sub_groups.insert(id, group);
        self.set_resync();
    }

    pub fn remove_sub_group(&mut self, id: &Uuid) -> Option<Rc<RefCell<dyn Group>>> {
        match self.sub_groups.swap_remove(id) {
            Some(sub_group) => {
                self.set_resync();
                Some(sub_group)
            }
            None => None,
        }
    }

    pub fn should_sync(&self) -> bool {
        self.should_sync
    }

    pub fn set_resync(&mut self) {
        self.should_sync = true;
    }

    fn sync_matrices(&mut self, parent: Option<&dyn Group>) {
        self.compose_model_matrix = match parent {
            Some(parent) => *parent.compose_model_matrix() * self.model_matrix,
            None => self.model_matrix.clone(),
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

    fn compose_model_matrix(&self) -> &Mat4 {
        &self.compose_model_matrix
    }

    fn bounding_volume(&self) -> Option<&CullingBoundingVolume> {
        self.bounding_volume.as_ref()
    }

    fn entity(&self, id: &Uuid) -> Option<Rc<RefCell<dyn Entity>>> {
        self.entities.get(id).map(|entity| Rc::clone(entity))
    }

    fn entities(&self) -> Box<dyn Iterator<Item = Rc<RefCell<dyn Entity>>> + '_> {
        Box::new(self.entities.values().map(|entity| Rc::clone(entity)))
    }

    fn sub_group(&self, id: &Uuid) -> Option<Rc<RefCell<dyn Group>>> {
        self.sub_groups
            .get(id)
            .map(|sub_group| Rc::clone(sub_group))
    }

    fn sub_groups(&self) -> Box<dyn Iterator<Item = Rc<RefCell<dyn Group>>> + '_> {
        Box::new(
            self.sub_groups
                .values()
                .map(|sub_group| Rc::clone(sub_group)),
        )
    }

    fn tick(&mut self, tick: &Tick) -> bool {
        let mut mutated = false;

        for (_, entity) in &mut self.entities {
            mutated = mutated | entity.borrow_mut().tick(tick);
        }

        for (_, sub_group) in &mut self.sub_groups {
            mutated = mutated | sub_group.borrow_mut().tick(tick);
        }

        mutated
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

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct SceneGroup(SimpleGroup);

impl Deref for SceneGroup {
    type Target = SimpleGroup;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SceneGroup {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl SceneGroup {
    pub fn new() -> Self {
        Self(SimpleGroup::new())
    }

    /// Syncs scene entities.
    pub fn sync(&mut self) {
        self.0.sync(None)
    }
}
