use std::{any::Any, borrow::Cow, cell::RefCell, iter::FromIterator, marker::PhantomData, rc::Rc};

use hashbrown::{hash_map::Entry, HashMap};
use indexmap::IndexMap;
use uuid::Uuid;

type Ports = Rc<RefCell<HashMap<Cow<'static, str>, IndexMap<Uuid, Box<dyn Any>>>>>;

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
    pub fn sender<K>(&self) -> Sender<K>
    where
        K: MessageKind + 'static,
    {
        Sender::<K>::new(Rc::clone(&self.ports))
    }

    /// Constructs and returns a new receiver register from this channel.
    pub fn register<K>(&self) -> Register<K>
    where
        K: MessageKind + 'static,
    {
        Register::<K>::new(Rc::clone(&self.ports))
    }
}

/// A trait defining a message target and payload.
pub trait MessageKind {
    type Payload;

    fn target() -> Cow<'static, str>
    where
        Self: Sized;
}

/// Message sender sending data to channel under specified message kind.
#[derive(Debug, Clone)]
pub struct Sender<K> {
    ports: Ports,
    _kind: PhantomData<K>,
}

impl<K> Sender<K>
where
    K: MessageKind + 'static,
{
    fn new(ports: Ports) -> Self {
        Self {
            ports,
            _kind: PhantomData,
        }
    }

    /// Sends a new message to channel.
    pub fn send(&self, message: K::Payload) {
        let mut ports = self.ports.borrow_mut();
        let key = K::target();
        let Some(port) = ports.get_mut(&key) else {
            return;
        };

        let mut index = 0;
        while index < port.len() {
            let (_, receiver) = port.get_index(index).unwrap();
            let Some(receiver) = receiver.downcast_ref::<Box<dyn Receiver<K>>>() else {
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
pub trait Receiver<K>
where
    K: MessageKind + 'static,
{
    /// Executes code when receive a message.
    fn receive(&self, message: &K::Payload);

    /// Returns `true` if this receiver should abort.
    fn abort(&self) -> bool;
}

/// Message register registering a new receiver to the channel.
#[derive(Debug, Clone)]
pub struct Register<K> {
    ports: Ports,
    _kind: PhantomData<K>,
}

impl<K> Register<K>
where
    K: MessageKind + 'static,
{
    fn new(ports: Ports) -> Self {
        Self {
            ports,
            _kind: PhantomData,
        }
    }

    /// Registers a new receiver to the channel.
    pub fn register<R>(&self, receiver: R) -> Unregister<K>
    where
        R: Receiver<K> + 'static,
    {
        let mut ports = self.ports.borrow_mut();
        let key = K::target();

        let id = uuid::Uuid::new_v4();
        let receiver = Box::new(Box::new(receiver) as Box<dyn Receiver<K>>) as Box<dyn Any>;
        match ports.entry(key) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().insert(id, receiver);
            }
            Entry::Vacant(entry) => {
                let port = IndexMap::from_iter([(id, receiver)]);
                entry.insert(port);
            }
        };

        Unregister::<K>::new(id, Rc::clone(&self.ports))
    }
}

/// Message unregister removing a receiver from the channel.
pub struct Unregister<K>
where
    K: MessageKind + 'static,
{
    id: Uuid,
    ports: Ports,
    _kind: PhantomData<K>,
}

impl<K> Unregister<K>
where
    K: MessageKind + 'static,
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
        let key = K::target();
        let Some(port) = ports.get_mut(&key) else {
            return;
        };
        port.swap_remove(&self.id);
    }
}

// #[cfg(test)]
// mod tests {
//     use std::borrow::Cow;

//     use crate::channel::{MessageKind, Receiver};

//     use super::MessageChannel;

//     #[test]
//     fn test() {
//         pub struct TestMessageKind;
//         impl MessageKind for TestMessageKind {
//             type Payload = i32;

//             fn key() -> Cow<'static, str>
//             where
//                 Self: Sized,
//             {
//                 Cow::Borrowed("TestMessage")
//             }
//         }

//         pub struct TestReceiver;
//         impl Receiver<TestMessageKind> for TestReceiver {
//             fn receive(&self, message: &i32) {
//                 println!("message: {}", message);
//             }

//             fn abort(&self) -> bool {
//                 false
//             }
//         }

//         let channel = MessageChannel::new();
//         let register = channel.register::<TestMessageKind>();
//         let sender = channel.sender::<TestMessageKind>();
//         let unregister = register.register(TestReceiver);
//         sender.send(10);
//         sender.send(11);
//         sender.send(12);
//         unregister.unregister();
//         sender.send(13);
//         sender.send(14);
//         sender.send(15);
//     }
// }
