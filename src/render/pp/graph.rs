use std::collections::{HashSet, VecDeque};

use super::error::Error;

/// An orthogonal linked list based directed graph.
pub(super) struct DirectedGraph<T> {
    vertices: Vec<Vertex<T>>,
    arcs_len: usize,
}

impl<T> DirectedGraph<T> {
    /// Constructs a new directed graph.
    pub(super) fn new() -> Self {
        Self {
            vertices: Vec::new(),
            arcs_len: 0,
        }
    }

    /// Returns the number of vertices in the graph.
    pub(super) fn vertices_len(&self) -> usize {
        self.vertices.len()
    }

    /// Returns the number of arcs in the graph.
    pub(super) fn arcs_len(&self) -> usize {
        self.arcs_len
    }

    /// Validates whether this graph is an Activity On Vertex Network or not.
    pub(super) fn validate(&self) -> bool {
        if self.vertices.len() == 0 {
            return true;
        }

        let mut input_counts = Vec::with_capacity(self.vertices.len());
        let mut stack = VecDeque::with_capacity(self.vertices.len());
        let mut count = 0;
        // finds all vertices that have no input arc, and saves all input count of each vertex
        for vertex in self.vertices.iter() {
            input_counts.push(vertex.input_count);
            if vertex.input_count == 0 {
                stack.push_back(vertex);
            }
        }

        unsafe {
            while let Some(vertex) = stack.pop_back() {
                count += 1;

                // finds all output arc of this vertex
                let mut next_ptr = vertex.first_out;
                while let Some(current_ptr) = next_ptr.take() {
                    let current = &*current_ptr;
                    next_ptr = current.right;

                    // subtract 1 for input count of each input vertex
                    let input_count = &mut input_counts[current.to_index];
                    *input_count -= 1;

                    // adds vertex to stack if input count is 0
                    if *input_count == 0 {
                        stack.push_back(&self.vertices[current.to_index]);
                    }
                }
            }
        }

        // it is a validate AOV Network if count equals vertices len
        count == self.vertices.len()
    }

    /// Adds a new vertex to graph by given data.
    /// Vertex index in directed graph returned.
    pub(super) fn add_vertex(&mut self, data: T) -> usize {
        self.vertices.push(Vertex {
            data,
            input_count: 0,
            output_count: 0,
            first_in: None,
            first_out: None,
        });
        self.vertices.len() - 1
    }

