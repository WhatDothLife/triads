use std::fmt::Debug;
use std::{collections::HashMap, collections::HashSet, hash::Hash};

use crate::tripolys::adjacency_list::{AdjacencyList, Set};

pub trait LocalConsistency<V0: Eq + Clone + Hash, V1: Eq + Clone + Hash>:
    Fn(&AdjacencyList<V0>, &AdjacencyList<V1>, Domains<V0, V1>) -> Option<Domains<V0, V1>>
{
}

impl<V0: Eq + Clone + Hash, V1: Eq + Clone + Hash, F> LocalConsistency<V0, V1> for F where
    F: Fn(&AdjacencyList<V0>, &AdjacencyList<V1>, Domains<V0, V1>) -> Option<Domains<V0, V1>>
{
}

/// Implementation of the AC-1 algorithm, specialized to find
/// graph homomorphisms.
///
/// f represents a list of vertices for each vertex of g0. If there's no list
/// specified for a vertex v, a list of all nodes of g1 is assigned to v.
///
/// Returns None, if an empty domain is derived for some vertex v, otherwise
/// arc-consistent domains are returned.
pub fn ac1_precolour<V0, V1>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
    mut domains: Domains<V0, V1>,
) -> Option<Domains<V0, V1>>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
{
    // for v0 in g0.vertices() {
    //     if !f.contains_key(&v0) {
    //         f.insert(v0.clone(), g1.vertices().cloned().collect::<Set<_>>());
    //     }
    // }

    let mut edges = g0.edges();

    let mut changed = true;
    while changed {
        changed = false;
        for (u0, v0) in &mut edges {
            for u1 in domains.get_domain(&u0).unwrap().clone().iter() {
                let mut is_possible = false;
                for v1 in domains.get_domain(&v0).unwrap().iter() {
                    if g1.has_edge(u1, v1) {
                        is_possible = true;
                        break;
                    }
                }

                if !is_possible {
                    domains.get_domain_mut(&u0).unwrap().remove(&u1);
                    if domains.get_domain(&u0).unwrap().is_empty() {
                        return None;
                    }
                    changed = true;
                }
            }

            for v1 in domains.get_domain(&v0).unwrap().clone().iter() {
                let mut is_possible = false;
                for u1 in domains.get_domain(&u0).unwrap().iter() {
                    if g1.has_edge(u1, v1) {
                        is_possible = true;
                        break;
                    }
                }

                if !is_possible {
                    domains.get_domain_mut(&v0).unwrap().remove(&v1);
                    if domains.get_domain(&v0).unwrap().is_empty() {
                        return None;
                    }
                    changed = true;
                }
            }
        }
    }

    Some(domains)
}

/// A modification of `ac1_precolour` that is initialized with a list of all nodes
/// of g1 for each node in g0.
pub fn ac1<V0, V1>(g0: &AdjacencyList<V0>, g1: &AdjacencyList<V1>) -> Option<Domains<V0, V1>>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
{
    ac1_precolour(g0, g1, Domains::from_lists(g0, g1))
}

