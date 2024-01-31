use std::marker::PhantomData;

/// Memory unsafe Lru cache line.
/// If there is only one node in the cache,
/// `most_recently` and `least_recently` are both point to a same node.
///
/// # Unsafe
///
/// This is an performance considered, memory unsafe and aggressive Lru cache line implementation.
/// Do not use this if you are not clearly understand this implementation.
///
/// Please remember following rules when you use this:
/// 1. Never drops a [`LruNode`] pointer outside [`Lru`].
/// 2. Never shares [`LruNode`]s between different [`Lru`].
pub(crate) struct Lru<T = ()> {
    most_recently: Option<*mut LruNode<T>>,
    least_recently: Option<*mut LruNode<T>>,
}

#[allow(unused)]
impl<T> Lru<T> {
    /// Constructs a new Lru cache line.
    pub(crate) fn new() -> Self {
        Self {
            most_recently: None,
            least_recently: None,
        }
    }

    /// Caches a new Lru node or updates a caching Lru Node.
    pub(crate) unsafe fn cache(&mut self, node: *mut LruNode<T>) {
        let Some(most_recently) = self.most_recently else {
            self.most_recently = Some(node);
            return;
        };
        if most_recently == node {
            return;
        }

        let Some(least_recently) = self.least_recently else {
            (*most_recently).more_recently = Some(node);
            (*most_recently).less_recently = None;
            (*node).less_recently = Some(most_recently);
            self.most_recently = Some(node);
            self.least_recently = Some(most_recently);
            return;
        };

        let more_recently = (*node).more_recently;
        let less_recently = (*node).less_recently;
        if let Some(more_recently) = more_recently {
            (*more_recently).less_recently = less_recently;
        }
        if let Some(less_recently) = less_recently {
            (*less_recently).more_recently = more_recently;
        }
        (*node).more_recently = None;
        (*node).less_recently = None;

        (*node).less_recently = Some(most_recently);
        (*most_recently).more_recently = Some(node);
        self.most_recently = Some(node);

        if least_recently == node {
            self.least_recently = more_recently;
        }
    }

    /// Removes a Lru node from cache line.
    pub(crate) unsafe fn remove(&mut self, node: *mut LruNode<T>) {
        let more_recently = (*node).more_recently;
        let less_recently = (*node).less_recently;
        if let Some(more_recently) = more_recently {
            (*more_recently).less_recently = less_recently;
        }
        if let Some(less_recently) = less_recently {
            (*less_recently).more_recently = more_recently;
        }
        (*node).more_recently = None;
        (*node).less_recently = None;

        if self
            .most_recently
            .map(|most_recently| most_recently == node)
            .unwrap_or(false)
        {
            self.most_recently = less_recently;
        }

        if self
            .least_recently
            .map(|least_recently| least_recently == node)
            .unwrap_or(false)
        {
            self.least_recently = more_recently;
        }

        drop(Box::from_raw(node));
    }

    /// Node length.
    pub(crate) unsafe fn len(&self) -> usize {
        self.iter_most_to_least().count()
    }

    /// Gets least recently node.
    pub(crate) unsafe fn least_recently(&self) -> Option<*mut LruNode<T>> {
        if let Some(least_recently) = self.least_recently {
            Some(least_recently)
        } else if let Some(most_recently) = self.most_recently {
            Some(most_recently)
        } else {
            None
        }
    }

    /// Gets most recently node.
    pub(crate) unsafe fn most_recently(&self) -> Option<*mut LruNode<T>> {
        if let Some(most_recently) = self.most_recently {
            Some(most_recently)
        } else {
            None
        }
    }

    pub(crate) unsafe fn iter_least_to_most<'a>(&'a self) -> LeastToMostIterator<'a, T> {
        LeastToMostIterator(self.least_recently, PhantomData)
    }

    pub(crate) unsafe fn iter_most_to_least<'a>(&'a self) -> MostToLeastIterator<'a, T> {
        MostToLeastIterator(self.most_recently, PhantomData)
    }
}