    /// Removes a vertex to graph by vertex index.
    pub(super) fn remove_vertex(&mut self, index: usize) {
        // deletes all arcs that associated with removed vertex
        unsafe {
            let mut next_ptr = self.vertices[index].first_out;
            while let Some(current_ptr) = next_ptr.take() {
                let current = &mut *current_ptr;

                if let Some(top) = current.top {
                    (*top).bottom = current.bottom;
                }
                if let Some(bottom) = current.bottom {
                    (*bottom).top = current.top;
                }
                next_ptr = current.right;

                // resets first in and input count
                self.vertices[current.to_index].input_count -= 1;
                if current_ptr == self.vertices[current.to_index].first_in.unwrap() {
                    self.vertices[current.to_index].first_in = current.bottom;
                }

                drop(Box::from_raw(current_ptr));
                self.arcs_len -= 1;
            }
        }
        unsafe {
            let mut next_ptr = self.vertices[index].first_in;
            while let Some(current_ptr) = next_ptr.take() {
                let current = &mut *current_ptr;

                if let Some(left) = current.left {
                    (*left).right = current.right;
                }
                if let Some(right) = current.right {
                    (*right).left = current.left;
                }
                next_ptr = current.bottom;

                // resets first out and output count
                self.vertices[current.from_index].output_count -= 1;
                if current_ptr == self.vertices[current.from_index].first_out.unwrap() {
                    self.vertices[current.from_index].first_out = current.right;
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
            // 2. for partition with `from_index` > `index`, subtract 1 for `from_index` of each arc
            // 3. for partition with `to_index` > `index`, subtract 1 for `to_index` of each arc

            let len = self.vertices.len();
            // 2.
            {
                for from_index in index + 1..len {
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
                for to_index in index + 1..len {
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

        self.vertices.remove(index);
    }

    /// Gets vertex data from graph by vertex index.
    pub(super) fn vertex(&self, index: usize) -> Option<&T> {
        self.vertices.get(index).map(|vertex| &vertex.data)
    }

    /// Gets mutable vertex data from graph by vertex index.
    pub(super) fn vertex_mut(&mut self, index: usize) -> Option<&mut T> {
        self.vertices.get_mut(index).map(|vertex| &mut vertex.data)
    }

    /// Adds a new arc to graph to connect two vertices by vertex index and to another vertex index.
    /// 
    /// # Errors
    /// 
    /// - [`Error::SelfReferential`] if `from_index` equals the `to_index`.
    /// - [`Error::AlreadyConnected`] if arc already existing.
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
                    } else {
                        // marks as first out if nothing on left
                        out_vertex.first_out = Some(new_arc);
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
                    } else {
                        // marks as first in if nothing on top
                        in_vertex.first_in = Some(new_arc);
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
        self.vertices[from_index].output_count += 1;
        self.vertices[to_index].input_count += 1;

        Ok(())
    }

    /// Removes an arc from graph by from vertex index to another vertex index.
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
                    self.vertices[from_index].output_count -= 1;
                    self.vertices[to_index].input_count -= 1;
                } else if current.to_index < to_index {
                    next_ptr = current.right;
                }
            }
        }
    }

    /// As same as `dfs_iter()`.
    pub(super) fn iter(&self) -> Result<DfsIter<'_, T>, Error> {
        // do validation first before constructing iterator
        if self.validate() {
            Ok(DfsIter::new(self))
        } else {
            Err(Error::InvalidateGraph)
        }
    }

    /// As same as `dfs_iter_mut()`.
    pub(super) fn iter_mut(&mut self) -> Result<DfsIterMut<'_, T>, Error> {
        // do validation first before constructing iterator
        if self.validate() {
            Ok(DfsIterMut::new(self))
        } else {
            Err(Error::InvalidateGraph)
        }
    }

    pub(super) fn dfs_iter(&self) -> Result<DfsIter<'_, T>, Error> {
        // do validation first before constructing iterator
        if self.validate() {
            Ok(DfsIter::new(self))
        } else {
            Err(Error::InvalidateGraph)
        }
    }

    pub(super) fn dfs_iter_mut(&mut self) -> Result<DfsIterMut<'_, T>, Error> {
        // do validation first before constructing iterator
        if self.validate() {
            Ok(DfsIterMut::new(self))
        } else {
            Err(Error::InvalidateGraph)
        }
    }

    pub(super) fn bfs_iter(&self) -> Result<BfsIter<'_, T>, Error> {
        // do validation first before constructing iterator
        if self.validate() {
            Ok(BfsIter::new(self))
        } else {
            Err(Error::InvalidateGraph)
        }
    }

    pub(super) fn bfs_iter_mut(&mut self) -> Result<BfsIterMut<'_, T>, Error> {
        // do validation first before constructing iterator
        if self.validate() {
            Ok(BfsIterMut::new(self))
        } else {
            Err(Error::InvalidateGraph)
        }
    }
}

