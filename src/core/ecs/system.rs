use crate::core::carrier::Listener;

pub trait System<D>
where
    D: ?Sized,
{
    fn execute(&self, message: &D);
}

impl<S, D> Listener<D> for S
where
    S: System<D>,
    D: ?Sized,
{
    fn execute(&self, message: &D) {
        System::<D>::execute(self, message);
    }
}
