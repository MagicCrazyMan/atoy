use std::{any::Any, cell::RefCell, collections::VecDeque, rc::Rc};

use gl_matrix4rust::mat4::Mat4;
use indexmap::IndexMap;
use uuid::Uuid;
use web_sys::WebGlVertexArrayObject;

use crate::{
    bounding::{merge_bounding_volumes, CullingBoundingVolume},
    clock::Tick,
    geometry::{Geometry, GeometryMessage},
    material::webgl::{MaterialMessage, StandardMaterial},
    message::{channel, Aborter, Executor, Receiver, Sender},
    renderer::webgl::{
        attribute::AttributeValue,
        uniform::{UniformBlockValue, UniformValue},
    },
    value::Readonly,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityMessage {
    Changed,
    GeometryChanged,
    MaterialChanged,
    ModelMatrixChanged,
    BoundingVolumeChanged,
}

pub trait Entity {
    fn id(&self) -> &Uuid;

    fn compose_model_matrix(&self) -> Readonly<'_, Mat4>;

    fn compose_normal_matrix(&self) -> Readonly<'_, Mat4>;

    fn bounding_volume(&self) -> Option<Readonly<'_, CullingBoundingVolume>>;

    fn geometry(&self) -> Option<&dyn Geometry>;

    fn geometry_mut(&mut self) -> Option<&mut dyn Geometry>;

    fn material(&self) -> Option<&dyn StandardMaterial>;

    fn material_mut(&mut self) -> Option<&mut dyn StandardMaterial>;

    fn attribute_value(&self, name: &str) -> Option<AttributeValue<'_>>;

    fn uniform_value(&self, name: &str) -> Option<UniformValue<'_>>;

    fn uniform_block_value(&self, name: &str) -> Option<UniformBlockValue<'_>>;

    fn tick(&mut self, tick: &Tick);

    fn changed(&self) -> Receiver<EntityMessage>;

    fn should_update(&self) -> bool;

    // fn mark_update(&mut self);

    fn update(&mut self, group: &dyn Group);

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn as_vertex_array_object_entity(&self) -> Option<&dyn VertexArrayObjectEntity>;

    fn as_vertex_array_object_entity_mut(&mut self) -> Option<&mut dyn VertexArrayObjectEntity>;
}

pub trait VertexArrayObjectEntity {
    fn vertex_array_object(&self) -> Option<WebGlVertexArrayObject>;

    fn store_vertex_array_object(&mut self, vao: WebGlVertexArrayObject);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GroupMessage {
    Changed,
    ModelMatrixChanged,
    BoundingVolumeChanged,
    EntityChanged,
    SubGroupChanged,
    AddEntity,
    RemoveEntity,
    AddSubGroup,
    RemoveSubGroup,
}

pub trait Group {
    fn id(&self) -> &Uuid;

    fn compose_model_matrix(&self) -> Readonly<'_, Mat4>;

    fn bounding_volume(&self) -> Option<Readonly<'_, CullingBoundingVolume>>;

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

    fn tick(&mut self, tick: &Tick);

    fn changed(&self) -> Receiver<GroupMessage>;

    fn should_update(&self) -> bool;

    // fn mark_update(&mut self);

    fn update(&mut self, parent: Option<&dyn Group>);

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
    parent_compose_model_matrix: Mat4,
    compose_model_matrix: Mat4,
    compose_normal_matrix: Mat4,

    geometry: Option<(Box<dyn Geometry>, Aborter<GeometryMessage>)>,
    material: Option<(Box<dyn StandardMaterial>, Aborter<MaterialMessage>)>,

    enable_bounding: bool,
    bounding_volume: Option<CullingBoundingVolume>,

    vao: Rc<RefCell<Option<WebGlVertexArrayObject>>>,

    channel: (Sender<EntityMessage>, Receiver<EntityMessage>),

    should_update: Rc<RefCell<bool>>,
    should_recalculate_matrices: Rc<RefCell<bool>>,
    should_recalculate_bounding: Rc<RefCell<bool>>,
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

            vao: Rc::new(RefCell::new(None)),

            channel: channel(),

            should_update: Rc::new(RefCell::new(true)),
            should_recalculate_matrices: Rc::new(RefCell::new(true)),
            should_recalculate_bounding: Rc::new(RefCell::new(true)),
        }
    }

    pub fn model_matrix(&self) -> &Mat4 {
        &self.model_matrix
    }

    pub fn set_model_matrix(&mut self, model_matrix: Mat4) {
        self.model_matrix = model_matrix;
        *self.should_update.borrow_mut() = true;
        *self.should_recalculate_matrices.borrow_mut() = true;
        *self.should_recalculate_bounding.borrow_mut() = true;
        self.channel.0.send(EntityMessage::ModelMatrixChanged);
        self.channel.0.send(EntityMessage::Changed);
    }

    pub fn set_geometry<G>(&mut self, geometry: Option<G>) -> Option<Box<dyn Geometry>>
    where
        G: Geometry + 'static,
    {
        let old_geometry = match self.geometry.take() {
            Some((geometry, aborter)) => {
                aborter.off();
                Some(geometry)
            }
            None => None,
        };

        self.geometry = geometry.map(|geometry| {
            struct GeometryChanged {
                sender: Sender<EntityMessage>,
                vao: Rc<RefCell<Option<WebGlVertexArrayObject>>>,
                should_update: Rc<RefCell<bool>>,
                should_recalculate_bounding: Rc<RefCell<bool>>,
            }

            impl Executor for GeometryChanged {
                type Message = GeometryMessage;

                fn execute(&mut self, msg: &Self::Message) {
                    *self.should_update.borrow_mut() = true;

                    if *msg == GeometryMessage::BoundingVolumeChanged {
                        *self.should_recalculate_bounding.borrow_mut() = true;
                        self.sender.send(EntityMessage::BoundingVolumeChanged);
                    } else if *msg == GeometryMessage::VertexArrayObjectChanged {
                        self.vao.borrow_mut().take();
                    }

                    self.sender.send(EntityMessage::GeometryChanged);
                    self.sender.send(EntityMessage::Changed);
                }
            }

            let aborter = geometry.changed().on(GeometryChanged {
                sender: self.channel.0.clone(),
                vao: Rc::clone(&self.vao),
                should_update: Rc::clone(&self.should_update),
                should_recalculate_bounding: Rc::clone(&self.should_recalculate_bounding),
            });
            let geometry = Box::new(geometry) as Box<dyn Geometry>;
            (geometry, aborter)
        });

        *self.should_update.borrow_mut() = true;
        self.channel.0.send(EntityMessage::GeometryChanged);
        self.channel.0.send(EntityMessage::Changed);

        old_geometry
    }

    pub fn set_material<M>(&mut self, material: Option<M>) -> Option<Box<dyn StandardMaterial>>
    where
        M: StandardMaterial + 'static,
    {
        let old_material = match self.material.take() {
            Some((material, aborter)) => {
                aborter.off();
                Some(material)
            }
            None => None,
        };

        self.material = material.map(|material| {
            struct MaterialChanged {
                sender: Sender<EntityMessage>,
                vao: Rc<RefCell<Option<WebGlVertexArrayObject>>>,
                should_update: Rc<RefCell<bool>>,
            }

            impl Executor for MaterialChanged {
                type Message = MaterialMessage;

                fn execute(&mut self, msg: &Self::Message) {
                    *self.should_update.borrow_mut() = true;

                    if *msg == MaterialMessage::VertexArrayObjectChanged {
                        self.vao.borrow_mut().take();
                    };
                    
                    self.sender.send(EntityMessage::MaterialChanged);
                    self.sender.send(EntityMessage::Changed);
                }
            }

            let aborter = material.changed().on(MaterialChanged {
                sender: self.channel.0.clone(),
                vao: Rc::clone(&self.vao),
                should_update: Rc::clone(&self.should_update),
            });
            let material = Box::new(material) as Box<dyn StandardMaterial>;
            (material, aborter)
        });

        *self.should_update.borrow_mut() = true;
        self.channel.0.send(EntityMessage::MaterialChanged);
        self.channel.0.send(EntityMessage::Changed);

        old_material
    }

    pub fn bounding_enabled(&self) -> bool {
        self.enable_bounding
    }

    pub fn enable_bounding(&mut self) {
        self.enable_bounding = true;
        self.bounding_volume = None;
        *self.should_update.borrow_mut() = true;
        *self.should_recalculate_bounding.borrow_mut() = true;
    }

    pub fn disable_bounding(&mut self) {
        self.enable_bounding = false;
        self.bounding_volume = None;
        *self.should_update.borrow_mut() = true;
        *self.should_recalculate_bounding.borrow_mut() = true;
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
        self.geometry
            .as_ref()
            .map(|(geometry, _)| geometry.as_ref())
    }

    fn geometry_mut(&mut self) -> Option<&mut dyn Geometry> {
        match self.geometry.as_mut() {
            Some((geometry, _)) => Some(geometry.as_mut()),
            None => None,
        }
    }

    fn material(&self) -> Option<&dyn StandardMaterial> {
        self.material
            .as_ref()
            .map(|(material, _)| material.as_ref())
    }

    fn material_mut(&mut self) -> Option<&mut dyn StandardMaterial> {
        match self.material.as_mut() {
            Some((material, _)) => Some(material.as_mut()),
            None => None,
        }
    }

    fn attribute_value(&self, _: &str) -> Option<AttributeValue<'_>> {
        None
    }

    fn uniform_value(&self, _: &str) -> Option<UniformValue<'_>> {
        None
    }

    fn uniform_block_value(&self, _: &str) -> Option<UniformBlockValue<'_>> {
        None
    }

    fn tick(&mut self, tick: &Tick) {
        if let Some((geometry, _)) = self.geometry.as_mut() {
            geometry.tick(tick);
        }
        if let Some((material, _)) = self.material.as_mut() {
            material.tick(tick);
        }
    }

    fn changed(&self) -> Receiver<EntityMessage> {
        self.channel.1.clone()
    }

    fn should_update(&self) -> bool {
        *self.should_update.borrow()
    }

    // fn mark_update(&mut self) {
    //     *self.should_update_matrices.borrow_mut() = true;
    // }

    fn update(&mut self, group: &dyn Group) {
        let should_update = *self.should_update.borrow();
        let should_recalculate_matrices = *self.should_recalculate_matrices.borrow();
        let should_recalculate_bounding = *self.should_recalculate_bounding.borrow();
        let enable_bounding = self.enable_bounding;

        if should_update {
            if should_recalculate_matrices {
                self.update_matrices(group);
                *self.should_recalculate_matrices.borrow_mut() = false;
            }

            match (enable_bounding, should_recalculate_bounding) {
                (true, true) => {
                    self.update_bounding_volume();
                    *self.should_recalculate_bounding.borrow_mut() = false;
                }
                (true, false) => {
                    // do nothing
                }
                (false, true) => {
                    self.bounding_volume = None;
                    *self.should_recalculate_bounding.borrow_mut() = false;
                }
                (false, false) => {
                    // do nothing
                }
            }

            *self.should_update.borrow_mut() = false;
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_vertex_array_object_entity(&self) -> Option<&dyn VertexArrayObjectEntity> {
        Some(self)
    }

    fn as_vertex_array_object_entity_mut(&mut self) -> Option<&mut dyn VertexArrayObjectEntity> {
        Some(self)
    }
}

impl VertexArrayObjectEntity for SimpleEntity {
    fn vertex_array_object(&self) -> Option<WebGlVertexArrayObject> {
        self.vao.borrow().clone()
    }

    fn store_vertex_array_object(&mut self, vao: WebGlVertexArrayObject) {
        self.vao.borrow_mut().replace(vao);
    }
}

pub struct SimpleGroup {
    id: Uuid,

    model_matrix: Mat4,
    parent_compose_model_matrix: Mat4,
    compose_model_matrix: Mat4,

    entities: IndexMap<Uuid, (Rc<RefCell<dyn Entity>>, Aborter<EntityMessage>)>,
    sub_groups: IndexMap<Uuid, (Rc<RefCell<dyn Group>>, Aborter<GroupMessage>)>,

    enable_bounding: bool,
    bounding_volume: Option<CullingBoundingVolume>,

    channel: (Sender<GroupMessage>, Receiver<GroupMessage>),

    should_update: Rc<RefCell<bool>>,
    should_recalculate_matrices: Rc<RefCell<bool>>,
    should_recalculate_bounding: Rc<RefCell<bool>>,
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
            channel: channel(),
            should_update: Rc::new(RefCell::new(true)),
            should_recalculate_matrices: Rc::new(RefCell::new(true)),
            should_recalculate_bounding: Rc::new(RefCell::new(true)),
        }
    }

    pub fn model_matrix(&self) -> &Mat4 {
        &self.model_matrix
    }

    pub fn set_model_matrix(&mut self, model_matrix: Mat4) {
        self.model_matrix = model_matrix;
        *self.should_update.borrow_mut() = true;
        *self.should_recalculate_matrices.borrow_mut() = true;
        *self.should_recalculate_bounding.borrow_mut() = true;
        self.channel.0.send(GroupMessage::ModelMatrixChanged);
        self.channel.0.send(GroupMessage::Changed);
    }

    pub fn add_entity<E>(&mut self, entity: E)
    where
        E: Entity + 'static,
    {
        self.add_entity_shared(Rc::new(RefCell::new(entity)))
    }

    pub fn add_entity_shared(&mut self, entity: Rc<RefCell<dyn Entity>>) {
        struct EntityChanged {
            sender: Sender<GroupMessage>,
            should_update: Rc<RefCell<bool>>,
            should_recalculate_bounding: Rc<RefCell<bool>>,
        }

        impl Executor for EntityChanged {
            type Message = EntityMessage;

            fn execute(&mut self, msg: &Self::Message) {
                *self.should_update.borrow_mut() = true;

                if *msg == EntityMessage::BoundingVolumeChanged {
                    *self.should_recalculate_bounding.borrow_mut() = true;
                    self.sender.send(GroupMessage::BoundingVolumeChanged);
                }

                self.sender.send(GroupMessage::EntityChanged);
                self.sender.send(GroupMessage::Changed);
            }
        }

        let entity_ref = entity.borrow();
        let aborter = entity_ref.changed().on(EntityChanged {
            sender: self.channel.0.clone(),
            should_update: Rc::clone(&self.should_update),
            should_recalculate_bounding: Rc::clone(&self.should_recalculate_bounding),
        });
        let id = *entity_ref.id();
        drop(entity_ref);

        self.entities.insert(id, (entity, aborter));

        *self.should_update.borrow_mut() = true;
        *self.should_recalculate_bounding.borrow_mut() = true;
        self.channel.0.send(GroupMessage::AddEntity);
        self.channel.0.send(GroupMessage::Changed);
    }

    pub fn remove_entity(&mut self, id: &Uuid) -> Option<Rc<RefCell<dyn Entity>>> {
        match self.entities.swap_remove(id) {
            Some((entity, aborter)) => {
                aborter.off();
                *self.should_update.borrow_mut() = true;
                *self.should_recalculate_bounding.borrow_mut() = true;
                self.channel.0.send(GroupMessage::RemoveEntity);
                self.channel.0.send(GroupMessage::Changed);
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

    pub fn add_sub_group_shared(&mut self, group: Rc<RefCell<dyn Group>>) {
        struct SubGroupChanged {
            sender: Sender<GroupMessage>,
            should_update: Rc<RefCell<bool>>,
            should_recalculate_bounding: Rc<RefCell<bool>>,
        }

        impl Executor for SubGroupChanged {
            type Message = GroupMessage;

            fn execute(&mut self, msg: &Self::Message) {
                *self.should_update.borrow_mut() = true;

                if *msg == GroupMessage::BoundingVolumeChanged {
                    *self.should_recalculate_bounding.borrow_mut() = true;
                    self.sender.send(GroupMessage::BoundingVolumeChanged);
                }

                self.sender.send(GroupMessage::SubGroupChanged);
                self.sender.send(GroupMessage::Changed);
            }
        }

        let group_ref = group.borrow_mut();
        let aborter = group_ref.changed().on(SubGroupChanged {
            sender: self.channel.0.clone(),
            should_update: Rc::clone(&self.should_update),
            should_recalculate_bounding: Rc::clone(&self.should_recalculate_bounding),
        });
        let id = *group_ref.id();
        drop(group_ref);

        self.sub_groups.insert(id, (group, aborter));

        *self.should_update.borrow_mut() = true;
        *self.should_recalculate_bounding.borrow_mut() = true;
        self.channel.0.send(GroupMessage::AddSubGroup);
        self.channel.0.send(GroupMessage::Changed);
    }

    pub fn remove_sub_group(&mut self, id: &Uuid) -> Option<Rc<RefCell<dyn Group>>> {
        match self.sub_groups.swap_remove(id) {
            Some((sub_group, aborter)) => {
                aborter.off();
                *self.should_update.borrow_mut() = true;
                *self.should_recalculate_bounding.borrow_mut() = true;
                self.channel.0.send(GroupMessage::RemoveSubGroup);
                self.channel.0.send(GroupMessage::Changed);
                Some(sub_group)
            }
            None => None,
        }
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

    fn update_children(&mut self) {
        for entity in self.entities() {
            entity.borrow_mut().update(self);
        }

        for sub_group in self.sub_groups() {
            sub_group.borrow_mut().update(Some(self));
        }
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

    fn entity(&self, id: &Uuid) -> Option<Rc<RefCell<dyn Entity>>> {
        self.entities.get(id).map(|(entity, _)| Rc::clone(entity))
    }

    fn entities(&self) -> Box<dyn Iterator<Item = Rc<RefCell<dyn Entity>>> + '_> {
        Box::new(self.entities.values().map(|(entity, _)| Rc::clone(entity)))
    }

    fn sub_group(&self, id: &Uuid) -> Option<Rc<RefCell<dyn Group>>> {
        self.sub_groups
            .get(id)
            .map(|(sub_group, _)| Rc::clone(sub_group))
    }

    fn sub_groups(&self) -> Box<dyn Iterator<Item = Rc<RefCell<dyn Group>>> + '_> {
        Box::new(
            self.sub_groups
                .values()
                .map(|(sub_group, _)| Rc::clone(sub_group)),
        )
    }

    fn tick(&mut self, tick: &Tick) {
        for (_, (entity, _)) in &mut self.entities {
            entity.borrow_mut().tick(tick);
        }

        for (_, (sub_group, _)) in &mut self.sub_groups {
            sub_group.borrow_mut().tick(tick);
        }
    }

    fn changed(&self) -> Receiver<GroupMessage> {
        self.channel.1.clone()
    }

    fn should_update(&self) -> bool {
        *self.should_update.borrow()
    }

    // fn mark_update(&mut self) {
    //     self.should_update = true;
    // }

    fn update(&mut self, parent: Option<&dyn Group>) {
        let should_update = *self.should_update.borrow();
        let should_recalculate_matrices = *self.should_recalculate_matrices.borrow();
        let should_recalculate_bounding = *self.should_recalculate_bounding.borrow();
        let enable_bounding = self.enable_bounding;

        if should_update {
            if should_recalculate_matrices {
                self.update_matrices(parent);
                *self.should_recalculate_matrices.borrow_mut() = false;
            }

            match (enable_bounding, should_recalculate_bounding) {
                (true, true) => {
                    self.update_children_and_bounding_volume();
                    *self.should_recalculate_bounding.borrow_mut() = false;
                }
                (true, false) => self.update_children(),
                (false, true) => {
                    self.update_children();
                    self.bounding_volume = None;
                }
                (false, false) => {
                    // do nothing
                }
            };

            *self.should_update.borrow_mut() = false;
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
