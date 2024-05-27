use crate::core::carrier::Listener;

use super::archetype::AsArchetype;

pub trait System<D>
where
    D: ?Sized,
{
    type Query: AsArchetype;

    fn execute(&mut self, message: &mut D);
}

impl<S, D> Listener<D> for S
where
    S: System<D>,
    D: ?Sized,
{
    fn execute(&mut self, message: &mut D) {
        System::<D>::execute(self, message);
    }
}
