//! A homomorphism from H^k to H.
use std::{
    collections::HashMap,
    fmt::{self, Debug},
    hash::Hash,
};

use crate::{
    adjacency_list::AdjacencyList,
    consistency::{List, Lists},
};
use crate::{
    consistency::{ac3_precolour, find_precolour, LocalConsistency},
    list,
};

use super::{
    consistency::{sac1_precolour, search_precolour},
    triad::{level, Triad},
};

type Identity = fn(arity: &Arity, num: u32) -> Vec<Vec<Vec<u32>>>;

/// Returns a set of sets of vertices that should be contracted when searching
/// for wnu identity of arity `arity` of a graph with `num` nodes.
pub fn wnu(arity: &Arity, num: u32) -> Vec<Vec<Vec<u32>>> {
    let mut vec = Vec::<Vec<Vec<u32>>>::new();
    for i in 0..num {
        let mut v = Vec::<Vec<u32>>::new();
        match arity {
            Arity::Single(k) => {
                v.append(&mut wnu_i(*k, i, num));
            }
            Arity::Dual(k, l) => {
                v.append(&mut wnu_i(*k, i, num));
                v.append(&mut wnu_i(*l, i, num));
            }
        }
        vec.push(v);
    }
    vec
}

fn wnu_i(arity: u32, i: u32, num: u32) -> Vec<Vec<u32>> {
    let mut v = vec![vec![i; arity as usize]];

    for j in 0..num {
        if i == j {
            continue;
        }
        for k in 0..arity {
            let mut vec = vec![i; arity as usize];
            vec[k as usize] = j;
            v.push(vec);
        }
    }
    v
}

/// Returns a set of sets of vertices that should be contracted when searching
/// for commutative identity of arity `arity` of a graph with `num` nodes.
pub fn commutative(_: &Arity, num: u32) -> Vec<Vec<Vec<u32>>> {
    let mut vec = Vec::<Vec<Vec<u32>>>::new();
    for i in 0..num {
        for j in i + 1..num {
            vec.push(vec![vec![i, j], vec![j, i]]);
        }
    }
    vec
}

/// Returns a set of sets of vertices that should be contracted when searching
/// for siggers identity of arity `arity` of a graph with `num` nodes.
pub fn siggers(_: &Arity, num: u32) -> Vec<Vec<Vec<u32>>> {
    let mut vec = Vec::<Vec<Vec<u32>>>::new();
    for i in 0..num {
        for j in 0..num {
            for k in 0..num {
                if !(i == j && j == k) {
                    if j == k {
                        vec.push(vec![vec![i, j, k, i], vec![j, i, j, k], vec![i, k, i, j]]);
                    } else if i != k {
                        vec.push(vec![vec![i, j, k, i], vec![j, i, j, k]]);
                    }
                }
            }
        }
    }
    vec
}

/// TODO f(x,...,x,y) = f(x,...,x,y,x) = ... = f(y,x,...,x)
pub fn wnu_p<T: Eq + Clone + Hash + Debug>(a: &[T], b: &[T]) -> bool {
    assert!(a.len() >= 2 && b.len() >= 2, "length must be at least 2!");
    let elem_a = wnu_elem(a);
    let elem_b = wnu_elem(b);
    match (elem_a, elem_b) {
        (WNU::Unique(x1, y1), WNU::Unique(x2, y2)) => x1 == x2 && y1 == y2,
        (WNU::Even(x), WNU::Unique(_, z)) => x == z,
        (WNU::Unique(_, y), WNU::Even(z)) => y == z,
        (WNU::None, _) => false,
        (_, WNU::None) => false,
        _ => false,
    }
}

fn wnu_elem<T: Eq + Clone + Hash + Debug>(x: &[T]) -> WNU<T> {
    // (elem, frequency of elem)
    let elem_freq = x.iter().fold(HashMap::<T, usize>::new(), |mut m, y| {
        *m.entry(y.clone()).or_default() += 1;
        m
    });

    let len = elem_freq.len();
    if len == 1 {
        return WNU::Even(elem_freq.keys().cloned().next().unwrap());
    }
    if len == 2 {
        let vec = elem_freq.into_iter().collect::<Vec<_>>();
        let (e0, f0) = vec[0].clone();
        let (e1, f1) = vec[1].clone();
        if f0 == 1 {
            return WNU::Unique(e0, e1);
        };
        if f1 == 1 {
            return WNU::Unique(e1, e0);
        }
    }
    WNU::None
}

/// f(r,a,r,e) = f(a,r,e,a)
#[allow(dead_code)]
fn siggers_p<T: Eq>(v0: &[T], v1: &[T]) -> bool {
    assert!(v0.len() == 4 && v1.len() == 4, "length must be equal to 4!");
    let r = v0[1] == v1[0] && v0[1] == v1[2];
    let a = v0[0] == v0[3] && v0[0] == v1[1];
    let e = v0[2] == v1[3];
    r && a && e
}

