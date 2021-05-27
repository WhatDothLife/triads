use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    fs::File,
    hash::Hash,
    io::Write,
    iter::FromIterator,
    sync::Mutex,
};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

#[derive(Clone, Debug)]
pub struct Set<T: Eq> {
    items: Vec<T>,
}

impl<T: Eq> Set<T> {
    /// Creates an empty `Set`.
    ///
    /// # Examples
    ///
    /// ```
    /// let set: Set<i32> = Set::new();
    /// ```
    pub fn new() -> Self {
        Set { items: Vec::new() }
    }

    /// Adds a value to the set.
    ///
    /// If the set did not have this value present, `true` is returned.
    ///
    /// If the set did have this value present, `false` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut set = Set::new();
    ///
    /// assert_eq!(set.insert(2), true);
    /// assert_eq!(set.insert(2), false);
    /// assert_eq!(set.len(), 1);
    /// ```
    pub fn insert(&mut self, x: T) {
        self.items.push(x);
    }

    /// Removes a value from the set, returning `true` if the key was previously
    /// in the set, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut set = Set::new();
    /// set.insert(1, "a");
    ///
    /// assert!(set.remove(&1));
    /// assert!(!set.remove(&1));
    /// ```
    pub fn remove(&mut self, x: &T) -> bool {
        let mut res = false;
        self.items
            .retain(|v| (v != x).then(|| res = true).is_some());
        res
    }

    /// Returns `true` if the set contains the vertex with the given value.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut s = Set::new();
    /// assert!(!s.contains(1));
    ///
    /// s.insert(1);
    /// assert!(s.contains(1));
    /// ```
    pub fn contains(&self, x: &T) -> bool {
        self.items.contains(x)
    }

    /// Returns the number of elements in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut s = Set::new();
    /// s.insert(1);
    /// s.insert(2);
    ///
    /// assert_eq!(s.size(), 2);
    /// ```
    pub fn size(&self) -> usize {
        self.items.len()
    }

    /// An iterator visiting all elements in arbitrary order.
    /// The iterator element type is `(&'a T)`.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut set = Set::new();
    /// set.insert(1);
    /// set.insert(2);
    /// set.insert(3);
    ///
    /// for v in set.iter() {
    ///     println!("vertex: {}", v);
    /// }
    /// ```
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &T> + 'a {
        self.items.iter()
    }

    /// Returns `true` if the set contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut s = Set::new();
    /// assert!(s.is_empty());
    ///
    /// s.insert(1);
    /// assert!(!s.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
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
/// around a HashMap. For each vertex the HashMap contains an ordered pair, the
/// adjacency lists, where the first entry and second entry contain all
/// successors and predecessors, respectively.
#[derive(Debug, Clone)]
pub struct AdjacencyList<T: Eq + Hash + Clone> {
    // Vertex -> (Out-Edges, In-Edges)
    adjacency_list: HashMap<T, (Set<T>, Set<T>)>,
}

impl<T: Eq + Hash + Clone> AdjacencyList<T> {
    /// Creates an empty `AdjacencyList`.
    ///
    /// # Examples
    ///
    /// ```
    /// let graph: Adjacency<i32> = Adjacency::new();
    /// ```
    pub fn new() -> AdjacencyList<T> {
        AdjacencyList {
            adjacency_list: HashMap::new(),
        }
    }

    /// Adds a vertex to the graph.
    ///
    /// If the graph did not have this vertex present, `true` is returned.
    ///
    /// If the graph did have this vertex present, `false` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut graph = AdjacencyList::new();
    ///
    /// assert_eq!(graph.add_vertex(2), true);
    /// assert_eq!(graph.add_vertex(2), false);
    /// assert_eq!(graph.len(), 1);
    /// ```
    pub fn add_vertex(&mut self, x: T) -> bool {
        if !self.has_vertex(&x) {
            self.adjacency_list.insert(x, (Set::new(), Set::new()));
            true
        } else {
            false
        }
    }

    /// Removes a vertex from the graph, returning the ordered pair of adjacency
    /// lists if the vertex was previously in the graph.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut graph = AdjacencyList::new();
    /// graph.add_vertex(1);
    ///
    /// assert!(graph.remove(&1).is_some());
    /// assert!(graph.remove(&1).is_none());
    /// ```
    pub fn remove_vertex(&mut self, x: &T) -> Option<(Set<T>, Set<T>)> {
        // remove vertex
        if let Some((out_edges, in_edges)) = self.adjacency_list.remove(x) {
            // remove vertex from out-edge list of other vertices
            for u in in_edges.iter() {
                self.adjacency_list.get_mut(u).unwrap().0.remove(x);
            }

            // remove vertex from in-edge list of other vertices
            for u in out_edges.iter() {
                self.adjacency_list.get_mut(u).unwrap().1.remove(x);
            }

            return Some((out_edges, in_edges));
        } else {
            None
        }
    }