/// Graph iterator using depth first search and inputs controlling.
///
/// Graph should be ensured to be a VOA network before constructing an iterator.
pub(super) struct DfsIter<'a, T> {
    graph: &'a DirectedGraph<T>,
    stuff: Option<(Vec<usize>, VecDeque<(usize, &'a Vertex<T>)>, HashSet<usize>)>,
}

impl<'a, T> DfsIter<'a, T> {
    pub(super) fn new(graph: &'a DirectedGraph<T>) -> Self {
        Self { graph, stuff: None }
    }
}

impl<'a, T> Iterator for DfsIter<'a, T> {
    type Item = (usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.stuff.is_none() {
            let mut queue = VecDeque::with_capacity(self.graph.vertices_len());
            let mut visited = HashSet::with_capacity(self.graph.vertices_len());
            let mut input_counts = Vec::with_capacity(self.graph.vertices_len());
            // finds all vertices that have no input and collects input count of all vertices
            for (usize, vertex) in self.graph.vertices.iter().enumerate() {
                input_counts.push(vertex.input_count);
                if vertex.input_count == 0 {
                    queue.push_back((usize, vertex));
                    visited.insert(usize);
                }
            }

            self.stuff = Some((input_counts, queue, visited));
        }

        let (input_counts, queue, visited) = self.stuff.as_mut().unwrap();

        if queue.len() == 0 {
            return None;
        };

        unsafe {
            // finds first vertex that has no input
            let list_index = queue
                .iter()
                .position(|(index, _)| input_counts[*index] == 0)
                .unwrap(); // safely unwrap for a VOA network

            let (vertex_index, vertex) = queue.remove(list_index).unwrap(); // safe

            // subtracts input count for each to vertex
            let mut next_ptr = vertex.first_out;
            while let Some(current_ptr) = next_ptr.take() {
                let current = &*current_ptr;

                input_counts[current.to_index] -= 1;
                if !visited.contains(&current.to_index) {
                    queue.push_front((current.to_index, &self.graph.vertices[current.to_index]));
                    visited.insert(current.to_index);
                }

                next_ptr = current.right;
            }

            Some((vertex_index, &vertex.data))
        }
    }
}

/// Graph mutable iterator using depth first search and inputs controlling.
///
/// Graph should be ensured to be a VOA network before constructing an iterator.
pub(super) struct DfsIterMut<'a, T> {
    graph: &'a mut DirectedGraph<T>,
    stuff: Option<(
        Vec<usize>,
        VecDeque<(usize, *mut Vertex<T>)>,
        HashSet<usize>,
    )>,
}

impl<'a, T> DfsIterMut<'a, T> {
    pub(super) fn new(graph: &'a mut DirectedGraph<T>) -> Self {
        Self { graph, stuff: None }
    }
}

impl<'a, T> Iterator for DfsIterMut<'a, T> {
    type Item = (usize, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.stuff.is_none() {
            let mut queue = VecDeque::with_capacity(self.graph.vertices_len());
            let mut visited = HashSet::with_capacity(self.graph.vertices_len());
            let mut input_counts = Vec::with_capacity(self.graph.vertices_len());
            // finds all vertices that have no input and collects input count of all vertices
            for (usize, vertex) in self.graph.vertices.iter_mut().enumerate() {
                input_counts.push(vertex.input_count);
                if vertex.input_count == 0 {
                    queue.push_back((usize, vertex as *mut Vertex<T>));
                    visited.insert(usize);
                }
            }

            self.stuff = Some((input_counts, queue, visited));
        }

        let (input_counts, queue, visited) = self.stuff.as_mut().unwrap();

        if queue.len() == 0 {
            return None;
        };

        unsafe {
            // finds first vertex that has no input
            let list_index = queue
                .iter()
                .position(|(index, _)| input_counts[*index] == 0)
                .unwrap(); // safely unwrap for a VOA network

            let (vertex_index, vertex) = queue.remove(list_index).unwrap(); // safe

            // subtracts input count for each to vertex
            let mut next_ptr = (*vertex).first_out;
            while let Some(current_ptr) = next_ptr.take() {
                let current = &*current_ptr;

                input_counts[current.to_index] -= 1;
                if !visited.contains(&current.to_index) {
                    queue
                        .push_front((current.to_index, &mut self.graph.vertices[current.to_index]));
                    visited.insert(current.to_index);
                }

                next_ptr = current.right;
            }

            Some((vertex_index, &mut (*vertex).data))
        }
    }
}

