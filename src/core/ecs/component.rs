use std::any::TypeId;

use crate::core::AsAny;

pub trait Component: AsAny {
    fn component_type() -> TypeId
    where
        Self: Sized;

    fn component_type_instanced(&self) -> TypeId;
}