    /// Returns `true` if the graph contains the given vertex, false otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut g = AdjacencyList::new();
    /// g.add_vertex(1);
    ///
    /// assert!(!g.has_vertex(&2));
    /// assert!(g.has_vertex(&1));
    /// ```
    pub fn has_vertex(&self, v: &T) -> bool {
        self.adjacency_list.contains_key(v)
    }

    /// Contracts the vertex `y` with the vertex `x` so that the resulting vertex has id `x`.
    /// TODO Contract with itself?
    ///
    /// # Examples
    ///
    /// ```
    /// let mut g = AdjacencyList::new();
    /// g.add_vertex(1);
    /// g.add_vertex(2);
    ///
    /// assert!(!g.has_vertex(&1));
    /// assert!(g.has_vertex(&1));
    /// ```
    pub fn contract_vertices(&mut self, x: &T, y: &T) {
        let (out_edges, in_edges) = self.adjacency_list.get(y).unwrap().clone();
        // let (out_edges, in_edges) = self.remove_vertex(&y);

        for u in in_edges.iter() {
            // TODO this looks awful
            if !self.has_edge(u, x) {
                self.add_edge(u, x);
            }
        }

        for u in out_edges.iter() {
            // TODO this looks awful
            if !self.has_edge(x, u) {
                self.add_edge(x, u);
            }
        }

        self.remove_vertex(&y);
    }

    /// Returns the total count of neighboring vertices of the vertex x.
    pub fn degree(&self, x: &T) -> usize {
        let edges = self.adjacency_list.get(x).unwrap();
        edges.0.size() + edges.1.size()
    }

    /// Adds an edge to the graph.
    ///
    /// If the graph did not have this edge present, `true` is returned.
    ///
    /// If the graph did have this edge present, `false` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut graph = AdjacencyList::new();
    ///
    /// assert_eq!(graph.add_vertex(2), true);
    /// assert_eq!(graph.add_vertex(2), false);
    /// assert_eq!(graph.len(), 1);
    /// ```
    pub fn add_edge(&mut self, u: &T, v: &T) {
        self.adjacency_list.get_mut(u).unwrap().0.insert(v.clone());
        self.adjacency_list.get_mut(v).unwrap().1.insert(u.clone());
    }

    /// Removes an edge from the graph, returning true if the edge was previously
    /// present, false otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut graph = AdjacencyList::new();
    /// graph.add_vertex(1);
    ///
    /// assert!(graph.remove(&1).is_some());
    /// assert!(graph.remove(&1).is_none());
    /// ```
    pub fn remove_edge(&mut self, u: &T, v: &T) -> bool {
        if self.has_edge(u, v) {
            self.adjacency_list.get_mut(u).unwrap().0.remove(v);
            self.adjacency_list.get_mut(v).unwrap().1.remove(u);
            true
        } else {
            false
        }
    }

    /// Returns `true` if the graph contains the given edge, false otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut g = AdjacencyList::new();
    /// g.add_vertex(1);
    /// g.add_vertex(2);
    /// g.add_edge(&1, &2);

    /// assert!(!g.has_edge(&1, &1));
    /// assert!(g.has_edge(&1, &2));
    /// ```
    pub fn has_edge(&self, u: &T, v: &T) -> bool {
        self.adjacency_list.get(u).unwrap().0.contains(v)
    }