pub(crate) struct LeastToMostIterator<'a, T>(Option<*mut LruNode<T>>, PhantomData<&'a ()>);

impl<'a, T> Iterator for LeastToMostIterator<'a, T> {
    type Item = *mut LruNode<T>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let Some(current) = self.0.take() else {
                return None;
            };

            self.0 = (*current).more_recently;
            Some(current)
        }
    }
}

pub(crate) struct MostToLeastIterator<'a, T>(Option<*mut LruNode<T>>, PhantomData<&'a ()>);

impl<'a, T> Iterator for MostToLeastIterator<'a, T> {
    type Item = *mut LruNode<T>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let Some(current) = self.0.take() else {
                return None;
            };

            self.0 = (*current).less_recently;
            Some(current)
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

/// Lru node.
pub(crate) struct LruNode<T> {
    data: T,
    more_recently: Option<*mut LruNode<T>>,
    less_recently: Option<*mut LruNode<T>>,
}

#[allow(unused)]
impl<T> LruNode<T> {
    /// Constructs a new Lru node with `Box::leak`.
    pub(crate) unsafe fn new(data: T) -> *mut Self {
        Box::leak(Box::new(Self {
            data,
            more_recently: None,
            less_recently: None,
        }))
    }

    /// Gets data store inside this node.
    pub(crate) unsafe fn data(&self) -> &T {
        &self.data
    }

    /// Gets more recently node associated with this node.
    pub(crate) unsafe fn more_recently(&self) -> Option<*mut LruNode<T>> {
        self.more_recently
    }

    /// Gets less recently node associated with this node.
    pub(crate) unsafe fn less_recently(&self) -> Option<*mut LruNode<T>> {
        self.less_recently
    }
}

#[cfg(test)]
mod tests {
    use super::{Lru, LruNode};

    #[test]
    fn test_cache() {
        unsafe {
            let mut lru = Lru::new();
            let a = LruNode::new("A");
            let b = LruNode::new("B");
            let c = LruNode::new("C");
            let d = LruNode::new("D");
            let e = LruNode::new("E");
            lru.cache(a);
            lru.cache(b);
            lru.cache(c);
            lru.cache(d);
            lru.cache(e);

            assert_eq!(5, lru.len());
            let l2m = lru
                .iter_least_to_most()
                .map(|node| (*node).data().to_string())
                .collect::<Vec<_>>();
            assert_eq!(["A", "B", "C", "D", "E"], *l2m);
            let m2l = lru
                .iter_most_to_least()
                .map(|node| (*node).data().to_string())
                .collect::<Vec<_>>();
            assert_eq!(["E", "D", "C", "B", "A"], *m2l);

            lru.cache(e);
            assert_eq!(5, lru.len());
            let l2m = lru
                .iter_least_to_most()
                .map(|node| (*node).data().to_string())
                .collect::<Vec<_>>();
            assert_eq!(["A", "B", "C", "D", "E"], *l2m);
            let m2l = lru
                .iter_most_to_least()
                .map(|node| (*node).data().to_string())
                .collect::<Vec<_>>();
            assert_eq!(["E", "D", "C", "B", "A"], *m2l);

            lru.cache(a);
            assert_eq!(5, lru.len());
            let l2m = lru
                .iter_least_to_most()
                .map(|node| (*node).data().to_string())
                .collect::<Vec<_>>();
            assert_eq!(["B", "C", "D", "E", "A"], *l2m);
            let m2l = lru
                .iter_most_to_least()
                .map(|node| (*node).data().to_string())
                .collect::<Vec<_>>();
            assert_eq!(["A", "E", "D", "C", "B"], *m2l);

            lru.cache(c);
            assert_eq!(5, lru.len());
            let l2m = lru
                .iter_least_to_most()
                .map(|node| (*node).data().to_string())
                .collect::<Vec<_>>();
            assert_eq!(["B", "D", "E", "A", "C"], *l2m);
            let m2l = lru
                .iter_most_to_least()
                .map(|node| (*node).data().to_string())
                .collect::<Vec<_>>();
            assert_eq!(["C", "A", "E", "D", "B"], *m2l);
        }
    }

