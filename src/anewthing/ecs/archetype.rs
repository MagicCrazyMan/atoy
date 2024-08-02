use super::{
    component::{Component, ComponentKey, SharedComponentKey},
    error::Error,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Archetype(
    pub(super) Vec<ComponentKey>,       // non-shared components
    pub(super) Vec<SharedComponentKey>, // shared components
);

impl Archetype {
    /// Constructs a new empty archetype.
    pub fn new() -> Self {
        Self(Vec::new(), Vec::new())
    }

    /// Constructs a new archetype by a component.
    pub fn with_component<C>() -> Self
    where
        C: Component + 'static,
    {
        Self(vec![ComponentKey::new::<C>()], Vec::new())
    }

    /// Constructs a new archetype by a shared component.
    pub fn with_shared_component<C, T>() -> Self
    where
        C: Component + 'static,
        T: 'static,
    {
        Self(Vec::new(), vec![(SharedComponentKey::new::<C, T>())])
    }

    /// Returns all component keys.
    pub fn component_keys(&self) -> &[ComponentKey] {
        &self.0
    }

    /// Returns all shared component keys.
    pub fn shared_component_keys(&self) -> &[SharedComponentKey] {
        &self.1
    }

    /// Returns the number of non-shared components.
    pub fn components_len(&self) -> usize {
        self.0.len()
    }

    /// Returns the number of shared components.
    pub fn shared_components_len(&self) -> usize {
        self.1.len()
    }

    /// Returns the index of the component.
    pub fn component_index<C>(&self) -> Option<usize>
    where
        C: Component + 'static,
    {
        let key = ComponentKey::new::<C>();
        self.0.iter().position(|k| k == &key)
    }

    /// Returns true if the archetype has the component.
    pub fn has_component<C>(&self) -> bool
    where
        C: Component + 'static,
    {
        let key = ComponentKey::new::<C>();
        self.0.iter().any(|k| k == &key)
    }

    /// Returns true if the archetype has the shared component.
    pub fn has_shared_component<C, T>(&self) -> bool
    where
        C: Component + 'static,
        T: 'static,
    {
        let key = SharedComponentKey::new::<C, T>();
        self.1.iter().any(|k| k == &key)
    }

    pub fn add_component<C>(&self) -> Result<Self, Error>
    where
        C: Component + 'static,
    {
        let mut components = self.0.clone();
        components.push(ComponentKey::new::<C>());
        components.sort();
        components.dedup();
        if components.len() == self.0.len() {
            Ok(Self(components, self.1.clone()))
        } else {
            Err(Error::DuplicateComponent)
        }
    }

    pub fn remove_component<C>(&self) -> Result<Self, Error>
    where
        C: Component + 'static,
    {
        let key = ComponentKey::new::<C>();

        let mut components = self.0.clone();
        components.retain(|k| k != &key);
        if components.len() == self.0.len() {
            Err(Error::NoSuchComponent)
        } else {
            Ok(Self(components, self.1.clone()))
        }
    }

    pub fn add_shared_component<C, T>(&self) -> Result<Self, Error>
    where
        C: Component + 'static,
        T: 'static,
    {
        let mut components = self.1.clone();
        components.push(SharedComponentKey::new::<C, T>());
        components.sort();
        components.dedup();
        if components.len() == self.0.len() {
            Ok(Self(self.0.clone(), components))
        } else {
            Err(Error::DuplicateComponent)
        }
    }

    pub fn remove_shared_component<C, T>(&self) -> Result<Self, Error>
    where
        C: Component + 'static,
        T: 'static,
    {
        let key = SharedComponentKey::new::<C, T>();

        let mut components = self.1.clone();
        components.retain(|k| k != &key);
        if components.len() == self.0.len() {
            Err(Error::NoSuchComponent)
        } else {
            Ok(Self(self.0.clone(), components))
        }
    }
}

// pub trait ToArchetype {
//     fn to_archetype(&self) -> Archetype;
// }

// pub trait AsArchetype {
//     fn as_archetype() -> Archetype;
// }

// impl<A0> AsArchetype for A0
// where
//     A0: Component + 'static,
// {
//     fn as_archetype() -> Archetype {
//         Archetype::new([TypeId::of::<A0>()])
//     }
// }

// macro_rules! as_archetype {
//     ($($ct: tt),+) => {
//         impl<$($ct,)+> AsArchetype for ($($ct,)+)
//         where
//             $(
//                 $ct: Component + 'static,
//             )+
//         {
//             fn as_archetype() -> Archetype {
//                 Archetype::new([
//                     $(
//                         TypeId::of::<$ct>(),
//                     )+
//                 ])
//             }
//         }
//     };
// }

// as_archetype!(A0);
// as_archetype!(A0, A1);
// as_archetype!(A0, A1, A2);
// as_archetype!(A0, A1, A2, A3);
