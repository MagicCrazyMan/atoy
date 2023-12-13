/// Memory unsafe LRU cache line.
/// If there is only one node in the cache,
/// `most_recently` and `least_recently` are both point to the same node.
///
/// # Unsafe
///
/// This is an performance considered, memory unsafe and aggressive Lru cache line implementation.
/// Do not use this if you are not clearly understand this implementation.
///
/// Please remember following rules when you use this:
/// 1. Never drops a [`LruNode`] pointer outside [`Lru`].
/// 2. Never shares [`LruNode`]s between different [`Lru`].
pub struct Lru<T = ()> {
    len: usize,
    most_recently: Option<*mut LruNode<T>>,
    least_recently: Option<*mut LruNode<T>>,
}

impl<T> Lru<T> {
    /// Constructs a new Lru cache line.
    pub fn new() -> Self {
        Self {
            len: 0,
            most_recently: None,
            least_recently: None,
        }
    }

    /// Caches a new Lru node or updates a caching Lru Node.
    pub unsafe fn cache(&mut self, node: *mut LruNode<T>) {
        let is_new = (*node).less_recently.is_none() && (*node).more_recently.is_none();

        if let Some(more_recently) = (*node).more_recently {
            (*more_recently).less_recently = (*node).less_recently;
        }
        if let Some(less_recently) = (*node).less_recently {
            (*less_recently).more_recently = (*node).more_recently;
        }

        if let Some(most_recently) = self.most_recently {
            if most_recently != node {
                (*most_recently).more_recently = Some(node);
                (*node).less_recently = Some(most_recently);
                (*node).more_recently = None;
                self.most_recently = Some(node);
            }
        } else {
            (*node).less_recently = None;
            (*node).more_recently = None;
            self.most_recently = Some(node);
        }

        if self
            .least_recently
            .map(|least_recently| least_recently == node)
            .unwrap_or(false)
        {
            self.least_recently = (*node).more_recently;
        }

        if is_new {
            self.len += 1;

            if self.len == 2 {
                self.least_recently = (*node).less_recently;
            }
        }
    }

    /// Removes a Lru node from cache line.
    pub unsafe fn remove(&mut self, node: *mut LruNode<T>) {
        if (*node).less_recently.is_none() && (*node).more_recently.is_none() {
            return;
        }

        if let Some(more_recently) = (*node).more_recently {
            (*more_recently).less_recently = (*node).less_recently;
        }

        if let Some(less_recently) = (*node).less_recently {
            (*less_recently).more_recently = (*node).more_recently;
        }

        if self
            .most_recently
            .map(|most_recently| most_recently == node)
            .unwrap_or(false)
        {
            self.most_recently = (*node).less_recently;
        }

        if self
            .least_recently
            .map(|least_recently| least_recently == node)
            .unwrap_or(false)
        {
            self.least_recently = (*node).more_recently;
        }

        (*node).more_recently = None;
        (*node).less_recently = None;

        self.len -= 1;

        if self.len == 1 {
            self.least_recently = None;
        }

        drop(Box::from_raw(node));
    }

    /// Node length.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Gets least recently node.
    pub unsafe fn least_recently(&self) -> Option<*mut LruNode<T>> {
        if let Some(least_recently) = self.least_recently {
            Some(least_recently)
        } else if let Some(most_recently) = self.most_recently {
            Some(most_recently)
        } else {
            None
        }
    }

    /// Gets most recently node.
    pub unsafe fn most_recently(&self) -> Option<*mut LruNode<T>> {
        if let Some(most_recently) = self.most_recently {
            Some(most_recently)
        } else {
            None
        }
    }
}

impl<T> Drop for Lru<T> {
    fn drop(&mut self) {
        let Some(most_recently) = self.most_recently else {
            return;
        };

        unsafe {
            let mut next = Some(most_recently);
            while let Some(current) = next.take() {
                next = (*current).less_recently;
                drop(Box::from_raw(current));
            }
        }
    }
}

pub struct LruNode<T> {
    data: T,
    more_recently: Option<*mut LruNode<T>>,
    less_recently: Option<*mut LruNode<T>>,
}

impl<T> LruNode<T> {
    /// Constructs a new Lru node with `Box::leak`.
    pub unsafe fn new(data: T) -> *mut Self {
        Box::leak(Box::new(Self {
            data,
            more_recently: None,
            less_recently: None,
        }))
    }

    /// Gets data store inside this node.
    pub unsafe fn data(&self) -> &T {
        &self.data
    }

    /// Gets more recently node associated with this node.
    pub unsafe fn more_recently(&self) -> Option<*mut LruNode<T>> {
        self.more_recently
    }

    /// Gets less recently node associated with this node.
    pub unsafe fn less_recently(&self) -> Option<*mut LruNode<T>> {
        self.less_recently
    }
}

pub enum LruStrategy {
    Size,
}