/// Graph iterator using breadth first search and inputs controlling.
///
/// Graph should be ensured to be a VOA network before constructing an iterator.
pub(super) struct BfsIter<'a, T> {
    graph: &'a DirectedGraph<T>,
    stuff: Option<(Vec<usize>, Vec<(usize, &'a Vertex<T>)>, HashSet<usize>)>,
}

impl<'a, T> BfsIter<'a, T> {
    pub(super) fn new(graph: &'a DirectedGraph<T>) -> Self {
        Self { graph, stuff: None }
    }
}

impl<'a, T> Iterator for BfsIter<'a, T> {
    type Item = (usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.stuff.is_none() {
            let mut list = Vec::with_capacity(self.graph.vertices_len());
            let mut visited = HashSet::with_capacity(self.graph.vertices_len());
            let mut input_counts = Vec::with_capacity(self.graph.vertices_len());
            // finds all vertices that have no input and collects input count of all vertices
            for (usize, vertex) in self.graph.vertices.iter().enumerate() {
                input_counts.push(vertex.input_count);
                if vertex.input_count == 0 {
                    list.push((usize, vertex));
                    visited.insert(usize);
                }
            }

            self.stuff = Some((input_counts, list, visited));
        }

        let (input_counts, list, visited) = self.stuff.as_mut().unwrap();

        if list.len() == 0 {
            return None;
        };

        unsafe {
            // finds first vertex that has no input
            let list_index = list
                .iter()
                .position(|(index, _)| input_counts[*index] == 0)
                .unwrap(); // safely unwrap for a VOA network

            let (vertex_index, vertex) = list.remove(list_index);

            // subtracts input count for each to vertex
            let mut next_ptr = vertex.first_out;
            while let Some(current_ptr) = next_ptr.take() {
                let current = &*current_ptr;

                input_counts[current.to_index] -= 1;
                if !visited.contains(&current.to_index) {
                    list.push((current.to_index, &self.graph.vertices[current.to_index]));
                    visited.insert(current.to_index);
                }

                next_ptr = current.right;
            }

            Some((vertex_index, &vertex.data))
        }
    }
}

/// Graph mutable iterator using breadth first search and inputs controlling.
///
/// Graph should be ensured to be a VOA network before constructing an iterator.
pub(super) struct BfsIterMut<'a, T> {
    graph: &'a mut DirectedGraph<T>,
    stuff: Option<(Vec<usize>, Vec<(usize, *mut Vertex<T>)>, HashSet<usize>)>,
}

impl<'a, T> BfsIterMut<'a, T> {
    pub(super) fn new(graph: &'a mut DirectedGraph<T>) -> Self {
        Self { graph, stuff: None }
    }
}

impl<'a, T> Iterator for BfsIterMut<'a, T> {
    type Item = (usize, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.stuff.is_none() {
            let mut list = Vec::with_capacity(self.graph.vertices_len());
            let mut visited = HashSet::with_capacity(self.graph.vertices_len());
            let mut input_counts = Vec::with_capacity(self.graph.vertices_len());
            // finds all vertices that have no input and collects input count of all vertices
            for (usize, vertex) in self.graph.vertices.iter_mut().enumerate() {
                input_counts.push(vertex.input_count);
                if vertex.input_count == 0 {
                    list.push((usize, vertex as *mut Vertex<T>));
                    visited.insert(usize);
                }
            }

            self.stuff = Some((input_counts, list, visited));
        }

        let (input_counts, list, visited) = self.stuff.as_mut().unwrap();

        if list.len() == 0 {
            return None;
        };

        unsafe {
            // finds first vertex that has no input
            let list_index = list
                .iter()
                .position(|(index, _)| input_counts[*index] == 0)
                .unwrap(); // safely unwrap for a VOA network

            let (vertex_index, vertex) = list.remove(list_index);

            // subtracts input count for each to vertex
            let mut next_ptr = (*vertex).first_out;
            while let Some(current_ptr) = next_ptr.take() {
                let current = &*current_ptr;

                input_counts[current.to_index] -= 1;
                if !visited.contains(&current.to_index) {
                    list.push((current.to_index, &mut self.graph.vertices[current.to_index]));
                    visited.insert(current.to_index);
                }

                next_ptr = current.right;
            }

            Some((vertex_index, &mut (*vertex).data))
        }
    }
}

