//! A collection of various local-consistency algorithms such as AC-3 and
//! SAC-Opt implemented to work on graphs.
use std::fmt::Debug;
use std::iter::FromIterator;
use std::time::Instant;
use std::{collections::HashMap, collections::HashSet, hash::Hash};

use crate::adjacency_list::VertexID;
use crate::adjacency_list::{AdjacencyList, Set};
use crate::metrics::Metrics;

/// Abstraction of a local consistency algorithm that takes two graphs and a
/// list and tries to make the list consistent. Returns None, if the list
/// can not be made consistent, otherwise the consistent lists is returned.
pub trait LocalConsistency<V0: Eq + Clone + Hash, V1: Eq + Clone + Hash>:
    Fn(&AdjacencyList<V0>, &AdjacencyList<V1>, Lists<V0, V1>) -> Option<Lists<V0, V1>>
{
}

impl<V0: Eq + Clone + Hash, V1: Eq + Clone + Hash, F> LocalConsistency<V0, V1> for F where
    F: Fn(&AdjacencyList<V0>, &AdjacencyList<V1>, Lists<V0, V1>) -> Option<Lists<V0, V1>>
{
}

/// Creates a List containing the arguments.
///
/// list! allows Lists to be defined with the same syntax as array expressions.
#[macro_export]
macro_rules! list {
    ($($v:expr),* $(,)?) => {
        std::iter::Iterator::collect(std::array::IntoIter::new([$($v,)*]))
    };
}

/// Implementation of the AC-1 algorithm, specialized to find graph
/// homomorphisms.
///
/// f represents a list of vertices for each vertex of g0. If there's no list
/// specified for a vertex v, a list of all nodes of g1 is assigned to v.
///
/// Returns None, if an empty list is derived for some vertex v, otherwise
/// arc-consistent lists are returned.
pub fn ac1_precolour<V0, V1>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
    mut lists: Lists<V0, V1>,
) -> Option<Lists<V0, V1>>
where
    V0: VertexID,
    V1: VertexID,
{
    for v0 in g0.vertices() {
        if !lists.contains_variable(&v0) {
            lists.insert(v0.clone(), g1.vertices().cloned().collect::<List<_>>());
        }
    }

    let mut edges = g0.edges();

    let mut changed = true;
    while changed {
        changed = false;
        for (u0, v0) in &mut edges {
            for u1 in lists.get(&u0).unwrap().clone().iter() {
                let mut is_possible = false;
                for v1 in lists.get(&v0).unwrap().iter() {
                    if g1.has_edge(u1, v1) {
                        is_possible = true;
                        break;
                    }
                }

                if !is_possible {
                    lists.get_mut(&u0).unwrap().remove(&u1);
                    if lists.get(&u0).unwrap().is_empty() {
                        return None;
                    }
                    changed = true;
                }
            }

            for v1 in lists.get(&v0).unwrap().clone().iter() {
                let mut is_possible = false;
                for u1 in lists.get(&u0).unwrap().iter() {
                    if g1.has_edge(u1, v1) {
                        is_possible = true;
                        break;
                    }
                }

                if !is_possible {
                    lists.get_mut(&v0).unwrap().remove(&v1);
                    if lists.get(&v0).unwrap().is_empty() {
                        return None;
                    }
                    changed = true;
                }
            }
        }
    }

    Some(lists)
}

/// A modification of `ac1_precolour` that is initialized with a list of all nodes
/// of g1 for each node in g0.
pub fn ac1<V0, V1>(g0: &AdjacencyList<V0>, g1: &AdjacencyList<V1>) -> Option<Lists<V0, V1>>
where
    V0: VertexID,
    V1: VertexID,
{
    ac1_precolour(g0, g1, Lists::new())
}

pub fn ac3_precolour<V0, V1>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
    f: Lists<V0, V1>,
) -> Option<Lists<V0, V1>>
where
    V0: VertexID + Debug,
    V1: VertexID + Debug,
{
    ac3_precolour_lists(g0, g1, f).map(|(a, _)| a)
}

