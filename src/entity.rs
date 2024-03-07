use std::{any::Any, cell::RefCell, collections::VecDeque, rc::Rc};

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
    share::Share,
};

pub trait Entity {
    fn id(&self) -> &Uuid;

    fn compose_model_matrix(&self) -> Readonly<'_, Mat4>;

    fn compose_normal_matrix(&self) -> Readonly<'_, Mat4>;

    fn bounding_volume(&self) -> Option<Readonly<'_, CullingBoundingVolume>>;

    fn geometry(&self) -> Option<&dyn Geometry>;

    fn geometry_mut(&mut self) -> Option<&mut dyn Geometry>;

    fn material(&self) -> Option<&dyn StandardMaterial>;

    fn material_mut(&mut self) -> Option<&mut dyn StandardMaterial>;

    fn attribute_value(&self, name: &str) -> Option<Readonly<'_, AttributeValue>>;

    fn uniform_value(&self, name: &str) -> Option<Readonly<'_, UniformValue>>;

    fn uniform_block_value(&self, name: &str) -> Option<Readonly<'_, UniformBlockValue>>;

    fn tick(&mut self, tick: &Tick) -> bool;

    fn should_update(&self) -> bool;

    fn mark_update(&mut self);

    fn update(&mut self, group: &dyn Group) -> bool;

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait Group {
    fn id(&self) -> &Uuid;

    fn compose_model_matrix(&self) -> Readonly<'_, Mat4>;

    fn bounding_volume(&self) -> Option<Readonly<'_, CullingBoundingVolume>>;

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

    fn tick(&mut self, tick: &Tick) -> bool;

    fn should_update(&self) -> bool;

    fn mark_update(&mut self);

    fn update(&mut self, parent: Option<&dyn Group>) -> bool;

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

    model_matrix: Mat4,
    parent_compose_model_matrix: Mat4,
    compose_model_matrix: Mat4,
    compose_normal_matrix: Mat4,

    geometry: Option<Box<dyn Geometry>>,
    material: Option<Box<dyn StandardMaterial>>,

    enable_bounding: bool,
    bounding_volume: Option<CullingBoundingVolume>,

    should_update: bool,
}

impl SimpleEntity {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            model_matrix: Mat4::<f64>::new_identity(),
            parent_compose_model_matrix: Mat4::<f64>::new_identity(),
            compose_model_matrix: Mat4::<f64>::new_identity(),
            compose_normal_matrix: Mat4::<f64>::new_identity(),
            geometry: None,
            material: None,
            enable_bounding: true,
            bounding_volume: None,
            should_update: true,
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
        self.mark_update();
    }

    pub fn disable_bounding(&mut self) {
        self.enable_bounding = false;
        self.bounding_volume = None;
        self.mark_update();
    }

    fn update_matrices(&mut self, group: &dyn Group) {
        self.parent_compose_model_matrix = *group.compose_model_matrix();
        self.compose_model_matrix = self.parent_compose_model_matrix * self.model_matrix;
        self.compose_normal_matrix = self.compose_model_matrix.clone();
        self.compose_normal_matrix
            .invert_in_place()
            .expect("invert a matrix with zero determinant is not allowed");
        self.compose_normal_matrix.transpose_in_place();
    }

    fn update_bounding_volume(&mut self) {
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

    fn compose_model_matrix(&self) -> Readonly<'_, Mat4> {
        Readonly::Borrowed(&self.compose_model_matrix)
    }

    fn compose_normal_matrix(&self) -> Readonly<'_, Mat4> {
        Readonly::Borrowed(&self.compose_normal_matrix)
    }

    fn bounding_volume(&self) -> Option<Readonly<'_, CullingBoundingVolume>> {
        self.bounding_volume
            .as_ref()
            .map(|volume| Readonly::Borrowed(volume))
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
            mutated = geometry.tick(tick) || mutated;
        }
        if let Some(material) = self.material.as_mut() {
            mutated = material.tick(tick) || mutated;
        }

        if mutated {
            self.mark_update();
        }

        mutated
    }

    fn should_update(&self) -> bool {
        self.should_update
    }

    fn mark_update(&mut self) {
        self.should_update = true;
    }

    fn update(&mut self, group: &dyn Group) -> bool {
        let should_update = self.should_update
            || group.compose_model_matrix().as_ref() != &self.parent_compose_model_matrix;

        if should_update {
            self.update_matrices(group);
            self.update_bounding_volume();

            self.should_update = false;
            true
        } else {
            false
        }
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
    parent_compose_model_matrix: Mat4,
    compose_model_matrix: Mat4,

    entities: IndexMap<Uuid, Share<dyn Entity>>,
    sub_groups: IndexMap<Uuid, Share<dyn Group>>,

    enable_bounding: bool,
    bounding_volume: Option<CullingBoundingVolume>,

    should_update: bool,
}

