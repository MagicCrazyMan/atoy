use std::fmt::{Debug, Display};

pub enum Error {
    DuplicateComponent,
    EmptyComponents,
    NoSuchEntity,
    NoSuchComponent,
    ComponentInUsed,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for Error {}