/// Implementation of the AC-3 algorithm due to Mackworth 1977, specialized to
/// find graph homomorphisms.
///
/// f represents a list of vertices for each vertex of g0. If there's no list
/// specified for a vertex v, a list of all nodes of g1 is assigned to v.
///
/// Returns None, if an empty list is derived for some vertex v, otherwise an
/// arc-consistent map is returned.
pub fn ac3_precolour_lists<V0, V1>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
    mut f: Lists<V0, V1>,
) -> Option<(Lists<V0, V1>, Lists<V0, V1>)>
where
    V0: VertexID + Debug,
    V1: VertexID + Debug,
{
    for v0 in g0.vertices() {
        if !f.contains_variable(&v0) {
            f.insert(v0.clone(), g1.vertices().cloned().collect::<List<_>>());
        }
    }

    let edges = g0.edges();
    let mut worklist = HashSet::<(V0, V0, bool)>::new();

    for (x, y) in edges {
        worklist.insert((x.clone(), y.clone(), false));
        worklist.insert((y, x, true));
    }

    // list of worklist items for each vertex of g0
    // they're added to worklist, if the list of the respective vertex changed
    let mut items = HashMap::new();

    for v in g0.vertices() {
        items.insert(v.clone(), Vec::<(V0, V0, bool)>::new());
    }

    for (x, y, dir) in worklist.iter().cloned() {
        items.get_mut(&y).unwrap().push((x, y, dir));
    }

    let mut removed = Lists::<V0, V1>::new();

    while !worklist.is_empty() {
        let (x, y, dir) = worklist.iter().cloned().next().unwrap();
        worklist.remove(&(x.clone(), y.clone(), dir));

        if let Some(lists) = arc_reduce(x.clone(), y, dir, &mut f, &g1) {
            for (k, v) in lists {
                if removed.contains_variable(&k) {
                    removed.get_mut(&k).unwrap().merge(&v);
                } else {
                    removed.insert(k, v);
                }
            }
            // list of x changed, was the empty list derived?
            if f.get(&x).unwrap().is_empty() {
                return None;
            } else {
                for item in items.get(&x).unwrap().iter().cloned() {
                    worklist.insert(item);
                }
            }
        }
    }
    Some((f, removed))
}

/// A modification of `ac3_precolour` that is initialized with a list of all nodes
/// of g1 for each node in g0.
pub fn ac3<V0, V1>(g0: &AdjacencyList<V0>, g1: &AdjacencyList<V1>) -> Option<Lists<V0, V1>>
where
    V0: VertexID + Debug,
    V1: VertexID + Debug,
{
    ac3_precolour(g0, g1, Lists::new())
}

// Implementation of the arc-reduce operation from ac3.  Returns None, if the
// list of x was not reduced, otherwise the removed elements are returned.
fn arc_reduce<V0, V1>(
    x: V0,
    y: V0,
    dir: bool,
    f: &mut Lists<V0, V1>,
    g1: &AdjacencyList<V1>,
) -> Option<Lists<V0, V1>>
where
    V0: VertexID + Debug,
    V1: VertexID + Debug,
{
    let mut changed = false;
    let mut removed = Lists::<V0, V1>::new();
    for vx in f.get(&x).unwrap().clone().iter() {
        let mut is_possible = false;
        for vy in f.get(&y).unwrap().iter() {
            if dir {
                if g1.has_edge(vy, vx) {
                    is_possible = true;
                    break;
                }
            } else if g1.has_edge(vx, vy) {
                is_possible = true;
                break;
            }
        }

        if !is_possible {
            f.get_mut(&x).unwrap().remove(&vx);
            if removed.contains_variable(&x) {
                removed.get_mut(&x).unwrap().insert(vx.clone());
            } else {
                removed.insert(x.clone(), list![vx.clone()]);
            }
            changed = true;
        }
    }
    if changed {
        Some(removed)
    } else {
        None
    }
}

/// Implementation of the SAC-1 algorithm due to Bessiere and Debruyne 1997,
/// specialized to operate on graphs.
///
/// f represents a list of vertices for each vertex of g0. If there's no list
/// specified for a vertex v, a list of all nodes of g1 is assigned to v.
///
/// Returns None, if an empty list is derived for some vertex v, otherwise
/// singleton-arc-consistent lists are returned.
pub fn sac1_precolour<V0, V1>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
    lists: Lists<V0, V1>,
) -> Option<Lists<V0, V1>>
where
    V0: VertexID + Debug,
    V1: VertexID + Debug,
{
    let mut lists = match ac3_precolour(g0, g1, lists) {
        Some(v) => v,
        None => return None,
    };

    let mut changed = true;
    let mut iterations = 0;
    while changed {
        iterations += 1;
        changed = false;

        for (k, v) in lists.clone().iter() {
            for u in v.iter() {
                let mut list = List::new();
                list.insert(u.clone());

                let mut lists_copy = lists.clone();
                lists_copy.insert(k.clone(), list);

                if ac3_precolour(g0, g1, lists_copy).is_none() {
                    let mut v_clone = v.clone();
                    v_clone.remove(&u);
                    lists.insert(k.clone(), v_clone);
                    changed = true;
                };
            }
            if v.is_empty() {
                return None;
            }
        }
    }
    println!("{:?}", iterations);
    Some(lists)
}

