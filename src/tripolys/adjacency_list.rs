use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
    iter::FromIterator,
    ops::Mul,
    str::FromStr,
    sync::Mutex,
};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

#[derive(Clone, Debug)]
pub struct Set<T: Eq> {
    items: Vec<T>,
}

impl<T: Eq> Set<T> {
    pub fn new() -> Self {
        Set { items: Vec::new() }
    }

    pub fn insert(&mut self, x: T) {
        self.items.push(x);
    }

    pub fn remove(&mut self, x: &T) {
        self.items.retain(|v| v != x);
    }

    pub fn contains(&self, x: &T) -> bool {
        self.items.contains(x)
    }

    pub fn size(&self) -> usize {
        self.items.len()
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &T> + 'a {
        self.items.iter()
    }

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

#[derive(Debug, Clone)]
pub struct AdjacencyList<T: Eq + Hash + Clone> {
    //                 Vertex -> (Out-Edges, In-Edges)
    adjacency_list: HashMap<T, (Set<T>, Set<T>)>,
}

impl<T: Eq + Hash + Clone> AdjacencyList<T> {
    pub fn new() -> Self {
        AdjacencyList {
            adjacency_list: HashMap::new(),
        }
    }

    pub fn insert_vertex(&mut self, x: T) {
        if !self.contains_vertex(&x) {
            self.adjacency_list.insert(x, (Set::new(), Set::new()));
        }
    }

    pub fn remove_vertex(&mut self, x: &T) {
        // remove vertex
        let (out_edges, in_edges) = self.adjacency_list.remove(x).unwrap();

        // remove vertex from out-edge list of other vertices
        for u in in_edges.iter() {
            self.adjacency_list.get_mut(u).unwrap().0.remove(x);
        }

        // remove vertex from in-edge list of other vertices
        for u in out_edges.iter() {
            self.adjacency_list.get_mut(u).unwrap().1.remove(x);
        }
    }

    pub fn contains_vertex(&self, x: &T) -> bool {
        self.adjacency_list.contains_key(x)
    }

    // Contracts two vertices x and y. The new node is labeled with x.
    pub fn contract_vertices(&mut self, x: &T, y: &T) {
        let (out_edges, in_edges) = self.adjacency_list.get(y).unwrap().clone();

        for u in in_edges.iter() {
            if !self.contains_edge(u, x) {
                self.insert_edge(u, x);
            }
        }

        for u in out_edges.iter() {
            if !self.contains_edge(x, u) {
                self.insert_edge(x, u);
            }
        }

        self.remove_vertex(&y);
    }

    pub fn degree(&self, x: &T) -> u32 {
        let edges = self.adjacency_list.get(x).unwrap();
        (edges.0.size() + edges.1.size()) as u32
    }

    pub fn insert_edge(&mut self, u: &T, v: &T) {
        self.adjacency_list.get_mut(u).unwrap().0.insert(v.clone());
        self.adjacency_list.get_mut(v).unwrap().1.insert(u.clone());
    }

    pub fn remove_edge(&mut self, u: &T, v: &T) {
        self.adjacency_list.get_mut(u).unwrap().0.remove(v);
        self.adjacency_list.get_mut(v).unwrap().1.remove(u);
    }

    pub fn contains_edge(&self, u: &T, v: &T) -> bool {
        self.adjacency_list.get(u).unwrap().0.contains(v)
    }

    // fn contract_edge(&mut self, u: &T, v: &T) {
    //     self.contract_vertices(u, v);
    // }

    pub fn vertex_iter<'a>(&'a self) -> impl Iterator<Item = &T> + 'a {
        self.adjacency_list.keys()
    }

    pub fn edge_vec(&self) -> Vec<(T, T)> {
        self.vertex_iter()
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
    }
}

// TODO This assumes that the vertices of the dot are in list format, e.g. [1, 2] -> [2, 3]
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
        list.insert_vertex(v1.clone());
        list.insert_vertex(v2.clone());
        list.insert_edge(&v1, &v2);
    }

    list
}