struct Vertex<T> {
    data: T,
    input_count: usize,
    output_count: usize,
    first_in: Option<*mut Arc>,
    first_out: Option<*mut Arc>,
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
        graph.add_arc(2, 3)?;
        graph.add_arc(2, 0)?;
        graph.add_arc(3, 4)?;
        assert_eq!(graph.arcs_len(), 7);

        assert_eq!(graph.vertices[0].output_count, 2);
        assert_eq!(graph.vertices[0].input_count, 2);

        assert_eq!(graph.vertices[1].output_count, 2);
        assert_eq!(graph.vertices[1].input_count, 1);

        assert_eq!(graph.vertices[2].output_count, 2);
        assert_eq!(graph.vertices[2].input_count, 1);

        assert_eq!(graph.vertices[3].output_count, 1);
        assert_eq!(graph.vertices[3].input_count, 1);

        assert_eq!(graph.vertices[4].output_count, 0);
        assert_eq!(graph.vertices[4].input_count, 2);

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
        assert_eq!(graph.vertices[0].output_count, 1);
        assert_eq!(graph.vertices[0].input_count, 2);
        assert_eq!(graph.vertices[1].output_count, 2);
        assert_eq!(graph.vertices[1].input_count, 0);
        assert_eq!(graph.vertices[0].first_out.is_some(), true);
        assert_eq!(graph.vertices[0].first_in.is_some(), true);
        assert_eq!(graph.vertices[1].first_out.is_some(), true);
        assert_eq!(graph.vertices[1].first_in.is_none(), true);
        // test delete again
        graph.remove_arc(0, 1);
        assert_eq!(graph.arcs_len(), 6);
        assert_eq!(graph.vertices[0].output_count, 1);
        assert_eq!(graph.vertices[0].input_count, 2);
        assert_eq!(graph.vertices[1].output_count, 2);
        assert_eq!(graph.vertices[1].input_count, 0);
        assert_eq!(graph.vertices[0].first_out.is_some(), true);
        assert_eq!(graph.vertices[0].first_in.is_some(), true);
        assert_eq!(graph.vertices[1].first_out.is_some(), true);
        assert_eq!(graph.vertices[1].first_in.is_none(), true);

        graph.remove_arc(3, 4);
        assert_eq!(graph.arcs_len(), 5);
        assert_eq!(graph.vertices[3].output_count, 0);
        assert_eq!(graph.vertices[3].input_count, 1);
        assert_eq!(graph.vertices[4].output_count, 0);
        assert_eq!(graph.vertices[4].input_count, 1);
        assert_eq!(graph.vertices[3].first_out.is_none(), true);
        assert_eq!(graph.vertices[3].first_in.is_some(), true);
        assert_eq!(graph.vertices[4].first_out.is_none(), true);
        assert_eq!(graph.vertices[4].first_in.is_some(), true);

        graph.remove_arc(2, 3);
        assert_eq!(graph.arcs_len(), 4);
        assert_eq!(graph.vertices[2].output_count, 1);
        assert_eq!(graph.vertices[2].input_count, 1);
        assert_eq!(graph.vertices[3].output_count, 0);
        assert_eq!(graph.vertices[3].input_count, 0);
        assert_eq!(graph.vertices[2].first_out.is_some(), true);
        assert_eq!(graph.vertices[2].first_in.is_some(), true);
        assert_eq!(graph.vertices[3].first_out.is_none(), true);
        assert_eq!(graph.vertices[3].first_in.is_none(), true);

        graph.remove_arc(0, 4);
        assert_eq!(graph.arcs_len(), 3);
        assert_eq!(graph.vertices[0].output_count, 0);
        assert_eq!(graph.vertices[0].input_count, 2);
        assert_eq!(graph.vertices[4].output_count, 0);
        assert_eq!(graph.vertices[4].input_count, 0);
        assert_eq!(graph.vertices[0].first_out.is_none(), true);
        assert_eq!(graph.vertices[0].first_in.is_some(), true);
        assert_eq!(graph.vertices[4].first_out.is_none(), true);
        assert_eq!(graph.vertices[4].first_in.is_none(), true);