/// A modification of `sac1_precolour` that is initialized with a list of all nodes
/// of g1 for each node in g0.
pub fn sac1<V0, V1>(g0: &AdjacencyList<V0>, g1: &AdjacencyList<V1>) -> Option<Lists<V0, V1>>
where
    V0: VertexID + Debug,
    V1: VertexID + Debug,
{
    sac1_precolour(g0, g1, Lists::new())
}

/// Performs a depth-first-search to find a mapping from `g0` to `g1` that is
/// locally consistent. The type of local consistency is determined by the
/// algorithm `consistency`.
pub fn search_precolour<V0, V1>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
    lists: Lists<V0, V1>,
    metrics: &mut Metrics,
) -> Option<Lists<V0, V1>>
where
    V0: VertexID + Debug,
    V1: VertexID + Debug,
{
    let ac_start = Instant::now();
    let res = ac3_precolour(g0, g1, lists);
    metrics.ac_time = ac_start.elapsed();
    let mut lists = res?;

    // Sort vertices by their respective list length
    let mut sorted_list = lists.clone().into_iter().collect::<Vec<_>>();
    sorted_list.sort_by(|(_, l0), (_, l1)| l1.size().cmp(&l0.size()));
    let mut vertex_list = sorted_list.iter().map(|(a, _)| a).collect::<Vec<_>>();

    let mut backtracked = 0;
    let mut removed = Vec::<(&V0, Lists<V0, V1>)>::new();
    let mut set = Vec::<(V0, List<V1>)>::new();

    let search_start = Instant::now();
    let mut found = true;
    while !vertex_list.is_empty() {
        let v0 = vertex_list.pop().unwrap();
        let l = lists.get_mut(v0).unwrap();

        if let Some(elem) = l.pop() {
            set.push((v0.clone(), lists.get(v0).unwrap().clone()));
            lists.insert(v0.clone(), list![elem.clone()]);

            if let Some((res, rem)) = ac3_precolour_lists(g0, g1, lists.clone()) {
                removed.push((v0, rem));
                lists = res;
            } else {
                let (a, b) = set.pop().unwrap();
                lists.insert(a, b);
                vertex_list.push(v0);
            }
        } else {
            if let Some((v, l1)) = removed.pop() {
                lists.merge(&l1);
                let (a, b) = set.pop().unwrap();
                lists.insert(a, b);
                backtracked += 1;
                vertex_list.push(v0);
                vertex_list.push(v);
            } else {
                found = false;
                break;
            }
        }
    }
    metrics.search_time = search_start.elapsed();
    metrics.backtracked = backtracked;
    if found {
        Some(lists)
    } else {
        None
    }
}

// /// Tries to find a mapping from `g0` to `g1` that is locally consistent.
// /// The type of local consistency is determined by the algorithm `algo`.
// ///
// /// **NOTE:** This function assumes that algorithm `algo` solves the CSP.
// ///
// /// Returns None, if an empty list is derived for some vertex v, otherwise
// /// singleton-arc-consistent lists are returned.
// pub fn find_precolour<V0, V1, A>(
//     g0: &AdjacencyList<V0>,
//     g1: &AdjacencyList<V1>,
//     lists: Lists<V0, V1>,
//     algo: &A,
// ) -> Option<Lists<V0, V1>>
// where
//     V0: VertexID + Debug,
//     V1: VertexID,
//     A: LocalConsistency<V0, V1>,
// {
//     println!("Start");
//     let mut lists = algo(g0, g1, lists)?;
//     println!("End");

//     for v0 in g0.vertices() {
//         println!("{:?}", v0);
//         let list_v0 = lists.get(&v0).unwrap();
//         if list_v0.size() > 1 {
//             let mut found = false;

//             for v1 in list_v0.clone().iter() {
//                 let mut lists_sac = lists.clone();
//                 lists_sac.insert(v0.clone(), list![v1.clone()]);

//                 if let Some(lists_res) = algo(g0, g1, lists_sac) {
//                     lists = lists_res;
//                     found = true;
//                     break;
//                 }
//             }
//             if !found {
//                 return None;
//             }
//         }
//     }
//     Some(lists)
// }

