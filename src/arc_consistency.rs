use std::collections::HashMap;
use std::hash::Hash;
use std::iter::FromIterator;
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
pub struct Set<T: Eq> {
    items: Vec<T>,
}

impl<T: Eq> Set<T> {
    fn new() -> Self {
        Set { items: Vec::new() }
    }

    fn insert(&mut self, x: T) {
        self.items.push(x);
    }

    fn remove(&mut self, x: &T) {
        self.items.retain(|v| v != x);
    }

    fn contains(&self, x: &T) -> bool {
        self.items.contains(x)
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = &T> + 'a {
        self.items.iter()
    }
}

impl<T: Eq> FromIterator<T> for Set<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Set {
            items: iter.into_iter().collect::<Vec<_>>(),
        }
    }
}

pub struct AdjacencyList<T: Eq + Hash> {
    //                 Vertex -> (Out-Edges, In-Edges)
    adjacency_list: HashMap<T, (Set<T>, Set<T>)>,
}

impl<T: Eq + Hash + Copy> AdjacencyList<T> {
    pub fn new() -> Self {
        AdjacencyList {
            adjacency_list: HashMap::new(),
        }
    }

    pub fn insert_vertex(&mut self, x: T) {
        self.adjacency_list.insert(x, (Set::new(), Set::new()));
    }

    fn remove_vertex(&mut self, x: &T) {
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

    fn contains_vertex(&self, x: &T) -> bool {
        self.adjacency_list.contains_key(x)
    }

    pub fn insert_edge(&mut self, u: &T, v: &T) {
        self.adjacency_list.get_mut(u).unwrap().0.insert(*v);
        self.adjacency_list.get_mut(v).unwrap().1.insert(*u);
    }

    fn remove_edge(&mut self, u: &T, v: &T) {
        self.adjacency_list.get_mut(u).unwrap().0.insert(*v);
        self.adjacency_list.get_mut(v).unwrap().1.insert(*u);
    }

    fn contains_edge(&self, u: &T, v: &T) -> bool {
        self.adjacency_list.get(u).unwrap().0.contains(v)
    }

    fn vertex_iter<'a>(&'a self) -> impl Iterator<Item = &T> + 'a {
        self.adjacency_list.keys()
    }

    fn edge_vec(&self) -> Vec<(T, T)> {
        self.vertex_iter()
            .map(|u| {
                self.adjacency_list
                    // get edge list of vertex
                    .get(u)
                    .unwrap()
                    // iter out-edges
                    .0
                    .iter()
                    .map(|v| (*u, *v))
                    .collect::<Vec<_>>()
                    .into_iter()
            })
            .flatten()
            .collect::<Vec<_>>()
    }
}

fn write_dot<VertexID: 'static + Copy + Hash + Eq + std::fmt::Display>(
    graph: &AdjacencyList<VertexID>,
) {
    println!("digraph {}", '{');
    for v in graph.vertex_iter() {
        println!("{};", v);
    }

    for (u, v) in graph.edge_vec() {
        println!("{} -> {};", u, v);
    }

    println!("{}", '}');
}

fn is_homomorphism<V0, V1>(
    f: impl Fn(&V0) -> V1,
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
) -> bool
where
    V0: Eq + Copy + Hash,
    V1: Eq + Copy + Hash,
{
    for (u, v) in g0.edge_vec().iter() {
        if !g1.contains_edge(&f(&u), &f(&v)) {
            return false;
        }
    }
    true
}

fn is_polymorphism<V0, V1>(
    f: impl Fn(&V0, &V0) -> V1,
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
) -> bool
where
    V0: Eq + Copy + Hash + std::fmt::Display,
    V1: Eq + Copy + Hash + std::fmt::Display,
{
    for (u1, v1) in g0.edge_vec().iter() {
        for (u2, v2) in g0.edge_vec().iter() {
            if !g1.contains_edge(&f(&u1, &u2), &f(&v1, &v2)) {
                println!(
                    "(f({}, {}) = {}, f({}, {}) = {}) is not an edge!",
                    &u1,
                    &u2,
                    f(&u1, &u2),
                    &v1,
                    &v2,
                    f(&v1, &v2)
                );
                return false;
            }
        }
    }
    true
}

// find mapping from g0 to g1
pub fn arc_consistency<V0, V1>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
) -> HashMap<V0, Set<V1>>
where
    V0: Eq + Copy + Hash,
    V1: Eq + Copy + Hash,
{
    // list of vertices from g1 for each g0
    let mut f = HashMap::new();

    for v0 in g0.vertex_iter() {
        f.insert(*v0, g1.vertex_iter().cloned().collect::<Set<_>>());
    }

    let edges = g0.edge_vec();

    let mut changed = true;
    while changed {
        changed = false;
        for (u0, v0) in edges.iter() {
            for u1 in f.get(&u0).unwrap().clone().iter() {
                let mut is_possible = false;
                for v1 in f.get(&v0).unwrap().iter() {
                    if g1.contains_edge(u1, v1) {
                        is_possible = true;
                        break;
                    }
                }

                if !is_possible {
                    f.get_mut(&u0).unwrap().remove(&u1);
                    changed = true;
                }
            }

            for v1 in f.get(&v0).unwrap().clone().iter() {
                let mut is_possible = false;
                for u1 in f.get(&u0).unwrap().iter() {
                    if g1.contains_edge(u1, v1) {
                        is_possible = true;
                        break;
                    }
                }

                if !is_possible {
                    f.get_mut(&v0).unwrap().remove(&v1);
                    changed = true;
                }
            }
        }
    }

    f
}