impl SimpleGroup {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            model_matrix: Mat4::<f64>::new_identity(),
            parent_compose_model_matrix: Mat4::<f64>::new_identity(),
            compose_model_matrix: Mat4::<f64>::new_identity(),
            entities: IndexMap::new(),
            sub_groups: IndexMap::new(),
            enable_bounding: true,
            bounding_volume: None,
            should_update: true,
        }
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
        self.mark_update();
        let id = entity.borrow().id().clone();
        self.entities.insert(id, entity);
    }

    pub fn remove_entity(&mut self, id: &Uuid) -> Option<Share<dyn Entity>> {
        self.mark_update();
        self.entities.swap_remove(id)
    }

    pub fn add_sub_group<G>(&mut self, group: G)
    where
        G: Group + 'static,
    {
        self.add_sub_group_shared(Rc::new(RefCell::new(group)))
    }

    pub fn add_sub_group_shared(&mut self, group: Share<dyn Group>) {
        self.mark_update();
        let id = group.borrow().id().clone();
        self.sub_groups.insert(id, group);
    }

    pub fn remove_sub_group(&mut self, id: &Uuid) -> Option<Share<dyn Group>> {
        self.mark_update();
        self.sub_groups.swap_remove(id)
    }

    fn update_matrices(&mut self, parent: Option<&dyn Group>) {
        self.parent_compose_model_matrix = parent
            .map(|parent| *parent.compose_model_matrix())
            .unwrap_or(Mat4::<f64>::new_identity());
        self.compose_model_matrix = self.parent_compose_model_matrix * self.model_matrix;
    }

    fn update_children_and_bounding_volume(&mut self) {
        let mut bounding_volumes = Vec::new();

        for entity in self.entities() {
            entity.borrow_mut().update(self);

            if let Some(bounding) = entity.borrow().bounding_volume() {
                bounding_volumes.push(bounding.bounding_volume());
            }
        }

        for sub_group in self.sub_groups() {
            sub_group.borrow_mut().update(Some(self));

            if let Some(bounding) = sub_group.borrow().bounding_volume() {
                bounding_volumes.push(bounding.bounding_volume());
            }
        }

        self.bounding_volume = merge_bounding_volumes(bounding_volumes)
            .map(|bounding| CullingBoundingVolume::new(bounding));
    }

    fn update_children(&mut self) -> bool {
        let mut mutated = false;

        for entity in self.entities() {
            mutated = entity.borrow_mut().update(self) || mutated;
        }

        for sub_group in self.sub_groups() {
            mutated = sub_group.borrow_mut().update(Some(self)) || mutated;
        }

        mutated
    }
}

impl Group for SimpleGroup {
    fn id(&self) -> &Uuid {
        &self.id
    }

    fn compose_model_matrix(&self) -> Readonly<'_, Mat4> {
        Readonly::Borrowed(&self.compose_model_matrix)
    }

    fn bounding_volume(&self) -> Option<Readonly<'_, CullingBoundingVolume>> {
        self.bounding_volume
            .as_ref()
            .map(|volume| Readonly::Borrowed(volume))
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

    fn tick(&mut self, tick: &Tick) -> bool {
        let mut mutated = false;

        for (_, entity) in &mut self.entities {
            mutated = entity.borrow_mut().tick(tick) || mutated;
        }

        for (_, sub_group) in &mut self.sub_groups {
            mutated = sub_group.borrow_mut().tick(tick) || mutated;
        }

        mutated
    }

    fn should_update(&self) -> bool {
        if self.should_update {
            return true;
        }

        if self
            .entities
            .values()
            .any(|entity| entity.borrow().should_update())
        {
            return true;
        }

        if self
            .sub_groups
            .values()
            .any(|sub_group| sub_group.borrow().should_update())
        {
            return true;
        }

        false
    }

    fn mark_update(&mut self) {
        self.should_update = true;
    }

    fn update(&mut self, parent: Option<&dyn Group>) -> bool {
        let should_update = self.should_update
            || parent
                .map(|parent| parent.compose_model_matrix())
                .map(|parent_compose_model_matrix| {
                    parent_compose_model_matrix.as_ref() != &self.parent_compose_model_matrix
                })
                .unwrap_or(false);

        if should_update {
            self.update_matrices(parent);

            if self.enable_bounding {
                self.update_children_and_bounding_volume();
            } else {
                self.update_children();
                self.bounding_volume = None;
            }

            self.should_update = false;
            true
        } else {
            self.update_children()
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
