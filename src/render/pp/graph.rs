use std::collections::VecDeque;

use super::error::Error;

pub(super) struct DirectedGraph<T> {
    vertices: Vec<Vertex<T>>,
    arcs_len: usize,
}

impl<T> DirectedGraph<T> {
    pub(super) fn new() -> Self {
        Self {
            vertices: Vec::new(),
            arcs_len: 0,
        }
    }

    pub(super) fn vertices_len(&self) -> usize {
        self.vertices.len()
    }

    pub(super) fn arcs_len(&self) -> usize {
        self.arcs_len
    }

    pub(super) fn add_vertex(&mut self, data: T) -> usize {
        self.vertices.push(Vertex {
            data,
            first_in: None,
            first_out: None,
        });
        self.vertices.len() - 1
    }

    pub(super) fn remove_vertex(&mut self, index: usize) {
        // deletes all arcs that associated with removed vertex
        let vertex = self.vertices.remove(index);
        unsafe {
            let mut next_ptr = vertex.first_out;
            while let Some(current_ptr) = next_ptr.take() {
                let current = &mut *current_ptr;

                if let Some(top) = current.top {
                    (*top).bottom = current.bottom;
                }
                if let Some(bottom) = current.bottom {
                    (*bottom).top = current.top;
                }
                next_ptr = current.right;

                // resets first out
                {
                    let to_index = if current.to_index < index {
                        current.to_index
                    } else {
                        current.to_index - 1
                    };
                    if current_ptr == self.vertices[to_index].first_in.unwrap() {
                        self.vertices[to_index].first_in = current.bottom;
                    }
                }

                drop(Box::from_raw(current_ptr));
                self.arcs_len -= 1;
            }
        }
        unsafe {
            let mut next_ptr = vertex.first_in;
            while let Some(current_ptr) = next_ptr.take() {
                let current = &mut *current_ptr;

                if let Some(left) = current.left {
                    (*left).right = current.right;
                }
                if let Some(right) = current.right {
                    (*right).left = current.left;
                }
                next_ptr = current.bottom;

                // resets first in
                {
                    let from_index = if current.from_index < index {
                        current.from_index
                    } else {
                        current.from_index - 1
                    };
                    if current_ptr == self.vertices[from_index].first_out.unwrap() {
                        self.vertices[from_index].first_out = current.right;
                    }
                }

                // do not drop from_index == to_index here, it may cause double free
                if current.from_index != current.to_index {
                    drop(Box::from_raw(current_ptr));
                    self.arcs_len -= 1;
                }
            }
        }

        // rearranges index of each vertex that behind removed vertex
        unsafe {
            // splits matrix into 4 parts
            // 1. for partition with `from_index` < `index` and `to_index` < `index`, do nothings
            // 2. for partition with `from_index` >= `index`, subtract 1 for `from_index` of each arc
            // 3. for partition with `to_index` >= `index`, subtract 1 for `to_index` of each arc

            let len = self.vertices.len();
            // 2.
            {
                for from_index in index..len {
                    let vertex = &mut self.vertices[from_index];
                    let mut next_ptr = vertex.first_out;
                    while let Some(current_ptr) = next_ptr.take() {
                        let current = &mut *current_ptr;

                        current.from_index -= 1;
                        next_ptr = current.right;
                    }
                }
            }
            // 3.
            {
                for to_index in index..len {
                    let vertex = &mut self.vertices[to_index];
                    let mut next_ptr = vertex.first_in;
                    while let Some(current_ptr) = next_ptr.take() {
                        let current = &mut *current_ptr;

                        current.to_index -= 1;
                        next_ptr = current.bottom;
                    }
                }
            }
        }
    }

    pub(super) fn vertex(&self, index: usize) -> Option<&T> {
        self.vertices.get(index).map(|vertex| &vertex.data)
    }

    pub(super) fn vertex_mut(&mut self, index: usize) -> Option<&mut T> {
        self.vertices.get_mut(index).map(|vertex| &mut vertex.data)
    }