/// Implementation of the AC-3 algorithm due to Mackworth 1977, specialized to
/// find graph homomorphisms.
///
/// f represents a list of vertices for each vertex of g0. If there's no list
/// specified for a vertex v, a list of all nodes of g1 is assigned to v.
///
/// Returns None, if an empty domain is derived for some vertex v, otherwise an
/// arc-consistent map is returned.
pub fn ac3_precolour<V0, V1>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
    mut f: Domains<V0, V1>,
) -> Option<Domains<V0, V1>>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
{
    // for v0 in g0.vertices() {
    //     if !f.contains_key(&v0) {
    //         f.insert(v0.clone(), g1.vertices().cloned().collect::<Set<_>>());
    //     }
    // }

    let edges = g0.edges();
    let mut worklist = HashSet::<(V0, V0, bool)>::new();

    for (x, y) in edges {
        worklist.insert((x.clone(), y.clone(), false));
        worklist.insert((y, x, true));
    }

    // list of worklist items for each vertex of g0
    // they're added to worklist, if the domain of the respective vertex changed
    let mut items = HashMap::new();

    for v in g0.vertices() {
        items.insert(v.clone(), Vec::<(V0, V0, bool)>::new());
    }

    for (x, y, dir) in worklist.iter().cloned() {
        items.get_mut(&y).unwrap().push((x, y, dir));
    }

    while !worklist.is_empty() {
        let (x, y, dir) = worklist.iter().cloned().next().unwrap();
        worklist.remove(&(x.clone(), y.clone(), dir));

        if arc_reduce(x.clone(), y, dir, &mut f, &g1) {
            // domain of x changed, was the empty list derived?
            if f.get_domain(&x).unwrap().is_empty() {
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

/// A modification of `ac3_precolour` that is initialized with a list of all nodes
/// of g1 for each node in g0.
pub fn ac3<V0, V1>(g0: &AdjacencyList<V0>, g1: &AdjacencyList<V1>) -> Option<Domains<V0, V1>>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
{
    ac3_precolour(g0, g1, Domains::from_lists(&g0, &g1))
}

// Implementation of the arc-reduce operation from ac3.
// Returns true, if the domain of x was reduced, false otherwise.
fn arc_reduce<V0, V1>(
    x: V0,
    y: V0,
    dir: bool,
    f: &mut Domains<V0, V1>,
    g1: &AdjacencyList<V1>,
) -> bool
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
{
    let mut changed = false;
    for vx in f.get_domain(&x).unwrap().clone().iter() {
        let mut is_possible = false;
        for vy in f.get_domain(&y).unwrap().iter() {
            if dir {
                if g1.has_edge(vy, vx) {
                    is_possible = true;
                    break;
                }
            } else {
                if g1.has_edge(vx, vy) {
                    is_possible = true;
                    break;
                }
            }
        }

        if !is_possible {
            f.get_domain_mut(&x).unwrap().remove(&vx);
            changed = true;
        }
    }
    changed
}

/// Implementation of the SAC-1 algorithm due to Bessiere and Debruyne 1997,
/// specialized to operate on graphs.
///
/// f represents a list of vertices for each vertex of g0. If there's no list
/// specified for a vertex v, a list of all nodes of g1 is assigned to v.
///
/// Returns None, if an empty domain is derived for some vertex v, otherwise
/// singleton-arc-consistent domains are returned.
pub fn sac1_precolour<V0, V1>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
    mut f: Domains<V0, V1>,
) -> Option<Domains<V0, V1>>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
{
    let mut e = match ac3_precolour(g0, g1, f) {
        Some(v) => v,
        None => return None,
    };

    let mut changed = true;
    while changed {
        changed = false;

        for (k, v) in e.clone().iter() {
            for u in v.iter() {
                let mut set = Set::new();
                set.insert(u.clone());

                let mut map = e.clone();
                map.insert(k.clone(), set);

                if let None = ac3_precolour(g0, g1, map) {
                    v.clone().remove(&u);
                    e.insert(k.clone(), v.clone());
                    changed = true;
                };
            }
            if v.is_empty() {
                return None;
            }
        }
    }
    Some(e)
}

/// A modification of `sac1_precolour` that is initialized with a list of all nodes
/// of g1 for each node in g0.
pub fn sac1<V0, V1>(g0: &AdjacencyList<V0>, g1: &AdjacencyList<V1>) -> Option<Domains<V0, V1>>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
{
    sac1_precolour(g0, g1, Domains::from_lists(g0, g1))
}

/// Performs a depth-first-search to find a mapping from `g0` to `g1` that is
/// locally consistent. The type of local consistency is determined by the
/// algorithm `algo`.
pub fn search_precolour<V0, V1, A>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
    domains: Domains<V0, V1>,
    ac: &A,
) -> Option<HashMap<V0, V1>>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
    A: LocalConsistency<V0, V1>,
{
    let domains_ac = if let Some(v) = ac(g0, g1, domains) {
        v
    } else {
        return None;
    };

    let iter = domains_ac.clone().into_iter();

    if let Some(map) = search_rec(g0, g1, domains_ac, iter, ac) {
        Some(
            map.iter()
                .map(|(k, v)| (k.clone(), v.iter().cloned().next().unwrap()))
                .collect(),
        )
    } else {
        return None;
    }
}

/// Recursive helper function.
fn search_rec<V0, V1, I, A>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
    f: Domains<V0, V1>,
    mut iter: I,
    ac: A,
) -> Option<Domains<V0, V1>>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
    I: Iterator<Item = (V0, Set<V1>)>,
    A: LocalConsistency<V0, V1>,
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
        *map.get_domain_mut(&u).unwrap() = set;

        if let Some(map_sac) = ac(g0, g1, map.clone()) {
            map = map_sac;
            return search_rec(g0, g1, map, iter, ac);
        }
    }
    return None;
}

