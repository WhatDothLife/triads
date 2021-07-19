//! An adjacency-list that represents a graph.
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
    io::Write,
    iter::FromIterator,
    sync::Mutex,
};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

pub trait VertexID: Eq + Clone + Hash {}
impl VertexID for u32 {}
impl<T: VertexID> VertexID for Vec<T> {}

/// A simple set implemented as a wrapper around Vec.
#[derive(Clone, Debug, Default)]
pub struct Set<T: Eq> {
    items: Vec<T>,
}

impl<T: Eq> Set<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Inserts a value in the set.
    ///
    /// If the set did not have this value present, `true` is returned.
    ///
    /// If the set did have this value present, `false` is returned.
    pub fn insert(&mut self, x: T) {
        self.items.push(x);
    }

    /// Removes a value from the set, returning `true` if the key was previously
    /// in the set, `false` otherwise.
    pub fn remove(&mut self, x: &T) -> bool {
        let mut res = false;
        self.items
            .retain(|v| (v != x).then(|| res = true).is_some());
        res
    }

    /// Returns `true` if the set contains the vertex with the given value.
    pub fn contains(&self, x: &T) -> bool {
        self.items.contains(x)
    }

    /// Returns the number of elements in the set.
    pub fn size(&self) -> usize {
        self.items.len()
    }

    /// An iterator visiting all elements in arbitrary order.
    /// The iterator element type is `(&'a T)`.
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &T> + 'a {
        self.items.iter()
    }

    /// Returns `true` if the set contains no elements.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Clears the set, removing all values.
    pub fn clear(&mut self) {
        self.items.clear();
    }
}

impl<T: Eq> FromIterator<T> for Set<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Set {
            items: iter.into_iter().collect::<Vec<_>>(),
        }
    }
}

/// An adjacency list that represents a graph, implemented as a wrapper struct
/// around a `HashMap`. For each vertex the `HashMap` contains an ordered pair, the
/// adjacency lists, where the first entry and second entry contain all
/// successors and predecessors, respectively.
#[derive(Debug, Clone, Default)]
pub struct AdjacencyList<V: VertexID> {
    // Vertex -> (Out-Edges, In-Edges)
    adjacency_list: HashMap<V, (Set<V>, Set<V>)>,
}

impl<V: VertexID> AdjacencyList<V> {
    /// Creates an empty `AdjacencyList`.
    pub fn new() -> AdjacencyList<V> {
        AdjacencyList {
            adjacency_list: HashMap::new(),
        }
    }

    /// Adds a vertex to the graph.
    ///
    /// If the graph did not have this vertex present, `true` is returned.
    ///
    /// If the graph did have this vertex present, `false` is returned.
    pub fn add_vertex(&mut self, v: V) -> bool {
        if self.has_vertex(&v) {
            false
        } else {
            self.adjacency_list.insert(v, (Set::new(), Set::new()));
            true
        }
    }

    /// Removes a vertex from the graph, returning the ordered pair of adjacency
    /// lists if the vertex was previously in the graph.
    pub fn remove_vertex(&mut self, v: &V) -> Option<(Set<V>, Set<V>)> {
        // remove vertex
        if let Some((out_edges, in_edges)) = self.adjacency_list.remove(v) {
            // remove vertex from out-edge list of other vertices
            for u in in_edges.iter() {
                self.adjacency_list.get_mut(u).unwrap().0.remove(v);
            }

            // remove vertex from in-edge list of other vertices
            for u in out_edges.iter() {
                self.adjacency_list.get_mut(u).unwrap().1.remove(v);
            }

            Some((out_edges, in_edges))
        } else {
            None
        }
    }

    /// Returns `true` if the graph contains the given vertex, false otherwise.
    pub fn has_vertex(&self, v: &V) -> bool {
        self.adjacency_list.contains_key(v)
    }

    /// Contracts the vertex `y` with the vertex `x` so that the resulting vertex has id `x`.
    pub fn contract_vertices(&mut self, u: &V, v: &V) {
        assert!(u != v, "vertex can not be contracted with itself!");
        let (out_edges, in_edges) = self.remove_vertex(v).unwrap();

        for w in in_edges.iter() {
            self.add_edge(w, u);
        }

        for w in out_edges.iter() {
            self.add_edge(u, w);
        }
    }

    /// Returns the total count of neighboring vertices of the vertex `x`.
    pub fn degree(&self, v: &V) -> usize {
        let edges = self.adjacency_list.get(v).unwrap();
        edges.0.size() + edges.1.size()
    }