    pub(super) fn add_arc(&mut self, from_index: usize, to_index: usize) -> Result<(), Error> {
        if from_index == to_index {
            return Err(Error::SelfReferential);
        }

        unsafe {
            let new_arc = Box::new(Arc {
                from_index,
                to_index,
                top: None,
                left: None,
                right: None,
                bottom: None,
            });
            let new_arc = Box::leak(new_arc);

            // updates out arcs
            {
                let out_vertex = &mut self.vertices[from_index];
                if let Some(first_out) = out_vertex.first_out {
                    let mut left = None;
                    let mut right = None;

                    let mut next_ptr = Some(first_out);
                    while let Some(current_ptr) = next_ptr.take() {
                        let current = &mut *current_ptr;

                        if current.to_index == to_index {
                            return Err(Error::AlreadyConnected);
                        } else if current.to_index < to_index {
                            left = Some(current_ptr);
                            next_ptr = current.right;
                        } else {
                            right = Some(current_ptr);
                        }
                    }

                    (*new_arc).left = left;
                    (*new_arc).right = right;
                    if let Some(left) = left {
                        (*left).right = Some(new_arc);
                    }
                    if let Some(right) = right {
                        (*right).left = Some(new_arc);
                    }
                } else {
                    out_vertex.first_out = Some(new_arc);
                }
            }

            // updates in arcs
            {
                let in_vertex = &mut self.vertices[to_index];
                if let Some(first_in) = in_vertex.first_in {
                    let mut top = None;
                    let mut bottom = None;

                    let mut next_ptr = Some(first_in);
                    while let Some(current_ptr) = next_ptr.take() {
                        let current = &mut *current_ptr;

                        if current.from_index == from_index {
                            unreachable!();
                        } else if current.from_index < from_index {
                            top = Some(current_ptr);
                            next_ptr = current.bottom;
                        } else {
                            bottom = Some(current_ptr);
                        }
                    }

                    (*new_arc).top = top;
                    (*new_arc).bottom = bottom;
                    if let Some(top) = top {
                        (*top).bottom = Some(new_arc);
                    }
                    if let Some(bottom) = bottom {
                        (*bottom).top = Some(new_arc);
                    }
                } else {
                    in_vertex.first_in = Some(new_arc);
                }
            }
        }

        self.arcs_len += 1;

        Ok(())
    }

    pub(super) fn remove_arc(&mut self, from_index: usize, to_index: usize) {
        unsafe {
            let mut next_ptr = self.vertices[from_index].first_out;
            while let Some(current_ptr) = next_ptr.take() {
                let current = &mut *current_ptr;

                if current.to_index == to_index {
                    if let Some(top) = current.top {
                        (*top).bottom = current.bottom;
                    }
                    if let Some(bottom) = current.bottom {
                        (*bottom).top = current.top;
                    }
                    if let Some(left) = current.left {
                        (*left).right = current.right;
                    }
                    if let Some(right) = current.right {
                        (*right).left = current.left;
                    }

                    let out_vertex = &mut self.vertices[from_index];
                    // safely unwrap
                    if out_vertex.first_out.unwrap() == current_ptr {
                        out_vertex.first_out = current.right;
                    }

                    let in_vertex = &mut self.vertices[to_index];
                    // safely unwrap
                    if in_vertex.first_in.unwrap() == current_ptr {
                        in_vertex.first_in = current.bottom;
                    }

                    let arc = Box::from_raw(current);
                    drop(arc);

                    self.arcs_len -= 1;
                } else if current.to_index < to_index {
                    next_ptr = current.right;
                }
            }
        }
    }

    pub(super) fn iter(&self) -> Iter<'_, T> {
        Iter::new(self)
    }

    // pub(super) fn iter_mut(&mut self) -> IterMut<'_, T> {
    //     IterMut::new(self)
    // }
}

pub(super) struct Iter<'a, T> {
    graph: &'a DirectedGraph<T>,
    queue: VecDeque<(usize, &'a Vertex<T>)>,
}