    #[test]
    fn test_remove() {
        unsafe {
            let mut lru = Lru::new();
            let a = LruNode::new("A");
            let b = LruNode::new("B");
            let c = LruNode::new("C");
            let d = LruNode::new("D");
            let e = LruNode::new("E");
            lru.cache(a);
            lru.cache(b);
            lru.cache(c);
            lru.cache(d);
            lru.cache(e);

            lru.remove(e);
            assert_eq!(4, lru.len());
            let l2m = lru
                .iter_least_to_most()
                .map(|node| (*node).data().to_string())
                .collect::<Vec<_>>();
            assert_eq!(["A", "B", "C", "D"], *l2m);
            let m2l = lru
                .iter_most_to_least()
                .map(|node| (*node).data().to_string())
                .collect::<Vec<_>>();
            assert_eq!(["D", "C", "B", "A"], *m2l);

            lru.remove(a);
            assert_eq!(3, lru.len());
            let l2m = lru
                .iter_least_to_most()
                .map(|node| (*node).data().to_string())
                .collect::<Vec<_>>();
            assert_eq!(["B", "C", "D"], *l2m);
            let m2l = lru
                .iter_most_to_least()
                .map(|node| (*node).data().to_string())
                .collect::<Vec<_>>();
            assert_eq!(["D", "C", "B"], *m2l);

            let f = LruNode::new("F");
            lru.cache(f);
            assert_eq!(4, lru.len());
            let l2m = lru
                .iter_least_to_most()
                .map(|node| (*node).data().to_string())
                .collect::<Vec<_>>();
            assert_eq!(["B", "C", "D", "F"], *l2m);
            let m2l = lru
                .iter_most_to_least()
                .map(|node| (*node).data().to_string())
                .collect::<Vec<_>>();
            assert_eq!(["F", "D", "C", "B"], *m2l);

            lru.remove(c);
            assert_eq!(3, lru.len());
            let l2m = lru
                .iter_least_to_most()
                .map(|node| (*node).data().to_string())
                .collect::<Vec<_>>();
            assert_eq!(["B", "D", "F"], *l2m);
            let m2l = lru
                .iter_most_to_least()
                .map(|node| (*node).data().to_string())
                .collect::<Vec<_>>();
            assert_eq!(["F", "D", "B"], *m2l);

            lru.remove(b);
            assert_eq!(2, lru.len());
            let l2m = lru
                .iter_least_to_most()
                .map(|node| (*node).data().to_string())
                .collect::<Vec<_>>();
            assert_eq!(["D", "F"], *l2m);
            let m2l = lru
                .iter_most_to_least()
                .map(|node| (*node).data().to_string())
                .collect::<Vec<_>>();
            assert_eq!(["F", "D"], *m2l);

            lru.remove(f);
            assert_eq!(1, lru.len());
            let l2m = lru
                .iter_least_to_most()
                .map(|node| (*node).data().to_string())
                .collect::<Vec<_>>();
            assert_eq!(["D"], *l2m);
            let m2l = lru
                .iter_most_to_least()
                .map(|node| (*node).data().to_string())
                .collect::<Vec<_>>();
            assert_eq!(["D"], *m2l);

            lru.remove(d);
            assert_eq!(0, lru.len());
            let l2m = lru
                .iter_least_to_most()
                .map(|node| (*node).data().to_string())
                .collect::<Vec<_>>();
            assert_eq!([] as [&str; 0], *l2m);
            let m2l = lru
                .iter_most_to_least()
                .map(|node| (*node).data().to_string())
                .collect::<Vec<_>>();
            assert_eq!([] as [&str; 0], *m2l);
        }
    }
}