/// f(x,y) = f(y,x)
#[allow(dead_code)]
fn commutative_p<T: Eq>(a: &[T], b: &[T]) -> bool {
    assert!(a.len() == 2 && b.len() == 2, "length must be equal to 2!");
    a[0] == b[1] && a[1] == b[0]
}

/// f(x,x,y) = f(x,y,x) = f(y,x,x) = x
#[allow(dead_code)]
fn majority_p<T: Eq + Clone>(a: &[T], b: &[T]) -> bool {
    assert!(a.len() == 3 && b.len() == 3, "length must be equal to 3!");
    let v = major_elem(a);
    let w = major_elem(b);
    v.clone().and(w).map(|x| x == v.unwrap()).unwrap_or(false)
}

/// Returns an element if it occurs more often than all others, None otherwise.
fn major_elem<T: Eq + Clone>(x: &[T]) -> Option<T> {
    if x[0] == x[1] {
        Some(x[0].clone())
    } else if x[1] == x[2] {
        Some(x[1].clone())
    } else if x[2] == x[0] {
        Some(x[2].clone())
    } else {
        None
    }
}

enum WNU<T: Eq + Clone + Hash> {
    Unique(T, T),
    Even(T),
    None,
}

/// A polymorphism implemented as a wrapper struct around a `HashMap<Vec<U>, U>`.
///
/// TODO
#[derive(Debug)]
pub struct Polymorphism<U>
where
    U: Clone + Eq + Hash,
{
    map: HashMap<Vec<U>, U>,
}

impl<T> fmt::Display for Polymorphism<T>
where
    T: Hash + Clone + Eq + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        for (k, v) in self.map.iter() {
            s.push_str(format!("{:?} -> {:?}\n", k, v).as_str());
        }
        write!(f, "{}", s)
    }
}

/// Used to create a representation of a polymorphism finder. Polymorphism
/// settings are set using the "builder pattern" with the
/// [`PolymorphismFinder::find`] method being the terminal method that starts a
/// depth-first-search using a given local consistency algorithm as a heuristic.
///
/// **NOTE:** The mandatory "option" that one must set is `arity`. The "other"
/// may also appear in any order (so long as the [`PolymorphismFinder::find`]
/// method is the last method called).
///
/// # Example
///
/// ```no_run
/// let graph: AdjacencyList<u32> = Triad::from("10,10,0").into();
/// let res = PolymorphismFinder::new(Arity::Single(2))
///     .identity(commutative)
///     .conservative(true)
///     .find(graph, &ac3_precolour);
/// assert!(res.is_some());
/// ```
/// [`PolymorphismFinder::find`]: ./struct.PolymorphismFinder.html#method.find
#[allow(missing_debug_implementations)]
pub struct PolymorphismFinder {
    arity: Arity,
    identity: Option<Identity>,
    conservative: bool,
    idempotent: bool,
    majority: bool,
    linear: bool,
}

impl PolymorphismFinder {
    /// Constructs a new PolymorphismFinder for a polymorphism with arity
    /// `arity`.
    pub fn new(arity: Arity) -> PolymorphismFinder {
        PolymorphismFinder {
            arity,
            identity: None,
            conservative: false,
            idempotent: false,
            majority: false,
            linear: false,
        }
    }

    /// The identity the polymorphism should satisfy.
    pub fn identity(mut self, indentity: Identity) -> Self {
        self.identity = Some(indentity);
        self
    }

    /// Whether the polymorphism should be conservative.
    pub fn conservative(mut self, c: bool) -> Self {
        self.conservative = c;
        self
    }

    /// Whether the polymorphism should be idempotent.
    pub fn idempotent(mut self, i: bool) -> Self {
        self.idempotent = i;
        self
    }

    /// Whether the polymorphism should be a majority operation.
    pub fn majority(mut self, m: bool) -> Self {
        self.majority = m;
        self
    }

    /// Whether the algorithm used for finding the polymorphism solves the CSP.
    pub fn linear(mut self, l: bool) -> Self {
        self.linear = l;
        self
    }

    /// Tries to find the configured polymorphism for graph `g` by using
    /// algorithm `algorithm` as a heuristic. Returns None, if there is no such
    /// polymorphism, otherwise the polymorphism is returned.
    pub fn find<A>(&self, g: &AdjacencyList<u32>, algorithm: &A) -> Option<Polymorphism<u32>>
    where
        A: LocalConsistency<Vec<u32>, u32>,
    {
        let mut product = match self.arity {
            Arity::Single(k) => g.power(k),
            Arity::Dual(k, l) => g.power(k).union(&g.power(l)),
        };

        let mut domains = Lists::<Vec<u32>, u32>::new();

        if let Some(p) = self.identity {
            let vecs = p(&self.arity, g.vertices().count() as u32);
            for vec in vecs {
                for i in 1..vec.len() {
                    product.contract_vertices(&vec[0], &vec[i]);
                }
                if self.majority {
                    domains.insert(vec[0].clone(), list![vec[0][0]]);
                }
            }
        }

        if self.conservative {
            for vec in product.vertices() {
                domains.insert(vec.clone(), vec.iter().cloned().collect::<List<_>>());
            }
        }

        if self.idempotent {
            for vec in product.vertices() {
                if is_all_same(&vec) {
                    domains.insert(vec.clone(), list![vec[0]]);
                }
            }
        }

        if self.linear {
            if let Some(map) = find_precolour(&product, g, domains, algorithm) {
                return Some(Polymorphism {
                    map: map
                        .iter()
                        .map(|(k, v)| (k.clone(), v.iter().cloned().next().unwrap()))
                        .collect(),
                });
            } else {
                None
            }
        } else if let Some(map) = search_precolour(&product, g, domains, algorithm) {
            Some(Polymorphism { map })
        } else {
            None
        }
    }
}