impl<'a, T> Iter<'a, T> {
    pub(super) fn new(graph: &'a DirectedGraph<T>) -> Self {
        let queue = match graph.vertices.get(0) {
            Some(v) => VecDeque::from([(0, v)]),
            None => VecDeque::new(),
        };
        Self { graph, queue }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let Some((index, current)) = self.queue.pop_front() else {
            return None;
        };

        // finds all to vertices of current vertex
        unsafe {
            let mut next_ptr = current.first_out;
            while let Some(next) = next_ptr.take() {
                let next = &*next;

                self.queue
                    .push_back((next.to_index, &self.graph.vertices[next.to_index]));
            }
        }

        Some((index, &current.data))
    }
}

// pub(super) struct IterMut<'a, T> {
//     graph: &'a mut DirectedGraph<T>,
//     queue: VecDeque<(usize, *mut Vertex<T>)>,
// }

// impl<'a, T> IterMut<'a, T> {
//     pub(super) fn new(graph: &'a mut DirectedGraph<T>) -> Self {
//         let queue = match graph.vertices.get_mut(0) {
//             Some(v) => VecDeque::from([(0, v as *mut Vertex<T>)]),
//             None => VecDeque::new(),
//         };
//         Self { graph, queue }
//     }
// }

// impl<'a, T> Iterator for IterMut<'a, T> {
//     type Item = (usize, &'a mut T);

//     fn next(&mut self) -> Option<Self::Item> {
//         let Some((index, current)) = self.queue.pop_front() else {
//             return None;
//         };

//         // finds all to vertices of current vertex
//         unsafe {
//             let mut next_ptr = (*current).first_out;
//             while let Some(next) = next_ptr.take() {
//                 let next = &mut *next;

//                 self.queue
//                     .push_back((next.to_index, &mut self.graph.vertices[next.to_index]));
//             }

//             Some((index, &mut (*current).data))
//         }
//     }
// }

struct Vertex<T> {
    data: T,
    first_in: Option<*mut Arc>,
    first_out: Option<*mut Arc>,
}

impl<T> Vertex<T> {
    // pub(super) fn next_out(&self) -> Vec<&T> {
    //     let mut nodes = Vec::new();
    //     unsafe {
    //         let mut next_ptr = self.first_out;
    //         while let Some(current_ptr) = next_ptr.take() {
    //             let current = &*current_ptr;
    //             nodes.push(value)
    //         }
    //     }
    // }
}

struct Arc {
    from_index: usize,
    to_index: usize,
    top: Option<*mut Arc>,
    left: Option<*mut Arc>,
    right: Option<*mut Arc>,
    bottom: Option<*mut Arc>,
}

#[cfg(test)]
mod tests {
    use crate::render::pp::error::Error;

    use super::DirectedGraph;

    #[test]
    fn test_add_vertices() {
        let mut graph = DirectedGraph::<usize>::new();

        graph.add_vertex(0);
        graph.add_vertex(1);
        graph.add_vertex(2);

        assert_eq!(graph.vertices_len(), 3);
        for (i, vertex) in graph.vertices.iter().enumerate() {
            assert_eq!(vertex.data, i);
            assert_eq!(vertex.first_in.is_none(), true);
            assert_eq!(vertex.first_out.is_none(), true);
        }
    }

    #[test]
    fn test_remove_vertices() {
        let mut graph = DirectedGraph::<usize>::new();

        graph.add_vertex(0);
        graph.add_vertex(1);
        graph.add_vertex(2);

        graph.remove_vertex(0);
        assert_eq!(graph.vertices_len(), 2);
        assert_eq!(graph.vertices[0].data, 1);

        graph.remove_vertex(1);
        assert_eq!(graph.vertices_len(), 1);
        assert_eq!(graph.vertices[0].data, 1);

        graph.remove_vertex(0);
        assert_eq!(graph.vertices_len(), 0);
    }