    /// Returns the total count of outgoing edges of the vertex `x`.
    pub fn out_degree(&self, v: &V) -> usize {
        let edges = self.adjacency_list.get(v).unwrap();
        edges.0.size()
    }

    /// Returns the total count of incoming edges of the vertex `x`.
    pub fn in_degree(&self, v: &V) -> usize {
        let edges = self.adjacency_list.get(v).unwrap();
        edges.1.size()
    }

    /// Adds an edge to the graph.
    ///
    /// If the graph did not have this edge present, `true` is returned.
    ///
    /// If the graph did have this edge present, `false` is returned.
    pub fn add_edge(&mut self, u: &V, v: &V) -> bool {
        if self.has_edge(u, v) {
            false
        } else {
            self.adjacency_list.get_mut(u).unwrap().0.insert(v.clone());
            self.adjacency_list.get_mut(v).unwrap().1.insert(u.clone());
            true
        }
    }

    /// Removes an edge from the graph, returning true if the edge was previously
    /// present, false otherwise.
    pub fn remove_edge(&mut self, u: &V, v: &V) -> bool {
        if self.has_edge(u, v) {
            self.adjacency_list.get_mut(u).unwrap().0.remove(v);
            self.adjacency_list.get_mut(v).unwrap().1.remove(u);
            true
        } else {
            false
        }
    }

    /// Returns `true` if the graph contains the given edge, false otherwise.
    pub fn has_edge(&self, u: &V, v: &V) -> bool {
        self.adjacency_list.get(u).unwrap().0.contains(v)
    }

    /// Returns an iterator over references to all of the vertices in the graph.
    pub fn vertices<'a>(&'a self) -> impl Iterator<Item = &V> + 'a {
        self.adjacency_list.keys()
    }

    /// Returns an iterator over all of the edges in the graph.
    pub fn edges(&self) -> impl Iterator<Item = (V, V)> {
        self.vertices()
            .flat_map(|u| {
                self.adjacency_list
                    // get edge list of vertex
                    .get(u)
                    .unwrap()
                    // iter out-edges
                    .0
                    .iter()
                    .map(|v| (u.clone(), v.clone()))
                    .collect::<Vec<_>>()
                    .into_iter()
            })
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// Contracts each two vertices of the graph that satisfy the predicate `p`.
    ///
    /// **NOTE:** The method has a quadratic running time. A linear running time
    /// can be achieved by generating sets of vertices that must be contracted
    /// and then do it by hand by using the [`AdjacencyList::contract_vertices`]
    /// method.
    /// [`AdjacencyList::contract_vertices`]: ./struct.AdjacencyList.html#method.contract_vertices
    pub fn contract_if(&mut self, p: impl Fn(&V, &V) -> bool) {
        let vertices = self.vertices().cloned().collect::<Vec<_>>();
        let mut removed = HashSet::<V>::new();

        for (i, v) in vertices.iter().enumerate() {
            if removed.contains(v) {
                continue;
            }
            for j in i + 1..vertices.len() {
                let w = vertices.get(j).unwrap();
                if removed.contains(w) {
                    continue;
                }
                if p(v, w) {
                    self.contract_vertices(v, w);
                    removed.insert(w.clone());
                }
            }
        }
    }

    /// Performs the union of G and H, which is the graph with vertex set V(G) ∪
    /// V(H).
    ///
    /// **NOTE:** Be aware whether the two vertex sets V(G) and V(H) are
    /// disjoint or not.
    pub fn union(&self, l: &AdjacencyList<V>) -> AdjacencyList<V> {
        let mut map1 = self.adjacency_list.clone();
        let map2 = l.adjacency_list.clone();

        map1.extend(map2.into_iter());
        AdjacencyList {
            adjacency_list: map1,
        }
    }

    /// Returns a vector of the (weakly connected) components of the graph.
    pub fn components(&self) -> Vec<AdjacencyList<V>> {
        let mut to_visit = self.vertices().cloned().collect::<HashSet<_>>();
        let mut components = Vec::new();

        while let Some(v) = to_visit.clone().iter().next() {
            let mut graph = AdjacencyList::new();
            self.components_rec(v, &mut graph, &mut to_visit);
            components.push(graph);
        }
        components
    }

    fn components_rec(&self, v: &V, graph: &mut AdjacencyList<V>, to_visit: &mut HashSet<V>) {
        to_visit.remove(v);
        graph.add_vertex(v.clone());

        let (out_edges, in_edges) = self.adjacency_list.get(v).unwrap();

        for u in out_edges.iter() {
            graph.add_vertex(u.clone());
            if !graph.has_edge(v, u) {
                graph.add_edge(v, u);
            }
            if !to_visit.contains(u) {
                continue;
            }
            self.components_rec(u, graph, to_visit);
        }
        for u in in_edges.iter() {
            graph.add_vertex(u.clone());
            if !graph.has_edge(u, v) {
                graph.add_edge(u, v);
            }
            if !to_visit.contains(u) {
                continue;
            }
            self.components_rec(u, graph, to_visit);
        }
    }

    /// Returns the component that contains the vertex `v`.
    pub fn component(&self, v: &V) -> AdjacencyList<V> {
        let mut visited = HashSet::<V>::new();
        let mut graph = AdjacencyList::new();

        self.component_rec(v, &mut graph, &mut visited);
        graph
    }

    fn component_rec(&self, v: &V, graph: &mut AdjacencyList<V>, visited: &mut HashSet<V>) {
        visited.insert(v.clone());
        graph.add_vertex(v.clone());

        let (out_edges, in_edges) = self.adjacency_list.get(v).unwrap();

        for u in out_edges.iter() {
            if visited.contains(u) {
                continue;
            }
            graph.add_vertex(u.clone());
            graph.add_edge(v, u);
            self.component_rec(u, graph, visited);
        }
        for u in in_edges.iter() {
            if visited.contains(u) {
                continue;
            }
            graph.add_vertex(u.clone());
            graph.add_edge(u, v);
            self.component_rec(u, graph, visited);
        }
    }
}