pub fn find_precolour<V0, V1, A>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
    domains: Domains<V0, V1>,
    algo: &A,
) -> Option<Domains<V0, V1>>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
    A: LocalConsistency<V0, V1>,
{
    let mut domains = if let Some(v) = algo(g0, g1, domains) {
        v
    } else {
        return None;
    };

    for v0 in g0.vertices() {
        let dom = domains.get_domain(&v0).unwrap();
        if dom.size() > 1 {
            for v1 in dom.clone().iter() {
                let mut set = Set::new();
                set.insert(v1.clone());

                let mut domains_tmp = domains.clone();
                domains_tmp.insert(v0.clone(), set);

                if let Some(domains_algo) = algo(g0, g1, domains_tmp) {
                    domains = domains_algo;
                    break;
                }
            }
        }
    }
    Some(domains)
}

/// Implementation of the PC-2 algorithm by Mackworth 1977, specialized to work
/// on graphs.
///
/// Returns false, if an empty domain is derived for some vertex v, true otherwise.
pub fn pc2<V0, V1>(g0: &AdjacencyList<V0>, g1: &AdjacencyList<V1>) -> bool
where
    V0: Eq + Clone + Hash + Debug,
    V1: Eq + Clone + Hash + Debug,
{
    let mut lists = HashMap::<(V0, V0), Set<(V1, V1)>>::new();
    let mut worklist = HashSet::<(V0, V0, V0)>::new();

    let mut set = Set::<(V1, V1)>::new();
    for u in g1.vertices() {
        for v in g1.vertices() {
            set.insert((u.clone(), v.clone()));
        }
    }

    for u in g0.vertices() {
        for v in g0.vertices() {
            if u == v {
                let mut s = Set::<(V1, V1)>::new();
                for u in g1.vertices() {
                    s.insert((u.clone(), u.clone()));
                }
                lists.insert((u.clone(), v.clone()), s);
            } else if g0.has_edge(u, v) {
                let s = g1.edges().collect::<Set<_>>();
                lists.insert((u.clone(), v.clone()), s);
            } else {
                lists.insert((u.clone(), v.clone()), set.clone());
            }
            for w in g0.vertices() {
                worklist.insert((u.clone(), w.clone(), v.clone()));
            }
        }
    }
    while !worklist.is_empty() {
        let (x, y, z) = worklist.iter().cloned().next().unwrap();
        worklist.remove(&(x.clone(), y.clone(), z.clone()));
        if path_reduce(&x, &y, &z, &mut lists) {
            // list of x,y changed, was the empty list derived?
            if lists.get(&(x.clone(), y.clone())).unwrap().is_empty() {
                return false;
            } else {
                for u in g0.vertices() {
                    if *u != x && *u != y {
                        worklist.insert((u.clone(), x.clone(), y.clone()));
                        worklist.insert((u.clone(), y.clone(), x.clone()));
                    }
                }
            }
        }
    }
    true
}

// Implementation of the path-reduce operation from pc2.
// Returns true, if the domain of x,y was reduced, false otherwise.
fn path_reduce<V0, V1>(x: &V0, y: &V0, z: &V0, lists: &mut HashMap<(V0, V0), Set<(V1, V1)>>) -> bool
where
    V0: Eq + Clone + Hash + Debug,
    V1: Eq + Clone + Hash + Debug,
{
    for (a, b) in lists.get(&(x.clone(), y.clone())).unwrap().clone().iter() {
        if !is_possible(x, y, z, a, b, lists) {
            lists
                .get_mut(&(x.clone(), y.clone()))
                .unwrap()
                .remove(&(a.clone(), b.clone()));
            return true;
        }
    }
    false
}

// Implemented as separate function so that we can return early
fn is_possible<V0, V1>(
    x: &V0,
    y: &V0,
    z: &V0,
    a: &V1,
    b: &V1,
    lists: &mut HashMap<(V0, V0), Set<(V1, V1)>>,
) -> bool
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
{
    for (u, v) in lists.get(&(x.clone(), z.clone())).unwrap().iter() {
        if a == u {
            for (c, d) in lists.get(&(y.clone(), z.clone())).unwrap().iter() {
                if c == b && d == v {
                    return true;
                }
            }
        }
    }
    false
}

