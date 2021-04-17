use std::fmt::Debug;
use std::{collections::HashMap, collections::HashSet, hash::Hash};

use crate::tripolys::adjacency_list::{AdjacencyList, Set};

type Domains<V0, V1> = HashMap<V0, Set<V1>>;

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
    mut f: Domains<V0, V1>,
) -> Option<Domains<V0, V1>>
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
                    if f.get(&u0).unwrap().is_empty() {
                        return None;
                    }
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
                    if f.get(&v0).unwrap().is_empty() {
                        return None;
                    }
                    changed = true;
                }
            }
        }
    }

    Some(f)
}

/// A modification of `ac1_precolour` that is initialized with a list of all nodes
/// of g1 for each node in g0.
pub fn ac1<V0, V1>(g0: &AdjacencyList<V0>, g1: &AdjacencyList<V1>) -> Option<Domains<V0, V1>>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
{
    ac1_precolour(g0, g1, Domains::new())
}

/// Implementation of the AC-3 algorithm by Mackworth 1977, specialized to find
/// graph homomorphisms.
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
            // domain of x changed, was the empty list derived?
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

/// A modification of `ac3_precolour` that is initialized with a list of all nodes
/// of g1 for each node in g0.
pub fn ac3<V0, V1>(g0: &AdjacencyList<V0>, g1: &AdjacencyList<V1>) -> Option<Domains<V0, V1>>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
{
    ac3_precolour(g0, g1, Domains::new())
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

pub fn sac1_precolour<V0, V1>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
    mut f: Domains<V0, V1>,
) -> Option<Domains<V0, V1>>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
{
    for v0 in g0.vertex_iter() {
        if !f.contains_key(&v0) {
            f.insert(v0.clone(), g1.vertex_iter().cloned().collect::<Set<_>>());
        }
    }

    let mut e = match ac3_precolour(g0, g1, f) {
        Some(v) => v,
        None => return None,
    };

    let mut changed = true;
    while changed {
        changed = false;

        let e2 = e.clone();
        for (k, v) in e.iter_mut() {
            for u in v.clone().iter() {
                let mut set = Set::new();
                set.insert(u.clone());

                let mut map = e2.clone();
                map.insert(k.clone(), set);

                if let None = ac3_precolour(g0, g1, map) {
                    v.remove(&u);
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
    sac1_precolour(g0, g1, Domains::new())
}

fn ac_init<V0, V1>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
    f: &mut HashMap<V0, Set<V1>>,
    m: &mut HashMap<V0, HashSet<V1>>,
    counter: &mut HashMap<(V0, V0), HashMap<V1, u32>>,
    s_ac: &mut HashMap<(V0, V1), Vec<(V0, V1)>>,
    list_ac: &mut Vec<(V0, V1)>,
) where
    V0: Eq + Hash + Clone,
    V1: Eq + Hash + Clone,
{
    let edges = g0.edge_vec();

    for (u0, v0) in edges.iter() {
        for u1 in f.get(&u0).unwrap().clone().iter() {
            let mut total = 0;
            for v1 in f.get(&v0).unwrap().iter() {
                if g1.contains_edge(u1, v1) {
                    total += 1;
                    if let Some(entry) = s_ac.get_mut(&(v0.clone(), v1.clone())) {
                        entry.push((u0.clone(), u1.clone()));
                    } else {
                        s_ac.insert((v0.clone(), v1.clone()), vec![(u0.clone(), u1.clone())]);
                    }
                }
            }
            if total == 0 {
                f.get_mut(&u0).unwrap().remove(&u1);
                if let Some(entry) = m.get_mut(&u0) {
                    entry.insert(u1.clone());
                } else {
                    let mut set = HashSet::<V1>::new();
                    set.insert(u1.clone());
                    m.insert(u0.clone(), set);
                }
                list_ac.push((u0.clone(), u1.clone()));
            } else {
                if let Some(entry) = counter.get_mut(&(u0.clone(), v0.clone())) {
                    entry.insert(u1.clone(), total);
                } else {
                    let mut map = HashMap::<V1, u32>::new();
                    map.insert(u1.clone(), total);
                    counter.insert((u0.clone(), v0.clone()), map);
                }
            }
        }
    }
}

fn ac_prune<V0, V1>(
    f: &mut HashMap<V0, Set<V1>>,
    m: &mut HashMap<V0, HashSet<V1>>,
    counter: &mut HashMap<(V0, V0), HashMap<V1, u32>>,
    s_ac: &HashMap<(V0, V1), Vec<(V0, V1)>>,
    list_ac: &mut Vec<(V0, V1)>,
    s_sac: &mut HashMap<(V0, V1), Vec<(V0, V1)>>,
    list_sac: &mut Vec<(V0, V1)>,
) where
    V0: Eq + Hash + Clone,
    V1: Eq + Hash + Clone,
{
    while !list_ac.is_empty() {
        let (j, b) = list_ac.pop().unwrap();

        if let Some(entry) = s_ac.get(&(j.clone(), b)) {
            for (i, a) in entry.iter() {
                let count = counter
                    .get_mut(&(i.clone(), j.clone()))
                    .unwrap()
                    .get_mut(&a)
                    .unwrap();
                *count -= 1;

                if *count == 0 && !m.get(&i).unwrap().contains(&a) {
                    f.get_mut(&i).unwrap().remove(a);

                    m.get_mut(&i).unwrap().insert(a.clone());

                    list_ac.push((i.clone(), a.clone()));

                    if let Some(entry) = s_sac.get(&(i.clone(), a.clone())) {
                        for (k, c) in entry.iter() {
                            list_sac.push((k.clone(), c.clone()));
                        }
                    }
                }
            }
        }
    }
}

fn sac_init<V0, V1>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
    f: &mut HashMap<V0, Set<V1>>,
    m: &mut HashMap<V0, HashSet<V1>>,
    counter: &mut HashMap<(V0, V0), HashMap<V1, u32>>,
    s_ac: &HashMap<(V0, V1), Vec<(V0, V1)>>,
    s_sac: &mut HashMap<(V0, V1), Vec<(V0, V1)>>,
    list_sac: &mut Vec<(V0, V1)>,
) where
    V0: Eq + Hash + Clone,
    V1: Eq + Hash + Clone,
{
    for i in g0.vertex_iter() {
        for a in f.get(&i).unwrap().clone().iter() {
            let mut set = Set::new();
            set.insert(a.clone());

            let mut d = f.clone();
            d.insert(i.clone(), set);

            if let Some(_) = ac3_precolour(g0, g1, d.clone()) {
                for (j, l) in d.iter() {
                    for b in l.iter() {
                        // let vec = s_sac.get_mut(&(j.clone(), b.clone())).unwrap();
                        // vec.push((i.clone(), a.clone()));
                        if let Some(entry) = s_sac.get_mut(&(j.clone(), b.clone())) {
                            entry.push((i.clone(), a.clone()));
                        } else {
                            s_sac.insert((j.clone(), b.clone()), vec![(i.clone(), a.clone())]);
                        }
                    }
                }
            } else {
                f.get_mut(&i).unwrap().remove(a);

                if let Some(entry) = m.get_mut(&i) {
                    entry.insert(a.clone());
                } else {
                    let mut entry = HashSet::<V1>::new();
                    entry.insert(a.clone());
                    m.insert(i.clone(), entry);
                }

                let mut list = vec![(i.clone(), a.clone())];
                ac_prune(f, m, counter, s_ac, &mut list, s_sac, list_sac);

                if let Some(entry) = s_sac.get(&(i.clone(), a.clone())) {
                    for (k, c) in entry.iter() {
                        list_sac.push((k.clone(), c.clone()));
                    }
                }
            }
        }
    }
}

fn sac_prune<V0, V1>(
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
    V0: Eq + Hash + Clone,
    V1: Eq + Hash + Clone,
{
    while !list_sac.is_empty() {
        let (u0, u1) = list_sac.pop().unwrap();
        if f.get(&u0).unwrap().contains(&u1) {
            let mut set = Set::new();
            set.insert(u1.clone());

            let mut f2 = f.clone();
            f2.insert(u0.clone(), set);

            if let None = ac3_precolour(g0, g1, f.clone()) {
                f.get_mut(&u0).unwrap().remove(&u1);
                m.get_mut(&u0).unwrap().insert(u1.clone());
                ac_prune(f, m, counter, s_ac, list_ac, s_sac, list_sac);
                for elem in s_sac.get(&(u0, u1)).unwrap().iter() {
                    list_sac.push(elem.clone());
                }
            }
        }
    }
}

pub fn sac2_precolour<V0, V1>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
    mut f: HashMap<V0, Set<V1>>,
) -> Option<HashMap<V0, Set<V1>>>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
{
    let mut m = HashMap::<V0, HashSet<V1>>::new();
    let mut counter = HashMap::<(V0, V0), HashMap<V1, u32>>::new();

    let mut s_ac = HashMap::<(V0, V1), Vec<(V0, V1)>>::new();
    let mut list_ac = Vec::<(V0, V1)>::new();
    let mut s_sac = HashMap::<(V0, V1), Vec<(V0, V1)>>::new();
    let mut list_sac = Vec::<(V0, V1)>::new();

    for v0 in g0.vertex_iter() {
        if !f.contains_key(&v0) {
            f.insert(v0.clone(), g1.vertex_iter().cloned().collect::<Set<_>>());
        }
        m.insert(v0.clone(), HashSet::<V1>::new());
    }

    ac_init(
        g0,
        g1,
        &mut f,
        &mut m,
        &mut counter,
        &mut s_ac,
        &mut list_ac,
    );
    ac_prune(
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
        &mut f,
        &mut m,
        &mut counter,
        &s_ac,
        &mut s_sac,
        &mut list_sac,
    );
    sac_prune(
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
    Some(f)
}

/// A modification of `sac2_precolour` that is initialized with a list of all nodes
/// of g1 for each node in g0.
pub fn sac2<V0, V1>(g0: &AdjacencyList<V0>, g1: &AdjacencyList<V1>) -> Option<HashMap<V0, Set<V1>>>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
{
    sac2_precolour(g0, g1, HashMap::new())
}

/// Performs a depth-first-search to find a mapping from `g0` to `g1` that is
/// locally consistent. The type of local consistency is determined by the
/// algorithm `algo`.
pub fn dfs_precolour<V0, V1, A>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
    f: HashMap<V0, Set<V1>>,
    algo: &A,
) -> Option<HashMap<V0, V1>>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
    A: LocalConsistency<V0, V1>,
{
    let e = match ac3_precolour(g0, g1, f) {
        Some(v) => v,
        None => return None,
    };
    let vec = e.clone().into_iter().collect::<Vec<_>>();

    if let Some(map) = dfs_rec(g0, g1, e, vec.into_iter(), algo) {
        Some(
            map.iter()
                .map(|(k, v)| (k.clone(), v.iter().cloned().next().unwrap()))
                .collect(),
        )
    } else {
        return None;
    }
}

/// Recursive helper function for `dfs`.
fn dfs_rec<V0, V1, I, A>(
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
        *map.get_mut(&u).unwrap() = set;

        if ac3_precolour(g0, g1, map.clone()).is_some() {
            return dfs_rec(g0, g1, map, iter, ac);
        }
    }
    return None;
}

pub fn dfs<V0, V1, A>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
    algo: &A,
) -> Option<HashMap<V0, V1>>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
    A: LocalConsistency<V0, V1>,
{
    dfs_precolour(g0, g1, Domains::new(), algo)
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

    if let Some(_) = dfs_sac_backtrack_rec(g0, g1, f, vec.into_iter(), &mut backtracked) {
        if backtracked {
            return None;
        } else {
            return Some(HashMap::<_, _>::new());
        }
    } else {
        return Some(HashMap::<_, _>::new());
    }
}

fn dfs_sac_backtrack_rec<V0, V1, I>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
    f: Domains<V0, V1>,
    mut iter: I,
    backtracked: &mut bool,
) -> Option<Domains<V0, V1>>
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

        if sac2_precolour(g0, g1, map.clone()).is_some() {
            return dfs_sac_backtrack_rec(g0, g1, map, iter, backtracked);
        }
    }
    *backtracked = true;
    return None;
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
    for u in g1.vertex_iter() {
        for v in g1.vertex_iter() {
            set.insert((u.clone(), v.clone()));
        }
    }

    for u in g0.vertex_iter() {
        for v in g0.vertex_iter() {
            if u == v {
                let mut s = Set::<(V1, V1)>::new();
                for u in g1.vertex_iter() {
                    s.insert((u.clone(), u.clone()));
                }
                lists.insert((u.clone(), v.clone()), s);
            } else if g0.contains_edge(u, v) {
                let s = g1.edge_vec().iter().cloned().collect::<Set<_>>();
                lists.insert((u.clone(), v.clone()), s);
            } else {
                lists.insert((u.clone(), v.clone()), set.clone());
            }
            for w in g0.vertex_iter() {
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
                for u in g0.vertex_iter() {
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