impl<T: VertexID + Debug> AdjacencyList<T> {
    /// Prints the graph in dot format.
    pub fn to_dot(&self, output: &mut impl Write) {
        let mut s = String::from("digraph {\n");
        for v in self.vertices() {
            s.push_str(&format!("\"{:?}\";\n", v));
        }
        for (u, v) in self.edges() {
            s.push_str(&format!("\"{:?}\" -> \"{:?}\";\n", u, v));
        }
        s.push('}');
        output
            .write_all(s.as_bytes())
            .expect("Could not write the dot file!");
    }

    // // TODO remove me when not needed anymore
    // pub fn to_dot_file(&self, path: &str) {
    //     let mut f = File::create(path).unwrap();
    //     self.to_dot(&mut f);
    // }
}

impl<T: VertexID + Sync + Send> AdjacencyList<T> {
    /// Returns the k-ary product graph. The resulting graph uses `Vec` to represent
    /// the resulting tuples. The method uses parallelism.
    /// # Examples
    ///
    /// ```rust
    /// let mut graph = AdjacencyList::new();
    /// graph.add_vertex(0);
    /// graph.add_vertex(1);
    /// graph.add_edge(&0, &1);
    ///
    /// let graph2 = graph.power(2);
    ///
    /// // Iterate over edges
    /// for (u, v) in graph2.edges() {
    ///     vertices.push(u);
    ///     vertices.push(v);
    /// }
    ///
    /// assert_eq!(vertices.len(), 4);
    /// ```
    pub fn power(&self, k: u32) -> AdjacencyList<Vec<T>> {
        let mut graph = AdjacencyList::new();
        let mut vertices = vec![vec![]];
        let mut edges = vec![(vec![], vec![])];

        for _ in 0..k {
            let _vertices = Mutex::new(Some(vec![]));
            vertices.par_iter().for_each(|vec| {
                for v in self.vertices().cloned() {
                    let mut vec = vec.clone();
                    vec.push(v);
                    _vertices.lock().unwrap().as_mut().unwrap().push(vec);
                }
            });
            vertices = _vertices.lock().unwrap().take().unwrap();
        }
        for vec in vertices {
            graph.add_vertex(vec);
        }

        for _ in 0..k {
            let _edges = Mutex::new(Some(Vec::<(Vec<T>, Vec<T>)>::new()));
            edges.par_iter().for_each(|(u, v)| {
                for (x, y) in self.edges() {
                    let mut w1 = u.clone();
                    let mut w2 = v.clone();
                    w1.push(x.clone());
                    w2.push(y.clone());
                    _edges.lock().unwrap().as_mut().unwrap().push((w1, w2));
                }
            });
            edges = _edges.lock().unwrap().take().unwrap();
        }
        for (u, v) in edges {
            graph.add_edge(&u, &v);
        }

        graph
    }
}
