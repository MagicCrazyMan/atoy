use uuid::Uuid;

/// Listener identifier, for removing listener from [`EventTarget`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ListenerIdentifier(Uuid);

impl ListenerIdentifier {
    fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

struct Listener<E> {
    id: ListenerIdentifier,
    execution_count: usize,
    max_execution_count: Option<usize>,
    func: Box<dyn FnMut(&E)>,
}

/// `EventTarget` is a common event listener registration and dispatch agency.
///
/// Developer could register a listener to `EventTarget` using [`EventTarget::on()`],
/// [`EventTarget::once()`] or [`EventTarget::on_until()`] methods and remove a listener using [`EventTarget::off()`].
/// As for raising an event, invoking the [`EventTarget::raise()`] method simply get the job done.
/// Checks the methods documentation for more details.
pub struct EventTarget<E>(Vec<Listener<E>>);

impl<E> EventTarget<E> {
    /// Constructs a new event target agency.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Adds a listener to event target.
    pub fn on<F: FnMut(&E) + 'static>(&mut self, f: F) -> ListenerIdentifier {
        let id = ListenerIdentifier::new();
        self.0.push(Listener {
            id,
            execution_count: 0,
            max_execution_count: None,
            func: Box::new(f),
        });
        id
    }

    /// Adds a listener to event target and execute it only once.
    pub fn once<F: FnMut(&E) + 'static>(&mut self, f: F) -> ListenerIdentifier {
        let id = ListenerIdentifier::new();
        self.0.push(Listener {
            id,
            execution_count: 0,
            max_execution_count: Some(1),
            func: Box::new(f),
        });
        id
    }

    /// Adds a listener to event target and execute it until it reaches the specified execution count.
    pub fn on_until<F: FnMut(&E) + 'static>(&mut self, f: F, count: usize) -> ListenerIdentifier {
        let id = ListenerIdentifier::new();
        self.0.push(Listener {
            id,
            execution_count: 0,
            max_execution_count: Some(count),
            func: Box::new(f),
        });
        id
    }

    /// Removes a listener with a specified [`ListenerIdentifier`] from event target.
    pub fn off(&mut self, id: &ListenerIdentifier) {
        let Some(index) = self.0.iter().position(|listener| &listener.id == id) else {
            return;
        };
        self.0.remove(index);
    }

    /// Raises an event, notifies and invokes all registered listeners.
    pub fn raise(&mut self, event: E) {
        let mut len = self.0.len();
        let mut i = 0;
        while i < len {
            let listener = self.0.get_mut(i).unwrap();
            let func = listener.func.as_mut();
            func(&event);

            listener.execution_count += 1;

            if let Some(max_count) = listener.max_execution_count {
                // removes listener if it reaches the max executions.
                if listener.execution_count >= max_count {
                    self.0.remove(i);
                    len -= 1;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
    }
}