/// Implementation of the SAC-Opt algorithm due to Bessiere and Debruyne 2008,
/// specialized to operate on graphs.
///
/// f represents a list of vertices for each vertex of g0. If there's no list
/// specified for a vertex v, a list of all nodes of g1 is assigned to v.
///
/// Returns None, if an empty domain is derived for some vertex v, otherwise
/// singleton-arc-consistent domains are returned.
pub fn sac_opt_precolour<V0, V1>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
    domains: Domains<V0, V1>,
) -> Option<Domains<V0, V1>>
where
    V0: Eq + Clone + Hash + Debug,
    V1: Eq + Clone + Hash + Debug,
{
    let mut res = match ac3_precolour(g0, g1, domains) {
        Some(v) => v,
        None => return None,
    };

    let mut pending_list = HashSet::<(V0, V1)>::new();
    let mut ds = HashMap::<(V0, V1), Domains<V0, V1>>::new();
    let mut q = HashMap::<(V0, V1), Set<(V0, V1)>>::new();

    // Init phase
    for (i, v) in res.clone() {
        for a in v.iter() {
            let mut set = Set::new();
            set.insert(a.clone());

            let mut dom = res.clone();
            dom.insert(i.clone(), set);
            ds.insert((i.clone(), a.clone()), dom);

            let mut set = Set::<(V0, V1)>::new();
            for b in v.iter() {
                if b != a {
                    set.insert((i.clone(), b.clone()));
                }
            }
            q.insert((i.clone(), a.clone()), set);
            pending_list.insert((i.clone(), a.clone()));
        }
    }

    // Propag phase
    while let Some((i, a)) = pending_list.clone().iter().next() {
        pending_list.remove(&(i.clone(), a.clone()));
        println!("pending_list.size() = {:?}", pending_list.len());
        let d = ds.get_mut(&(i.clone(), a.clone())).unwrap();
        for (x, y) in q.get(&(i.clone(), a.clone())).unwrap().iter() {
            d.get_domain_mut(&x).unwrap().remove(y);
        }
        if let Some(v) = ac3_precolour(g0, g1, d.clone()) {
            q.get_mut(&(i.clone(), a.clone())).unwrap().clear();
            *d = v;
        } else {
            res.get_domain_mut(&i).unwrap().remove(&a);
            if res.get_domain(&i).unwrap().is_empty() {
                return None;
            }
            for ((j, b), m) in ds.iter_mut() {
                if m.get_domain_mut(&i).unwrap().remove(&a) {
                    q.get_mut(&(j.clone(), b.clone()))
                        .unwrap()
                        .insert((i.clone(), a.clone()));
                    pending_list.insert((j.clone(), b.clone()));
                }
            }
        }
    }

    Some(res)
}

pub fn sac_opt<V0, V1>(g0: &AdjacencyList<V0>, g1: &AdjacencyList<V1>) -> Option<Domains<V0, V1>>
where
    V0: Eq + Clone + Hash + Debug,
    V1: Eq + Clone + Hash + Debug,
{
    sac_opt_precolour(g0, g1, Domains::from_lists(g0, g1))
}

#[derive(Clone, Debug)]
pub struct Domains<V0: Eq + Hash, V1: Eq> {
    domains: HashMap<V0, Set<V1>>,
}

impl<V0: Eq + Hash, V1: Eq> Domains<V0, V1> {
    pub fn new() -> Domains<V0, V1> {
        Domains {
            domains: HashMap::<V0, Set<V1>>::new(),
        }
    }

    pub fn insert(&mut self, v: V0, d: Set<V1>) -> Option<Set<V1>> {
        self.domains.insert(v, d)
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (&V0, &Set<V1>)> + 'a {
        self.domains.iter()
    }

    pub fn domains(&self) -> impl Iterator<Item = &Set<V1>> {
        self.domains.values()
    }

    pub fn variables(&self) -> impl Iterator<Item = &V0> {
        self.domains.keys()
    }

    pub fn get_domain(&self, v: &V0) -> Option<&Set<V1>> {
        self.domains.get(v)
    }

    pub fn get_domain_mut(&mut self, v: &V0) -> Option<&mut Set<V1>> {
        self.domains.get_mut(v)
    }

    pub fn remove(&mut self, v: &V0, w: &V1) -> bool {
        self.domains.get_mut(&v).unwrap().remove(w)
    }

    pub fn contains_variable(&self, v: &V0) -> bool {
        self.domains.contains_key(v)
    }
}

// and we'll implement IntoIterator
impl<V0: Eq + Hash, V1: Eq> IntoIterator for Domains<V0, V1> {
    type Item = (V0, Set<V1>);
    type IntoIter = std::collections::hash_map::IntoIter<V0, Set<V1>>;

    fn into_iter(self) -> Self::IntoIter {
        self.domains.into_iter()
    }
}

impl<V0: Eq + Hash + Clone, V1: Eq + Clone + Hash> Domains<V0, V1> {
    pub fn from_lists(g0: &AdjacencyList<V0>, g1: &AdjacencyList<V1>) -> Domains<V0, V1> {
        let mut domains = Domains::new();
        for v0 in g0.vertices() {
            domains.insert(v0.clone(), g1.vertices().cloned().collect::<Set<_>>());
        }
        domains
    }
}