        graph.remove_arc(1, 0);
        assert_eq!(graph.arcs_len(), 2);
        assert_eq!(graph.vertices[0].output_count, 0);
        assert_eq!(graph.vertices[0].input_count, 1);
        assert_eq!(graph.vertices[1].output_count, 1);
        assert_eq!(graph.vertices[1].input_count, 0);
        assert_eq!(graph.vertices[0].first_out.is_none(), true);
        assert_eq!(graph.vertices[0].first_in.is_some(), true);
        assert_eq!(graph.vertices[1].first_out.is_some(), true);
        assert_eq!(graph.vertices[1].first_in.is_none(), true);

        graph.remove_arc(1, 2);
        assert_eq!(graph.arcs_len(), 1);
        assert_eq!(graph.vertices[1].output_count, 0);
        assert_eq!(graph.vertices[1].input_count, 0);
        assert_eq!(graph.vertices[2].output_count, 1);
        assert_eq!(graph.vertices[2].input_count, 0);
        assert_eq!(graph.vertices[1].first_out.is_none(), true);
        assert_eq!(graph.vertices[1].first_in.is_none(), true);
        assert_eq!(graph.vertices[2].first_out.is_some(), true);
        assert_eq!(graph.vertices[2].first_in.is_none(), true);

        graph.remove_arc(2, 0);
        assert_eq!(graph.arcs_len(), 0);
        assert_eq!(graph.vertices[0].output_count, 0);
        assert_eq!(graph.vertices[0].input_count, 0);
        assert_eq!(graph.vertices[2].output_count, 0);
        assert_eq!(graph.vertices[2].input_count, 0);
        assert_eq!(graph.vertices[0].first_out.is_none(), true);
        assert_eq!(graph.vertices[0].first_in.is_none(), true);
        assert_eq!(graph.vertices[2].first_out.is_none(), true);
        assert_eq!(graph.vertices[2].first_in.is_none(), true);

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
        assert_eq!(graph.vertices[0].output_count, 1);
        assert_eq!(graph.vertices[0].input_count, 0);
        assert_eq!(graph.vertices[1].output_count, 1);
        assert_eq!(graph.vertices[1].input_count, 1);
        assert_eq!(graph.vertices[2].output_count, 1);
        assert_eq!(graph.vertices[2].input_count, 1);
        assert_eq!(graph.vertices[3].output_count, 0);
        assert_eq!(graph.vertices[3].input_count, 1);
        assert_eq!(graph.vertices[0].first_out.is_some(), true);
        assert_eq!(graph.vertices[0].first_in.is_none(), true);
        assert_eq!(graph.vertices[1].first_out.is_some(), true);
        assert_eq!(graph.vertices[1].first_in.is_some(), true);
        assert_eq!(graph.vertices[2].first_out.is_some(), true);
        assert_eq!(graph.vertices[2].first_in.is_some(), true);
        assert_eq!(graph.vertices[3].first_out.is_none(), true);
        assert_eq!(graph.vertices[3].first_in.is_some(), true);

        graph.remove_vertex(0);
        assert_eq!(graph.vertices_len(), 3);
        assert_eq!(graph.arcs_len(), 2);
        assert_eq!(graph.vertices[0].output_count, 1);
        assert_eq!(graph.vertices[0].input_count, 0);
        assert_eq!(graph.vertices[1].output_count, 1);
        assert_eq!(graph.vertices[1].input_count, 1);
        assert_eq!(graph.vertices[2].output_count, 0);
        assert_eq!(graph.vertices[2].input_count, 1);
        assert_eq!(graph.vertices[0].first_out.is_some(), true);
        assert_eq!(graph.vertices[0].first_in.is_none(), true);
        assert_eq!(graph.vertices[1].first_out.is_some(), true);
        assert_eq!(graph.vertices[1].first_in.is_some(), true);
        assert_eq!(graph.vertices[2].first_out.is_none(), true);
        assert_eq!(graph.vertices[2].first_in.is_some(), true);