    /// Returns an iterator over references to all of the vertices in the graph.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut graph = AdjacencyList::new();
    /// let mut vertices = vec![];
    ///
    /// graph.add_vertex(0);
    /// graph.add_vertex(1);
    /// graph.add_vertex(2);
    /// graph.add_vertex(3);
    ///
    /// // Iterate over vertices
    /// for v in graph.vertices() {
    ///     vertices.push(v);
    /// }
    ///
    /// assert_eq!(vertices.len(), 4);
    /// ```
    pub fn vertices<'a>(&'a self) -> impl Iterator<Item = &T> + 'a {
        self.adjacency_list.keys()
    }

    /// Returns an iterator over all of the edges in the graph.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut graph = AdjacencyList::new();
    /// let mut vertices = vec![];
    ///
    /// graph.add_vertex(0);
    /// graph.add_vertex(1);
    /// graph.add_vertex(2);
    /// graph.add_vertex(3);
    ///
    /// graph.add_edge(&0, &1);
    /// graph.add_edge(&2, &3);
    ///
    /// // Iterate over edges
    /// for (u, v) in graph.edges() {
    ///     vertices.push(u);
    ///     vertices.push(v);
    /// }
    ///
    /// assert_eq!(vertices.len(), 4);
    /// ```
    pub fn edges<'a>(&'a self) -> impl Iterator<Item = (T, T)> + 'a {
        self.vertices()
            .map(|u| {
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
            .flatten()
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// Contracts each two vertices of the graph that satisfy the predicate `p`.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut graph = AdjacencyList::new();
    ///
    /// graph.add_vertex(0);
    /// graph.add_vertex(1);
    /// graph.add_vertex(2);
    /// graph.add_vertex(3);
    ///
    /// graph.add_edge(&0, &1);
    /// graph.add_edge(&2, &3);
    /// graph.add_edge(&0, &3);
    ///
    /// graph.contract_if(|x, y| x + 3 == y)
    ///
    /// let mut vertices = vec![];
    /// let mut edges = vec![];
    ///
    /// // Iterate over vertices
    /// for v in graph.vertices() {
    ///     vertices.push(v);
    /// }
    ///
    /// // Iterate over edges
    /// for (u, v) in graph.edges() {
    ///     edges.push((u, v));
    /// }
    ///
    /// assert_eq!(vertices.len(), 3);
    /// assert_eq!(edges.len(), 2);
    /// ```
    pub fn contract_iff(&mut self, p: impl Fn(&T, &T) -> bool) {
        let vs = self.vertices().cloned().collect::<Vec<_>>();
        let mut removed = HashSet::<T>::new();

        for (i, v) in vs.iter().enumerate() {
            if removed.contains(&v) {
                continue;
            }

            for j in i + 1..vs.len() {
                let w = vs.get(j).unwrap();
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

    /// Performs the disjoint union of G and H, which is the graph with vertex
    /// set V(G) ∪ V(H).
    ///
    /// **NOTE:** The method assumes that the two vertex sets are disjoint.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut graph1 = AdjacencyList::new();
    /// graph1.add_vertex(0);
    /// graph1.add_vertex(1);
    /// graph1.add_edge(&0, &1);
    ///
    /// let mut graph2 = AdjacencyList::new();
    /// graph2.add_vertex(2);
    /// graph2.add_vertex(3);
    /// graph2.add_edge(&2, &3);
    ///
    /// let graph3 = graph1.union(&graph2);
    ///
    /// // Iterate over edges
    /// for (u, v) in graph3.edges() {
    ///     vertices.push(u);
    ///     vertices.push(v);
    /// }
    ///
    /// assert_eq!(vertices.len(), 4);
    /// ```
    pub fn union(&self, l: &AdjacencyList<T>) -> AdjacencyList<T> {
        let mut m1 = self.adjacency_list.clone();
        let m2 = l.adjacency_list.clone();

        m1.extend(m2.into_iter().map(|(k, v)| (k.clone(), v.clone())));
        AdjacencyList { adjacency_list: m1 }
    }

    /// Returns a vector of the (weakly connected) components of the graph.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut graph = AdjacencyList::new();
    /// graph.add_vertex(0);
    /// graph.add_vertex(1);
    /// graph.add_vertex(2);
    /// graph.add_vertex(3);
    ///
    /// graph.add_edge(&0, &1);
    /// graph.add_edge(&2, &3);
    ///
    /// assert_eq!(graph.components().len(), 2);
    /// ```
    pub fn components(&self) -> Vec<AdjacencyList<T>> {
        let mut to_visit = self.vertices().cloned().collect::<HashSet<_>>();
        let mut vec = Vec::new();

        while let Some(v) = to_visit.clone().iter().next() {
            let mut graph = AdjacencyList::new();
            self.components_rec(v, &mut graph, &mut to_visit);
            vec.push(graph);
        }
        vec
    }

    fn components_rec(&self, v: &T, graph: &mut AdjacencyList<T>, to_visit: &mut HashSet<T>) {
        // println!("Yeeee {}", count);
        to_visit.remove(v);
        graph.add_vertex(v.clone());

        let (o, i) = self.adjacency_list.get(v).unwrap();

        for u in o.iter() {
            graph.add_vertex(u.clone());
            if !graph.has_edge(v, u) {
                graph.add_edge(v, u);
            }
            if !to_visit.contains(u) {
                continue;
            }
            self.components_rec(u, graph, to_visit);
        }
        for u in i.iter() {
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

    pub fn component(&self, v: &T) -> AdjacencyList<T> {
        let mut visited = HashSet::<T>::new();
        let mut graph = AdjacencyList::new();

        self.component_rec(v, &mut graph, &mut visited);
        graph
    }

    fn component_rec(&self, v: &T, graph: &mut AdjacencyList<T>, visited: &mut HashSet<T>) {
        visited.insert(v.clone());
        graph.add_vertex(v.clone());

        let (o, i) = self.adjacency_list.get(v).unwrap();

        for u in o.iter() {
            if visited.contains(u) {
                continue;
            }
            graph.add_vertex(u.clone());
            graph.add_edge(v, u);
            self.component_rec(u, graph, visited);
        }
        for u in i.iter() {
            if visited.contains(u) {
                continue;
            }
            graph.add_vertex(u.clone());
            graph.add_edge(u, v);
            self.component_rec(u, graph, visited);
        }
    }
}

impl<T: Clone + Eq + Hash + Debug> AdjacencyList<T> {
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

    // TODO remove me when not needed anymore
    pub fn to_dot_file(&self, path: &str) {
        let mut f = File::create(path).unwrap();
        self.to_dot(&mut f);
    }

    pub fn contract_if(&mut self, p: impl Fn(&T, &T) -> bool) {
        let vs = self.vertices().cloned().collect::<Vec<_>>();
        let mut removed = HashSet::<T>::new();

        for (i, v) in vs.iter().enumerate() {
            if removed.contains(&v) {
                continue;
            }
            for j in i + 1..vs.len() {
                let w = vs.get(j).unwrap();
                if removed.contains(w) {
                    continue;
                }
                if p(v, w) {
                    self.contract_vertices(v, w);
                    println!("Contracting done...");
                    removed.insert(w.clone());
                }
            }
        }
    }
}

impl<T: Eq + Hash + Clone + Sync + Send> AdjacencyList<T> {
    /// Returns the k-ary product graph. The resulting graph uses `Vec` to represent
    /// the resulting (mathematical) tuple. The method uses parallelism.
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
            let tmp = Mutex::new(Some(vec![]));
            vertices.par_iter().for_each(|vec| {
                for u in self.vertices().cloned() {
                    let mut v = vec.clone();
                    v.push(u);
                    tmp.lock().unwrap().as_mut().unwrap().push(v);
                }
            });
            vertices = tmp.lock().unwrap().take().unwrap();
        }
        for vec in vertices.into_iter() {
            graph.add_vertex(vec);
        }
        for _ in 0..k {
            let tmp = Mutex::new(Some(Vec::<(Vec<T>, Vec<T>)>::new()));
            edges.par_iter().for_each(|(v1, v2)| {
                for (u1, u2) in self.edges() {
                    let mut w1 = v1.clone();
                    let mut w2 = v2.clone();
                    w1.push(u1.clone());
                    w2.push(u2.clone());
                    tmp.lock().unwrap().as_mut().unwrap().push((w1, w2));
                }
            });
            edges = tmp.lock().unwrap().take().unwrap();
        }
        for (u, v) in edges.into_iter() {
            graph.add_edge(&u, &v);
        }

        graph
    }
}

// TODO So far this assumes that the vertices of the dot are in list format, e.g. [1, 2] -> [2, 3]
/// Parses a graph from dot format into an `AdjacencyList`.
pub fn from_dot(dot: &str) -> AdjacencyList<Vec<u32>> {
    let mut list = AdjacencyList::<Vec<u32>>::new();
    let mut split_vec = dot.split_terminator('\n').collect::<Vec<_>>();
    split_vec.pop();
    split_vec.remove(0);
    let edges = split_vec
        .iter()
        .map(|x| x.split(&['[', ',', ' ', ']'][..]).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    for vec in edges {
        let v1 = vec![
            vec[1].parse::<u32>().unwrap(),
            vec[3].parse::<u32>().unwrap(),
        ];
        let v2 = vec![
            vec[7].parse::<u32>().unwrap(),
            vec[9].parse::<u32>().unwrap(),
        ];
        list.add_vertex(v1.clone());
        list.add_vertex(v2.clone());
        list.add_edge(&v1, &v2);
    }

    list
}
