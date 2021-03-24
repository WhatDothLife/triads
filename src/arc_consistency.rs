use std::{collections::HashMap, collections::HashSet, hash::Hash};

use crate::adjacency_list::{AdjacencyList, Set};

type Domains<V0, V1> = HashMap<V0, Set<V1>>;

pub trait ArcConsistency<V0: Eq + Clone + Hash, V1: Eq + Clone + Hash>:
    Fn(&AdjacencyList<V0>, &AdjacencyList<V1>, Domains<V0, V1>) -> Option<Domains<V0, V1>>
{
}

impl<V0: Eq + Clone + Hash, V1: Eq + Clone + Hash, F> ArcConsistency<V0, V1> for F where
    F: Fn(&AdjacencyList<V0>, &AdjacencyList<V1>, Domains<V0, V1>) -> Option<Domains<V0, V1>>
{
}

/// Implementation of the AC-1 algorithm by TODO, specialized to find
/// graph homomorphisms.
///
/// f represents a list of vertices for each vertex of g0. If there's no list
/// specified for a vertex v, a list of all nodes of g1 is assigned to v.
///
/// Returns None, if an empty domain is derived for some vertex v, otherwise
/// arc-consistent domains are returned.
pub fn ac1<V0, V1>(g0: &AdjacencyList<V0>, g1: &AdjacencyList<V1>) -> Domains<V0, V1>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
{
    // list of vertices from g1 for each g0
    let mut f = Domains::new();

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

/// Implementation of the AC-3 algorithm by Mackworth 1977, specialized to find
/// graph homomorphisms.
///
/// f represents a list of vertices for each vertex of g0. If there's no list
/// specified for a vertex v, a list of all nodes of g1 is assigned to v.
///
/// Returns None, if an empty domain is derived for some vertex v, otherwise an
/// arc-consistent map is returned.
pub fn ac_3_precolour<V0, V1>(
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
pub fn ac3<V0, V1>(g0: &AdjacencyList<V0>, g1: &AdjacencyList<V1>) -> Option<Domains<V0, V1>>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
{
    ac_3_precolour(g0, g1, Domains::new())
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

// TODO add Hashmap parameter
pub fn dfs_ac<V0, V1, A>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
    ac: A,
) -> Option<HashMap<V0, V1>>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
    A: ArcConsistency<V0, V1>,
{
    let f = match ac3(g0, g1) {
        Some(v) => v,
        None => return None,
    };
    let vec = f.clone().into_iter().collect::<Vec<_>>();

    if let Some(map) = dfs_ac_rec(g0, g1, f, vec.into_iter(), ac) {
        Some(
            map.iter()
                .map(|(k, v)| (k.clone(), v.iter().cloned().next().unwrap()))
                .collect(),
        )
    } else {
        return None;
    }
}

fn dfs_ac_rec<V0, V1, I, A>(
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
    A: ArcConsistency<V0, V1>,
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

        if ac_3_precolour(g0, g1, map.clone()).is_some() {
            return dfs_ac_rec(g0, g1, map, iter, ac);
        }
    }
    return None;
}

pub fn sac1<V0, V1>(g0: &AdjacencyList<V0>, g1: &AdjacencyList<V1>) -> Option<Domains<V0, V1>>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
{
    sac1_precolour(g0, g1, Domains::new())
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

    let mut e = match ac_3_precolour(g0, g1, f) {
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

                if let None = ac_3_precolour(g0, g1, map) {
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

pub fn singleton_search<V0, V1>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
) -> Option<Domains<V0, V1>>
where
    V0: Eq + Clone + Hash,
    V1: Eq + Clone + Hash,
{
    let mut map = match sac1(&g0, &g1) {
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
            if let Some(e) = sac1_precolour(g0, g1, f) {
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

    let mut counter = 0;
    for v in l.iter() {
        println!("Iterating {}", &counter);
        let mut set = Set::new();
        set.insert(v.clone());

        let mut map = f.clone();
        *map.get_mut(&u).unwrap() = set;

        if sac1_precolour(g0, g1, map.clone()).is_some() {
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

            if let Some(_) = ac_3_precolour(g0, g1, d) {
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