impl<T, U> Mul<&AdjacencyList<U>> for &AdjacencyList<T>
where
    T: Eq + Hash + Clone,
    U: Eq + Hash + Clone,
{
    type Output = AdjacencyList<(T, U)>;

    fn mul(self, rhs: &AdjacencyList<U>) -> AdjacencyList<(T, U)> {
        let mut list = AdjacencyList::new();

        for v1 in self.vertex_iter().cloned() {
            for v2 in rhs.vertex_iter().cloned() {
                list.insert_vertex((v1.clone(), v2));
            }
        }

        for (x1, y1) in self.edge_vec().iter() {
            for (x2, y2) in rhs.edge_vec().iter() {
                list.insert_edge(&(x1.clone(), x2.clone()), &(y1.clone(), y2.clone()));
            }
        }

        list
    }
}

/// Contracts each two vertices of the `AdjacencyList` if they satisfy a predicate `p`.
impl<T: Eq + Hash + Clone> AdjacencyList<T> {
    pub fn contract_if(&mut self, p: impl Fn(&T, &T) -> bool) {
        let vs = self.vertex_iter().cloned().collect::<Vec<_>>();
        let mut removed = HashSet::<T>::new();

        for (i, v) in vs.iter().enumerate() {
            if removed.contains(&v) {
                continue;
            }

            for j in i + 1..vs.len() {
                let w = vs.get(j).unwrap();
                if p(v, w) {
                    self.contract_vertices(v, w);
                    removed.insert(w.clone());
                }
            }
        }
    }
}

/// Returns the k-ary product graph of the `AdjacencyList`.
impl<T: Eq + Hash + Clone + Sync + Send> AdjacencyList<T> {
    pub fn power(&self, k: u32) -> AdjacencyList<Vec<T>> {
        let mut list = AdjacencyList::new();

        let mut vertices = vec![vec![]];

        for _ in 0..k {
            let tmp_vec = Mutex::new(Some(vec![]));
            vertices.par_iter().for_each(|vec| {
                for u in self.vertex_iter().cloned() {
                    let mut v = vec.clone();
                    v.push(u);
                    tmp_vec.lock().unwrap().as_mut().unwrap().push(v);
                }
            });
            vertices = tmp_vec.lock().unwrap().take().unwrap();
        }

        for vec in vertices.into_iter() {
            list.insert_vertex(vec);
        }

        let mut edges = vec![(vec![], vec![])];

        for _ in 0..k {
            let tmp_vec = Mutex::new(Some(Vec::<(Vec<T>, Vec<T>)>::new()));
            edges.par_iter().for_each(|(v1, v2)| {
                for (u1, u2) in self.edge_vec().iter() {
                    let mut w1 = v1.clone();
                    let mut w2 = v2.clone();
                    w1.push(u1.clone());
                    w2.push(u2.clone());
                    tmp_vec.lock().unwrap().as_mut().unwrap().push((w1, w2));
                }
            });
            edges = tmp_vec.lock().unwrap().take().unwrap();
        }

        for (u, v) in edges.into_iter() {
            list.insert_edge(&u, &v);
        }

        list
    }
}

/// Prints the graph in dot format
pub fn write_dot<VertexID: Clone + Hash + Eq + Debug>(graph: &AdjacencyList<VertexID>) {
    println!("digraph {}", '{');
    for (u, v) in graph.edge_vec() {
        println!("\"{:?}\" -> \"{:?}\";", u, v);
    }
    println!("{}", '}');
}

impl<T: Eq + Hash + Clone + FromStr> AdjacencyList<T> {
    pub fn from_edge_list(list: &str) -> Result<AdjacencyList<T>, <T as FromStr>::Err> {
        let tree = list
            .split(&[',', '[', ']', ' '][..])
            .filter(|&x| !x.is_empty())
            .collect::<Vec<_>>();

        let mut list = AdjacencyList::<T>::new();
        for (i, _) in tree.iter().enumerate().step_by(2) {
            let v1 = tree[i].parse::<T>()?;
            let v2 = tree[i + 1].parse::<T>()?;

            list.insert_vertex(v1.clone());
            list.insert_vertex(v2.clone());
            list.insert_edge(&v1, &v2);
        }
        Ok(list)
    }
}
