// use std::{
//     cell::RefCell,
//     rc::{Rc, Weak},
// };

// use gl_matrix4rust::mat4::Mat4;
// use uuid::Uuid;

// use super::{
//     bounding::BoundingVolume, command::entity_collector::EntityFiltering,
//     transparency::{EntityTransparency, Transparency}, versioning::Versioning, web::webgl::entity::WebGlEntity,
//     AsAny,
// };

// pub trait Component: AsAny {
//     // fn property(&self, name: &str) -> Option<&dyn Any>;
// }

// pub trait Entity: Versioning + AsAny {
//     fn id(&self) -> &Uuid;

//     fn local_matrix(&self) -> &Mat4<f64>;
    
//     fn bounding_volume(&self) -> Option<&BoundingVolume>;

//     fn child(&self, id: &Uuid) -> Option<&dyn Entity>;

//     fn child_mut(&mut self, id: &Uuid) -> Option<&mut dyn Entity>;

//     fn children(&self) -> &[&dyn Entity];

//     fn children_mut(&mut self) -> &[&mut dyn Entity];

//     fn transparency(&self) -> Option<Transparency>;

//     fn as_filtering(&self) -> Option<&dyn EntityFiltering>;

//     fn as_webgl(&self) -> Option<&dyn WebGlEntity>;

//     fn as_webgl_mut(&mut self) -> Option<&mut dyn WebGlEntity>;
// }

// pub trait Group {
    
// }

// pub type EntityShared = Rc<RefCell<dyn Entity>>;
// pub type EntitySharedWeak = Weak<RefCell<dyn Entity>>;

// pub struct Collection {
//     id: Uuid,
//     entities: Vec<EntityShared>,
//     version: usize,
// }

// impl Collection {
//     pub fn new() -> Self {
//         Self {
//             id: Uuid::new_v4(),
//             entities: Vec::new(),
//             version: 0,
//         }
//     }

//     pub fn id(&self) -> &Uuid {
//         &self.id
//     }

//     pub fn entities(&self) -> &Vec<EntityShared> {
//         &self.entities
//     }
// }

// impl Versioning for Collection {
//     fn version(&self) -> usize {
//         self.version
//     }

//     fn set_version(&mut self, version: usize) {
//         self.version = version;
//     }
// }