/// Implementation of the PC-2 algorithm by Mackworth 1977, specialized to work
/// on graphs.
///
/// Returns false, if an empty list is derived for some vertex v, true otherwise.
pub fn pc2<V0, V1>(g0: &AdjacencyList<V0>, g1: &AdjacencyList<V1>) -> bool
where
    V0: VertexID + Debug,
    V1: VertexID + Debug,
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
// Returns true, if the list of x,y was reduced, false otherwise.
fn path_reduce<V0, V1>(x: &V0, y: &V0, z: &V0, lists: &mut HashMap<(V0, V0), Set<(V1, V1)>>) -> bool
where
    V0: Eq + Clone + Hash + Debug,
    V1: Eq + Clone + Hash + Debug,
{
    for (a, b) in lists.get(&(x.clone(), y.clone())).unwrap().clone().iter() {
        'middle: for (u, v) in lists.get(&(x.clone(), z.clone())).unwrap().iter() {
            if a == u {
                for (c, d) in lists.get(&(y.clone(), z.clone())).unwrap().iter() {
                    if c == b && d == v {
                        break 'middle;
                    }
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
/// Returns None, if an empty list is derived for some vertex v, otherwise
/// singleton-arc-consistent lists are returned.
pub fn sac_opt_precolour<V0, V1>(
    g0: &AdjacencyList<V0>,
    g1: &AdjacencyList<V1>,
    lists: Lists<V0, V1>,
) -> Option<Lists<V0, V1>>
where
    V0: VertexID + Debug,
    V1: VertexID + Debug,
{
    let mut lists = match ac3_precolour(g0, g1, lists) {
        Some(v) => v,
        None => return None,
    };

    let mut pending_list = HashSet::<(V0, V1)>::new();
    let mut ds = HashMap::<(V0, V1), Lists<V0, V1>>::new();
    // TODO is Set needed here?
    let mut q = HashMap::<(V0, V1), Set<(V0, V1)>>::new();

    // Init phase
    for (i, v) in lists.clone() {
        for a in v.iter() {
            let mut dom = lists.clone();
            dom.insert(i.clone(), list![a.clone()]);
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
    // println!("pending_list.size() = {:?}", pending_list.len());
    while let Some((i, a)) = pending_list.clone().iter().next() {
        pending_list.remove(&(i.clone(), a.clone()));
        let d = ds.get_mut(&(i.clone(), a.clone())).unwrap();
        for (x, y) in q.get(&(i.clone(), a.clone())).unwrap().iter() {
            d.get_mut(&x).unwrap().remove(y);
        }
        if let Some(v) = ac3_precolour(g0, g1, d.clone()) {
            q.get_mut(&(i.clone(), a.clone())).unwrap().clear();
            *d = v;
        } else {
            lists.get_mut(&i).unwrap().remove(&a);
            if lists.get(&i).unwrap().is_empty() {
                return None;
            }
            for ((j, b), m) in ds.iter_mut() {
                if m.get_mut(&i).unwrap().remove(&a) {
                    q.get_mut(&(j.clone(), b.clone()))
                        .unwrap()
                        .insert((i.clone(), a.clone()));
                    pending_list.insert((j.clone(), b.clone()));
                }
            }
        }
    }

    Some(lists)
}

/// A modification of `sac_opt_precolour` that is initialized with a list of all nodes
/// of g1 for each node in g0.
pub fn sac_opt<V0, V1>(g0: &AdjacencyList<V0>, g1: &AdjacencyList<V1>) -> Option<Lists<V0, V1>>
where
    V0: VertexID + Debug,
    V1: VertexID + Debug,
{
    sac_opt_precolour(g0, g1, Lists::new())
}

/// A list implemented as a wrapper around `HashSet`
#[derive(Clone, Debug, Default)]
pub struct List<T: Eq + Hash> {
    list: HashSet<T>,
}

impl<T: Eq + Hash + Clone> List<T> {
    /// Creates an empty `List`.
    ///
    /// # Examples
    ///
    /// ```
    /// let list: List<i32> = List::new();
    /// ```
    pub fn new() -> List<T> {
        List {
            list: HashSet::<T>::new(),
        }
    }

    /// Adds a value to the list.
    ///
    /// If the list did not have this value present, `true` is returned.
    ///
    /// If the list did have this value present, `false` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut list = List::new();
    ///
    /// assert_eq!(list.insert(2), true);
    /// assert_eq!(list.insert(2), false);
    /// assert_eq!(list.len(), 1);
    /// ```
    pub fn insert(&mut self, v: T) -> bool {
        self.list.insert(v)
    }

    /// Returns the number of elements in the list.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut l = List::new();
    /// l.insert(1);
    /// l.insert(2);
    ///
    /// assert_eq!(l.size(), 2);
    /// ```
    pub fn size(&self) -> usize {
        self.list.len()
    }

    /// An iterator visiting all elements in arbitrary order.
    /// The iterator element type is `(&'a T)`.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut list = List::new();
    /// list.insert(1);
    /// list.insert(2);
    /// list.insert(3);
    ///
    /// for v in list.iter() {
    ///     println!("vertex: {}", v);
    /// }
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.list.iter()
    }

    /// Returns `true` if the list contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut l = List::new();
    /// assert!(l.is_empty());
    ///
    /// l.insert(1);
    /// assert!(!l.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    /// Removes a value from the list, returning `true` if the key was previously
    /// in the list, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut list = List::new();
    /// list.insert(1, "a");
    ///
    /// assert!(list.remove(&1));
    /// assert!(!list.remove(&1));
    /// ```
    pub fn remove(&mut self, v: &T) -> bool {
        self.list.remove(v)
    }

    pub fn merge(&mut self, v: &List<T>) {
        for elem in v.iter() {
            self.insert(elem.clone());
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        let elemm = self.list.iter().cloned().next();
        if let Some(elem) = elemm {
            self.list.remove(&elem);
            return Some(elem);
        } else {
            None
        }
    }
}

impl<T: Eq + Hash> FromIterator<T> for List<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        List {
            list: iter.into_iter().collect::<HashSet<_>>(),
        }
    }
}

/// A list implemented as a wrapper around `HashSet`
#[derive(Clone, Debug, Default)]
pub struct Lists<V0: Eq + Hash, V1: Eq + Hash + Clone> {
    lists: HashMap<V0, List<V1>>,
}

impl<V0: Eq + Hash + Clone, V1: Eq + Hash + Clone> Lists<V0, V1> {
    /// Creates a new, empty set of lists.
    pub fn new() -> Lists<V0, V1> {
        Lists {
            lists: HashMap::<V0, List<V1>>::new(),
        }
    }

    /// Inserts a vertex-list pair into the map.
    ///
    /// If the map did not have this vertex present, [`None`] is returned.
    ///
    /// If the map did have this vertex present, the list is updated, and the old
    /// list is returned. The vertex is not updated, though; this matters for
    /// types that can be `==` without being identical.
    pub fn insert(&mut self, v: V0, d: List<V1>) -> Option<List<V1>> {
        self.lists.insert(v, d)
    }

    /// An iterator visiting all variable-list pairs in arbitrary order.
    /// The iterator element type is `(&'a V0, &'a Set<V1>)`.
    ///
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (&V0, &List<V1>)> + 'a {
        self.lists.iter()
    }

    /// An iterator visiting all lists in arbitrary order.
    /// The iterator element type is `&'a Set<V1>`.
    ///
    pub fn lists(&self) -> impl Iterator<Item = &List<V1>> {
        self.lists.values()
    }

    pub fn variables(&self) -> impl Iterator<Item = &V0> {
        self.lists.keys()
    }

    pub fn get(&self, v: &V0) -> Option<&List<V1>> {
        self.lists.get(v)
    }

    pub fn get_mut(&mut self, v: &V0) -> Option<&mut List<V1>> {
        self.lists.get_mut(v)
    }

    pub fn remove(&mut self, v: &V0, w: &V1) -> bool {
        self.lists.get_mut(&v).unwrap().remove(w)
    }

    pub fn contains_variable(&self, v: &V0) -> bool {
        self.lists.contains_key(v)
    }

    pub fn len(&self) -> usize {
        self.lists.len()
    }

    pub fn merge(&mut self, other: &Lists<V0, V1>) {
        for (k, v) in other.iter() {
            if self.contains_variable(&k) {
                self.get_mut(&k).unwrap().merge(&v);
            } else {
                self.insert(k.clone(), v.clone());
            }
        }
    }
}

impl<V0: Eq + Hash, V1: Eq + Hash + Clone> IntoIterator for Lists<V0, V1> {
    type Item = (V0, List<V1>);
    type IntoIter = std::collections::hash_map::IntoIter<V0, List<V1>>;

    fn into_iter(self) -> Self::IntoIter {
        self.lists.into_iter()
    }
}
