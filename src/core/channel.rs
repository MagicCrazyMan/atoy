use std::{
    any::{Any, TypeId},
    cell::RefCell,
    iter::FromIterator,
    marker::PhantomData,
    rc::Rc,
};

use hashbrown::{hash_map::Entry, HashMap};
use indexmap::IndexMap;
use uuid::Uuid;

type Ports = Rc<RefCell<HashMap<TypeId, IndexMap<Uuid, Box<dyn Any>>>>>;

/// Message channel is a middleware transferring message around the App.
#[derive(Debug, Clone)]
pub struct MessageChannel {
    ports: Ports,
}

impl MessageChannel {
    /// Constructs a new message channel.
    pub fn new() -> Self {
        Self {
            ports: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    /// Constructs and returns a new sender from this channel.
    pub fn sender<M>(&self) -> Sender<M>
    where
        M: 'static,
    {
        Sender::<M>::new(Rc::clone(&self.ports))
    }

    /// Constructs and returns a new receiver registry from this channel.
    pub fn registry<M>(&self) -> Registry<M>
    where
        M: 'static,
    {
        Registry::<M>::new(Rc::clone(&self.ports))
    }
}

/// Message sender sending data to channel under specified message kind.
#[derive(Debug)]
pub struct Sender<M> {
    ports: Ports,
    _kind: PhantomData<M>,
}

impl<M> Clone for Sender<M> {
    fn clone(&self) -> Self {
        Self {
            ports: Rc::clone(&self.ports),
            _kind: PhantomData,
        }
    }
}

impl<M> Sender<M>
where
    M: 'static,
{
    fn new(ports: Ports) -> Self {
        Self {
            ports,
            _kind: PhantomData,
        }
    }

    /// Sends a new message to channel.
    pub fn send(&self, message: M) {
        let mut ports = self.ports.borrow_mut();
        let Some(port) = ports.get_mut(&TypeId::of::<M>()) else {
            return;
        };

        let mut index = 0;
        while index < port.len() {
            let (_, receiver) = port.get_index(index).unwrap();
            let Some(receiver) = receiver.downcast_ref::<Box<dyn Receiver<M>>>() else {
                index += 1;
                continue;
            };
            receiver.receive(&message);
            if receiver.abort() {
                port.shift_remove_index(index);
            } else {
                index += 1;
            }
        }
    }
}

/// Message receiver receiving data from channel under specified message kind.
pub trait Receiver<M>
where
    M: 'static,
{
    /// Executes code when receive a message.
    fn receive(&self, message: &M);

    /// Returns `true` if this receiver should abort.
    fn abort(&self) -> bool {
        false
    }
}

/// Message register registering a new receiver to the channel.
#[derive(Debug, Clone)]
pub struct Registry<M> {
    ports: Ports,
    _kind: PhantomData<M>,
}

impl<M> Registry<M>
where
    M: 'static,
{
    fn new(ports: Ports) -> Self {
        Self {
            ports,
            _kind: PhantomData,
        }
    }

    /// Registers a new receiver to the channel.
    pub fn register<R>(&self, receiver: R) -> Unregister<M>
    where
        R: Receiver<M> + 'static,
    {
        let mut ports = self.ports.borrow_mut();

        let id = uuid::Uuid::new_v4();
        let receiver = Box::new(Box::new(receiver) as Box<dyn Receiver<M>>) as Box<dyn Any>;
        match ports.entry(TypeId::of::<M>()) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().insert(id, receiver);
            }
            Entry::Vacant(entry) => {
                let port = IndexMap::from_iter([(id, receiver)]);
                entry.insert(port);
            }
        };

        Unregister::<M>::new(id, Rc::clone(&self.ports))
    }
}

/// Message unregister removing a receiver from the channel.
pub struct Unregister<M>
where
    M: 'static,
{
    id: Uuid,
    ports: Ports,
    _kind: PhantomData<M>,
}

impl<M> Unregister<M>
where
    M: 'static,
{
    fn new(id: Uuid, ports: Ports) -> Self {
        Self {
            id,
            ports,
            _kind: PhantomData,
        }
    }

    /// Removes the associated receiver from the channel.
    pub fn unregister(self) {
        let mut ports = self.ports.borrow_mut();
        let Some(port) = ports.get_mut(&TypeId::of::<M>()) else {
            return;
        };
        port.swap_remove(&self.id);
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::core::channel::Receiver;

//     use super::MessageChannel;

//     #[test]
//     fn test() {
//         pub struct TestMessage(i32);

//         pub struct TestReceiver;
//         impl Receiver<TestMessage> for TestReceiver {
//             fn receive(&self, message: &TestMessage) {
//                 println!("message: {}", message.0);
//             }

//             fn abort(&self) -> bool {
//                 false
//             }
//         }

//         let channel = MessageChannel::new();
//         let register = channel.registry::<TestMessage>();
//         let sender = channel.sender::<TestMessage>();
//         let unregister = register.register(TestReceiver);
//         sender.send(TestMessage(10));
//         sender.send(TestMessage(11));
//         sender.send(TestMessage(12));
//         unregister.unregister();
//         sender.send(TestMessage(13));
//         sender.send(TestMessage(14));
//         sender.send(TestMessage(15));
//     }
// }