    #[test]
    fn test_add_arcs() -> Result<(), Error> {
        let mut graph = DirectedGraph::<usize>::new();

        graph.add_vertex(0);
        graph.add_vertex(1);
        graph.add_vertex(2);
        graph.add_vertex(3);
        graph.add_vertex(4);
        assert_eq!(graph.vertices_len(), 5);

        graph.add_arc(0, 1)?;
        graph.add_arc(0, 4)?;
        graph.add_arc(1, 0)?;
        graph.add_arc(1, 2)?;
        graph.add_arc(2, 0)?;
        graph.add_arc(2, 3)?;
        graph.add_arc(3, 4)?;
        assert_eq!(graph.arcs_len(), 7);

        let e = graph.add_arc(0, 1);
        assert_eq!(e, Err(Error::AlreadyConnected));

        let e = graph.add_arc(1, 1);
        assert_eq!(e, Err(Error::SelfReferential));

        Ok(())
    }

    #[test]
    fn test_remove_arcs() -> Result<(), Error> {
        let mut graph = DirectedGraph::<usize>::new();

        graph.add_vertex(0);
        graph.add_vertex(1);
        graph.add_vertex(2);
        graph.add_vertex(3);
        graph.add_vertex(4);
        assert_eq!(graph.vertices_len(), 5);

        graph.add_arc(0, 1)?;
        graph.add_arc(0, 4)?;
        graph.add_arc(1, 0)?;
        graph.add_arc(1, 2)?;
        graph.add_arc(2, 0)?;
        graph.add_arc(2, 3)?;
        graph.add_arc(3, 4)?;
        assert_eq!(graph.arcs_len(), 7);

        graph.remove_arc(0, 1);
        assert_eq!(graph.arcs_len(), 6);
        graph.remove_arc(0, 1); // test delete again
        assert_eq!(graph.arcs_len(), 6);
        graph.remove_arc(3, 4);
        assert_eq!(graph.arcs_len(), 5);
        graph.remove_arc(2, 3);
        assert_eq!(graph.arcs_len(), 4);
        graph.remove_arc(0, 4);
        assert_eq!(graph.arcs_len(), 3);
        graph.remove_arc(1, 0);
        assert_eq!(graph.arcs_len(), 2);
        graph.remove_arc(1, 2);
        assert_eq!(graph.arcs_len(), 1);
        graph.remove_arc(2, 0);
        assert_eq!(graph.arcs_len(), 0);

        // Adds iterate test after deleting in the future

        Ok(())
    }

    #[test]
    fn test_remove_vertices_with_arcs() -> Result<(), Error> {
        let mut graph = DirectedGraph::<usize>::new();

        graph.add_vertex(0);
        graph.add_vertex(1);
        graph.add_vertex(2);
        graph.add_vertex(3);
        graph.add_vertex(4);
        assert_eq!(graph.vertices_len(), 5);

        graph.add_arc(0, 1)?;
        graph.add_arc(0, 4)?;
        graph.add_arc(1, 0)?;
        graph.add_arc(1, 2)?;
        graph.add_arc(2, 0)?;
        graph.add_arc(2, 3)?;
        graph.add_arc(3, 4)?;
        assert_eq!(graph.arcs_len(), 7);

        graph.remove_vertex(0);
        assert_eq!(graph.vertices_len(), 4);
        assert_eq!(graph.arcs_len(), 3);

        graph.remove_vertex(0);
        assert_eq!(graph.vertices_len(), 3);
        assert_eq!(graph.arcs_len(), 2);

        graph.remove_vertex(1);
        assert_eq!(graph.vertices_len(), 2);
        assert_eq!(graph.arcs_len(), 0);

        graph.remove_vertex(0);
        assert_eq!(graph.vertices_len(), 1);
        assert_eq!(graph.arcs_len(), 0);

        graph.remove_vertex(0);
        assert_eq!(graph.vertices_len(), 0);
        assert_eq!(graph.arcs_len(), 0);

        // Adds iterate test after deleting in the future

        Ok(())
    }
}