impl PolymorphismFinder {
    /// Optimization for commutative polymorphisms
    /// TODO refactor me
    pub fn find_commutative<A>(&self, triad: &Triad, algorithm: &A) -> Option<Polymorphism<u32>>
    where
        A: LocalConsistency<Vec<u32>, u32>,
    {
        let g: AdjacencyList<u32> = triad.into();
        let mut product = AdjacencyList::<Vec<u32>>::new();
        if let Arity::Single(k) = self.arity {
            product = g.power(k);
        }
        if let Some(p) = self.identity {
            // product.contract_if(p);
            let vecs = p(&self.arity, g.vertices().count() as u32);
            for vec in vecs {
                for i in 1..vec.len() {
                    product.contract_vertices(&vec[0], &vec[i]);
                }
            }
        }

        // Only consider consider the component with vertices (u, v) where u and
        // v are on the same level.
        let mut indicator = AdjacencyList::<Vec<u32>>::new();
        for comp in product.components() {
            let v = comp.vertices().next().unwrap();
            if level(v[0], triad) == level(v[1], triad) {
                indicator = indicator.union(&comp);
            }
        }

        let mut domains = Lists::<Vec<u32>, u32>::new();

        if self.conservative {
            for vec in indicator.vertices() {
                domains.insert(vec.clone(), vec.iter().cloned().collect::<List<_>>());
            }
        }

        if self.idempotent {
            for vec in indicator.vertices() {
                if is_all_same(&vec) {
                    domains.insert(vec.clone(), list![vec[0]]);
                }
            }
        }

        if let Some(map) = search_precolour(&indicator, &g, domains, algorithm) {
            Some(Polymorphism { map })
        } else {
            None
        }
    }
}

fn is_all_same<T: PartialEq>(arr: &[T]) -> bool {
    arr.windows(2).all(|w| w[0] == w[1])
}

/// The arity of a graph identity.
#[derive(Debug)]
pub enum Arity {
    /// The usual case.
    Single(u32),
    /// Needed for e.g. 3-4 weak near unamity polymorphisms.
    Dual(u32, u32),
}

/// The registered polymorphisms.
#[derive(Debug)]
pub enum PolymorphismKind {
    /// (2-ary) commutative polymorphism
    Commutative,
    /// (3-ary) majority polymorphism
    Majority,
    /// (4-ary) siggers polymorphism
    Siggers,
    /// 3-4 weak near unamity polymorphism
    WNU34,
    /// 3 weak near unamity polymorphism
    WNU3,
}

impl fmt::Display for PolymorphismKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PolymorphismKind::Commutative => write!(f, "commutative"),
            PolymorphismKind::Majority => write!(f, "majority"),
            PolymorphismKind::Siggers => write!(f, "siggers"),
            PolymorphismKind::WNU34 => write!(f, "3/4 wnu"),
            PolymorphismKind::WNU3 => write!(f, "3 wnu"),
        }
    }
}

/// Returns None, if `list` does not have a polymorphism of kind `kind`,
/// otherwise a polymorphism of `list` is returned.
pub fn find_polymorphism(triad: &Triad, kind: &PolymorphismKind) -> Option<Polymorphism<u32>> {
    match kind {
        PolymorphismKind::Commutative => PolymorphismFinder::new(Arity::Single(2))
            .identity(commutative)
            .linear(true)
            .find(&triad.into(), &ac3_precolour),

        PolymorphismKind::Majority => PolymorphismFinder::new(Arity::Single(3))
            .identity(wnu)
            .majority(true)
            .linear(true)
            .find(&triad.into(), &sac1_precolour),

        PolymorphismKind::Siggers => PolymorphismFinder::new(Arity::Single(4))
            .identity(siggers)
            .linear(true)
            .find(&triad.into(), &ac3_precolour),

        PolymorphismKind::WNU34 => PolymorphismFinder::new(Arity::Dual(3, 4))
            .identity(wnu)
            .linear(true)
            .find(&triad.into(), &ac3_precolour),

        PolymorphismKind::WNU3 => PolymorphismFinder::new(Arity::Single(3))
            .identity(wnu)
            .find(&triad.into(), &ac3_precolour),
    }
}
