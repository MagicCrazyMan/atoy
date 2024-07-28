use std::{any::TypeId, marker::PhantomData};

use hashbrown::HashSet;

pub struct Query {
    withs: HashSet<TypeId>,
    withouts: HashSet<TypeId>,
    maybes: HashSet<TypeId>,
}

impl Query {
    pub fn withs(&self) -> &HashSet<TypeId> {
        &self.withs
    }

    pub fn withouts(&self) -> &HashSet<TypeId> {
        &self.withouts
    }

    pub fn maybes(&self) -> &HashSet<TypeId> {
        &self.maybes
    }

    pub fn apply(&self, id: TypeId) -> bool {
        // self.withs.contains(value)
        todo!()
    }
}

pub trait QueryOp {
    // fn op(&self) ->
}

pub struct With<C>(PhantomData<C>);

pub struct Without<C>(PhantomData<C>);

pub struct Maybe<C>(PhantomData<C>);