        graph.remove_vertex(1);
        assert_eq!(graph.vertices_len(), 2);
        assert_eq!(graph.arcs_len(), 0);
        assert_eq!(graph.vertices[0].output_count, 0);
        assert_eq!(graph.vertices[0].input_count, 0);
        assert_eq!(graph.vertices[1].output_count, 0);
        assert_eq!(graph.vertices[1].input_count, 0);
        assert_eq!(graph.vertices[0].first_out.is_none(), true);
        assert_eq!(graph.vertices[0].first_in.is_none(), true);
        assert_eq!(graph.vertices[1].first_out.is_none(), true);
        assert_eq!(graph.vertices[1].first_in.is_none(), true);

        graph.remove_vertex(0);
        assert_eq!(graph.vertices_len(), 1);
        assert_eq!(graph.arcs_len(), 0);
        assert_eq!(graph.vertices[0].output_count, 0);
        assert_eq!(graph.vertices[0].input_count, 0);
        assert_eq!(graph.vertices[0].first_out.is_none(), true);
        assert_eq!(graph.vertices[0].first_in.is_none(), true);

        graph.remove_vertex(0);
        assert_eq!(graph.vertices_len(), 0);
        assert_eq!(graph.arcs_len(), 0);

        // Adds iterate test after deleting in the future

