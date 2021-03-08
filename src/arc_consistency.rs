use std::iter::FromIterator;
use std::{
    collections::HashMap, collections::HashSet, fmt::Debug, hash::Hash, ops::Mul, sync::Mutex,
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

    fn remove(&mut self, x: &T) {
        self.items.retain(|v| v != x);
    }

    fn contains(&self, x: &T) -> bool {
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

    // Contracts two vertices x and y. The new node is labeled with x.
    fn contract_vertices(&mut self, x: &T, y: &T) {
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

    pub fn insert_edge(&mut self, u: &T, v: &T) {
        self.adjacency_list.get_mut(u).unwrap().0.insert(v.clone());
        self.adjacency_list.get_mut(v).unwrap().1.insert(u.clone());
    }

    fn remove_edge(&mut self, u: &T, v: &T) {
        self.adjacency_list.get_mut(u).unwrap().0.remove(v);
        self.adjacency_list.get_mut(v).unwrap().1.remove(u);
    }

    fn contains_edge(&self, u: &T, v: &T) -> bool {
        self.adjacency_list.get(u).unwrap().0.contains(v)
    }

    fn contract_edge(&mut self, u: &T, v: &T) {
        self.contract_vertices(u, v);
    }

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

impl<T: Eq + Hash + Clone + Sync + Send + Debug> AdjacencyList<T> {
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

pub fn write_dot<VertexID: Clone + Hash + Eq + Debug>(graph: &AdjacencyList<VertexID>) {
    println!("digraph {}", '{');
    for (u, v) in graph.edge_vec() {
        println!("\"{:?}\" -> \"{:?}\";", u, v);
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
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
{
    // list of vertices from g1 for each g0
    let mut f = HashMap::new();

    for v0 in g0.vertex_iter() {
        f.insert(v0.clone(), g1.vertex_iter().cloned().collect::<Set<_>>());
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

/// Implementation of the AC-3 algorithm by Mackworth 1977 specialized to find graph homomorphisms
///
/// f represents an unary constraint (a list of vertices?) for each vertex of g0
/// If there's no list specified for a vertex v, a list of all nodes of g1 is assigned to v
///
/// Returns None, if an empty domain is derived for some vertex v, otherwise an
/// arc-consistent map is returned.
pub fn ac3_precolour<V0, V1>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
    mut f: HashMap<V0, Set<V1>>,
) -> Option<HashMap<V0, Set<V1>>>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
{
    for v0 in g0.vertex_iter() {
        if !f.contains_key(&v0) {
            f.insert(v0.clone(), g1.vertex_iter().cloned().collect::<Set<_>>());
        }
    }

    let edges = g0.edge_vec();
    let mut worklist = HashSet::<(V0, V0, bool)>::new();

    for (x, y) in edges.iter().cloned() {
        worklist.insert((x.clone(), y.clone(), false));
        worklist.insert((y, x, true));
    }

    // list of worklist items for each vertex of g0
    // they're added to worklist, if the domain of the respective vertex changed
    let mut items = HashMap::new();

    for v in g0.vertex_iter() {
        items.insert(v.clone(), Vec::<(V0, V0, bool)>::new());
    }

    for (x, y, dir) in worklist.iter().cloned() {
        items.get_mut(&y).unwrap().push((x, y, dir));
    }

    while !worklist.is_empty() {
        let (x, y, dir) = worklist.iter().cloned().next().unwrap();
        worklist.remove(&(x.clone(), y.clone(), dir));

        if arc_reduce(x.clone(), y, dir, &mut f, &g1) {
            // domain of x changed, was the emtpy list derived?
            if f.get(&x).unwrap().is_empty() {
                return None;
            } else {
                for item in items.get(&x).unwrap().iter().cloned() {
                    worklist.insert(item);
                }
            }
        }
    }
    Some(f)
}

// ac3 is a specialized version of ac3_precolour
pub fn ac3<V0, V1>(g0: &AdjacencyList<V0>, g1: &AdjacencyList<V1>) -> Option<HashMap<V0, Set<V1>>>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
{
    ac3_precolour(g0, g1, HashMap::new())
}

// Implementation of the arc-reduce operation from ac3.
// Returns true, if the domain of x was reduced, false otherwise.
fn arc_reduce<V0, V1>(
    x: V0,
    y: V0,
    dir: bool,
    f: &mut HashMap<V0, Set<V1>>,
    g1: &AdjacencyList<V1>,
) -> bool
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
{
    let mut changed = false;
    for vx in f.get(&x).unwrap().clone().iter() {
        let mut is_possible = false;
        for vy in f.get(&y).unwrap().iter() {
            if dir {
                if g1.contains_edge(vy, vx) {
                    is_possible = true;
                    break;
                }
            } else {
                if g1.contains_edge(vx, vy) {
                    is_possible = true;
                    break;
                }
            }
        }

        if !is_possible {
            f.get_mut(&x).unwrap().remove(&vx);
            changed = true;
        }
    }
    changed
}

// TODO add Hashmap parameter
pub fn dfs_ac3<V0, V1>(g0: &AdjacencyList<V0>, g1: &AdjacencyList<V1>) -> Option<HashMap<V0, V1>>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
{
    let f = match ac3(g0, g1) {
        Some(v) => v,
        None => return None,
    };
    let vec = f.clone().into_iter().collect::<Vec<_>>();

    if let Some(map) = dfs_ac3_rec(g0, g1, f, vec.into_iter()) {
        Some(
            map.iter()
                .map(|(k, v)| (k.clone(), v.iter().cloned().next().unwrap()))
                .collect(),
        )
    } else {
        return None;
    }
}

fn dfs_ac3_rec<V0, V1, I>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
    f: HashMap<V0, Set<V1>>,
    mut iter: I,
) -> Option<HashMap<V0, Set<V1>>>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
    I: Iterator<Item = (V0, Set<V1>)>,
{
    let (u, l) = if let Some(v) = iter.next() {
        v
    } else {
        return Some(f);
    };

    for v in l.iter() {
        let mut set = Set::new();
        set.insert(v.clone());

        let mut map = f.clone();
        *map.get_mut(&u).unwrap() = set;

        if ac3_precolour(g0, g1, map.clone()).is_some() {
            return dfs_ac3_rec(g0, g1, map, iter);
        }
    }
    return None;
}

pub fn sac<V0, V1>(g0: &AdjacencyList<V0>, g1: &AdjacencyList<V1>) -> Option<HashMap<V0, Set<V1>>>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
{
    sac_precolour(g0, g1, HashMap::new())
}

pub fn sac_precolour<V0, V1>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
    mut f: HashMap<V0, Set<V1>>,
) -> Option<HashMap<V0, Set<V1>>>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
{
    for v0 in g0.vertex_iter() {
        if !f.contains_key(&v0) {
            f.insert(v0.clone(), g1.vertex_iter().cloned().collect::<Set<_>>());
        }
    }

    let e = match ac3_precolour(g0, g1, f) {
        Some(v) => v,
        None => return None,
    };

    for (k, v) in e.iter() {
        for u in v.iter() {
            let mut set = Set::new();
            set.insert(u.clone());

            let mut map = e.clone();
            map.insert(k.clone(), set);

            if let None = ac3_precolour(g0, g1, map) {
                return None;
            };
        }
    }
    Some(e)
}

pub fn singleton_search<V0, V1>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
) -> Option<HashMap<V0, Set<V1>>>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
{
    let mut map = match sac(&g0, &g1) {
        Some(v) => v,
        None => return None,
    };

    for (v, l) in map.clone().iter() {
        let mut found = false;

        for u in l.iter() {
            let mut set = Set::new();
            set.insert(u.clone());

            let f = map.clone();
            map.insert(v.clone(), set);
            if let Some(e) = sac_precolour(g0, g1, f) {
                map = e;
                found = true;
            };
        }
        if !found {
            return None;
        }
    }

    Some(map)
}

pub fn dfs_sac_backtrack<V0, V1>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
) -> Option<HashMap<V0, V1>>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
{
    let f = match ac3(g0, g1) {
        Some(v) => v,
        None => return None,
    };
    let vec = f.clone().into_iter().collect::<Vec<_>>();
    let mut backtracked = false;

    if let Some(map) = dfs_sac_backtrack_rec(g0, g1, f, vec.into_iter(), &mut backtracked) {
        if backtracked {
            Some(
                map.iter()
                    .map(|(k, v)| (k.clone(), v.iter().cloned().next().unwrap()))
                    .collect(),
            )
        } else {
            return None;
        }
    } else {
        return None;
    }
}

fn dfs_sac_backtrack_rec<V0, V1, I>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
    f: HashMap<V0, Set<V1>>,
    mut iter: I,
    backtracked: &mut bool,
) -> Option<HashMap<V0, Set<V1>>>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
    I: Iterator<Item = (V0, Set<V1>)>,
{
    let (u, l) = if let Some(v) = iter.next() {
        v
    } else {
        return Some(f);
    };

    let mut counter = 0;
    for v in l.iter() {
        println!("Iterating {}", &counter);
        let mut set = Set::new();
        set.insert(v.clone());

        let mut map = f.clone();
        *map.get_mut(&u).unwrap() = set;

        if sac_precolour(g0, g1, map.clone()).is_some() {
            return dfs_sac_backtrack_rec(g0, g1, map, iter, backtracked);
        }
        counter += 1;
    }
    *backtracked = true;
    return None;
}

fn ac_prune<V0, V1>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
    f: &mut HashMap<V0, Set<V1>>,
    m: &mut HashMap<V0, HashSet<V1>>,
    counter: &mut HashMap<(V0, V0), HashMap<V1, u32>>,
    s_ac: &HashMap<(V0, V1), Vec<(V0, V1)>>,
    list_ac: &mut Vec<(V0, V1)>,
    s_sac: &mut HashMap<(V0, V1), Vec<(V0, V1)>>,
    list_sac: &mut Vec<(V0, V1)>,
) where
    V0: Eq + Clone + Hash + Clone,
    V1: Eq + Clone + Hash + Clone,
{
    while !list_ac.is_empty() {
        let (j, b) = list_ac.pop().unwrap();

        for (i, a) in s_ac.get(&(j.clone(), b)).unwrap().iter() {
            let mut count = counter
                .get_mut(&(i.clone(), j.clone()))
                .unwrap()
                .get_mut(&a)
                .unwrap();
            *count -= 1;

            if *count == 0 && !m.get(&i).unwrap().contains(&a) {
                let mut domain = f.get(&i).unwrap().clone();
                domain.remove(a);
                f.insert(i.clone(), domain);

                m.get_mut(&i).unwrap().insert(a.clone());

                list_ac.push((i.clone(), a.clone()));

                for (k, c) in s_sac.get(&(i.clone(), a.clone())).unwrap().iter() {
                    list_sac.push((k.clone(), c.clone()));
                }
            }
        }
    }
}

fn sac_init<V0, V1>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
    f: &HashMap<V0, Set<V1>>,
    m: &mut HashMap<V0, HashSet<V1>>,
    counter: &mut HashMap<(V0, V0), HashMap<V1, u32>>,
    s_ac: &HashMap<(V0, V1), Vec<(V0, V1)>>,
    list_ac: &Vec<(V0, V1)>,
    s_sac: &mut HashMap<(V0, V1), Vec<(V0, V1)>>,
    list_sac: &mut Vec<(V0, V1)>,
) where
    V0: Eq + Clone + Hash + Clone,
    V1: Eq + Clone + Hash + Clone,
{
    let mut e = f.clone();
    for i in g0.vertex_iter() {
        for a in f.get(&i).unwrap().iter() {
            let mut set = Set::new();
            set.insert(a.clone());

            let mut d = e.clone();
            d.insert(i.clone(), set);

            if let Some(_) = ac3_precolour(g0, g1, d) {
                for (j, l) in e.iter() {
                    for b in l.iter() {
                        let vec = s_sac.get_mut(&(j.clone(), b.clone())).unwrap();
                        vec.push((i.clone(), a.clone()));
                    }
                }
            } else {
                let mut domain = f.get(&i).unwrap().clone();
                domain.remove(a);
                e.insert(i.clone(), domain);

                m.get_mut(&i).unwrap().insert(a.clone());
                let mut list = vec![(i.clone(), a.clone())];

                ac_prune(g0, g1, &mut e, m, counter, s_ac, &mut list, s_sac, list_sac);

                for (k, c) in s_sac.get(&(i.clone(), a.clone())).unwrap().iter() {
                    list_sac.push((k.clone(), c.clone()));
                }
            }
        }
    }
}