        Ok(())
    }

    #[test]
    fn test_validation() -> Result<(), Error> {
        let mut graph = DirectedGraph::<usize>::new();

        graph.add_vertex(0);
        graph.add_vertex(1);
        graph.add_vertex(2);
        graph.add_vertex(3);
        graph.add_vertex(4);
        graph.add_vertex(5);
        graph.add_vertex(6);
        graph.add_vertex(7);
        graph.add_vertex(8);
        graph.add_vertex(9);
        graph.add_vertex(10);
        graph.add_vertex(11);
        graph.add_vertex(12);
        graph.add_vertex(13);
        assert_eq!(graph.vertices_len(), 14);

        graph.add_arc(0, 4)?;
        graph.add_arc(0, 5)?;
        graph.add_arc(0, 11)?;
        graph.add_arc(1, 4)?;
        graph.add_arc(1, 8)?;
        graph.add_arc(1, 2)?;
        graph.add_arc(2, 5)?;
        graph.add_arc(2, 6)?;
        graph.add_arc(2, 9)?;
        graph.add_arc(3, 2)?;
        graph.add_arc(3, 13)?;
        graph.add_arc(4, 7)?;
        graph.add_arc(5, 8)?;
        graph.add_arc(5, 12)?;
        graph.add_arc(6, 5)?;
        graph.add_arc(9, 10)?;
        graph.add_arc(9, 11)?;
        graph.add_arc(10, 13)?;
        graph.add_arc(12, 9)?;
        assert_eq!(graph.arcs_len(), 19);
        assert_eq!(graph.validate(), true);

        graph.add_arc(0, 1)?;
        assert_eq!(graph.arcs_len(), 20);
        assert_eq!(graph.validate(), true);

        graph.add_arc(2, 0)?;
        assert_eq!(graph.arcs_len(), 21);
        assert_eq!(graph.validate(), false);

        graph.remove_arc(2, 0);
        assert_eq!(graph.arcs_len(), 20);
        assert_eq!(graph.validate(), true);

        graph.remove_arc(0, 1);
        assert_eq!(graph.arcs_len(), 19);
        assert_eq!(graph.validate(), true);

        Ok(())
    }

    #[test]
    fn test_dfs_iter() -> Result<(), Error> {
        let mut graph = DirectedGraph::<usize>::new();

        graph.add_vertex(0);
        graph.add_vertex(1);
        graph.add_vertex(2);
        graph.add_vertex(3);
        graph.add_vertex(4);
        graph.add_vertex(5);
        graph.add_vertex(6);
        assert_eq!(graph.vertices_len(), 7);

        graph.add_arc(0, 1)?;
        graph.add_arc(0, 2)?;
        graph.add_arc(0, 3)?;
        graph.add_arc(1, 6)?;
        graph.add_arc(2, 4)?;
        graph.add_arc(3, 5)?;
        graph.add_arc(3, 4)?;
        graph.add_arc(4, 6)?;
        graph.add_arc(5, 6)?;
        assert_eq!(graph.arcs_len(), 9);

        let data = graph.dfs_iter()?.map(|(_, data)| *data).collect::<Vec<_>>();

        assert_eq!(&data, &[0, 3, 5, 2, 4, 1, 6]);

        Ok(())
    }

    #[test]
    fn test_dfs_iter_mut() -> Result<(), Error> {
        let mut graph = DirectedGraph::<usize>::new();

        graph.add_vertex(0);
        graph.add_vertex(1);
        graph.add_vertex(2);
        graph.add_vertex(3);
        graph.add_vertex(4);
        graph.add_vertex(5);
        graph.add_vertex(6);
        assert_eq!(graph.vertices_len(), 7);

        graph.add_arc(0, 1)?;
        graph.add_arc(0, 2)?;
        graph.add_arc(0, 3)?;
        graph.add_arc(1, 6)?;
        graph.add_arc(2, 4)?;
        graph.add_arc(3, 5)?;
        graph.add_arc(3, 4)?;
        graph.add_arc(4, 6)?;
        graph.add_arc(5, 6)?;
        assert_eq!(graph.arcs_len(), 9);

        graph.dfs_iter_mut()?.for_each(|(_, data)| *data *= 20);
        let data = graph.dfs_iter()?.map(|(_, data)| *data).collect::<Vec<_>>();

        assert_eq!(&data, &[0, 60, 100, 40, 80, 20, 120]);

        Ok(())
    }

    #[test]
    fn test_bfs_iter() -> Result<(), Error> {
        let mut graph = DirectedGraph::<usize>::new();

        graph.add_vertex(0);
        graph.add_vertex(1);
        graph.add_vertex(2);
        graph.add_vertex(3);
        graph.add_vertex(4);
        graph.add_vertex(5);
        graph.add_vertex(6);
        assert_eq!(graph.vertices_len(), 7);

        graph.add_arc(0, 1)?;
        graph.add_arc(0, 2)?;
        graph.add_arc(0, 3)?;
        graph.add_arc(1, 6)?;
        graph.add_arc(2, 4)?;
        graph.add_arc(2, 5)?;
        graph.add_arc(3, 5)?;
        graph.add_arc(4, 6)?;
        graph.add_arc(5, 6)?;
        assert_eq!(graph.arcs_len(), 9);

        let data = graph.bfs_iter()?.map(|(_, data)| *data).collect::<Vec<_>>();

        assert_eq!(&data, &[0, 1, 2, 3, 4, 5, 6]);

        Ok(())
    }

    #[test]
    fn test_bfs_iter_mut() -> Result<(), Error> {
        let mut graph = DirectedGraph::<usize>::new();

        graph.add_vertex(0);
        graph.add_vertex(1);
        graph.add_vertex(2);
        graph.add_vertex(3);
        graph.add_vertex(4);
        graph.add_vertex(5);
        graph.add_vertex(6);
        assert_eq!(graph.vertices_len(), 7);

        graph.add_arc(0, 1)?;
        graph.add_arc(0, 2)?;
        graph.add_arc(0, 3)?;
        graph.add_arc(1, 6)?;
        graph.add_arc(2, 4)?;
        graph.add_arc(2, 5)?;
        graph.add_arc(3, 5)?;
        graph.add_arc(4, 6)?;
        graph.add_arc(5, 6)?;
        assert_eq!(graph.arcs_len(), 9);

        graph.bfs_iter_mut()?.for_each(|(_, data)| *data += 10);

        let data = graph.bfs_iter()?.map(|(_, data)| *data).collect::<Vec<_>>();

        assert_eq!(&data, &[10, 11, 12, 13, 14, 15, 16]);

        Ok(())
    }
}