pub fn sac2<V0, V1>(g0: &AdjacencyList<V0>, g1: &AdjacencyList<V1>, mut f: HashMap<V0, Set<V1>>)
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
{
    let mut m = HashMap::<V0, HashSet<V1>>::new();
    let mut counter = HashMap::<(V0, V0), HashMap<V1, u32>>::new();

    let s_ac = HashMap::<(V0, V1), Vec<(V0, V1)>>::new();
    let mut list_ac = Vec::<(V0, V1)>::new();
    let mut s_sac = HashMap::<(V0, V1), Vec<(V0, V1)>>::new();
    let mut list_sac = Vec::<(V0, V1)>::new();

    for v0 in g0.vertex_iter() {
        if !f.contains_key(&v0) {
            f.insert(v0.clone(), g1.vertex_iter().cloned().collect::<Set<_>>());
        }
    }

    // ac_init();
    ac_prune(
        g0,
        g1,
        &mut f,
        &mut m,
        &mut counter,
        &s_ac,
        &mut list_ac,
        &mut s_sac,
        &mut list_sac,
    );
    sac_init(
        g0,
        g1,
        &f,
        &mut m,
        &mut counter,
        &s_ac,
        &list_ac,
        &mut s_sac,
        &mut list_sac,
    );
    // sac_prune();
}
