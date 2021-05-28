use std::{
    collections::HashMap,
    fmt::{self, Debug},
    hash::Hash,
};

use crate::tripolys::adjacency_list::AdjacencyList;
use crate::tripolys::consistency::{ac3_precolour, LocalConsistency};

use super::{
    adjacency_list::Set,
    consistency::{find_precolour, sac2_precolour},
    triad::{level, Triad},
};

pub fn wnu(arity: &Arity, num: u32) -> Vec<Vec<Vec<u32>>> {
    let mut vec = Vec::<Vec<Vec<u32>>>::new();
    for i in 0..num {
        let mut v = Vec::<Vec<u32>>::new();
        match arity {
            Arity::Single(k) => {
                v.append(&mut wnu_help(*k, i, num));
            }
            Arity::Dual(k, l) => {
                v.append(&mut wnu_help(*k, i, num));
                v.append(&mut wnu_help(*l, i, num));
            }
        }
        vec.push(v);
    }
    vec
}

fn wnu_help(arity: u32, i: u32, num: u32) -> Vec<Vec<u32>> {
    let mut v = Vec::<Vec<u32>>::new();
    v.push(vec![i; arity as usize]);

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

pub fn commutative(_: &Arity, num: u32) -> Vec<Vec<Vec<u32>>> {
    let mut vec = Vec::<Vec<Vec<u32>>>::new();
    for i in 0..num {
        for j in i + 1..num {
            vec.push(vec![vec![i, j], vec![j, i]]);
        }
    }
    vec
}

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
pub fn wnu_p<T: Eq + Clone + Hash + Debug>(a: &Vec<T>, b: &Vec<T>) -> bool {
    assert!(a.len() >= 2 && b.len() >= 2, "length must be at least 2!");
    let v = wnu_elem_p(a);
    let w = wnu_elem_p(b);
    match (v, w) {
        (WNU::Unique(x1, y1), WNU::Unique(x2, y2)) => x1 == x2 && y1 == y2,
        (WNU::Even(x), WNU::Unique(_, z)) => x == z,
        (WNU::Unique(_, y), WNU::Even(z)) => y == z,
        (WNU::None, _) => return false,
        (_, WNU::None) => return false,
        _ => return false,
    }
}

fn wnu_elem_p<T: Eq + Clone + Hash + Debug>(x: &Vec<T>) -> WNU<T> {
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
fn siggers_p<T: Eq>(x: &Vec<T>, y: &Vec<T>) -> bool {
    assert!(x.len() == 4 && y.len() == 4, "length must be equal to 4!");
    let r = x[1] == y[0] && x[1] == y[2];
    let a = x[0] == x[3] && x[0] == y[1];
    let e = x[2] == y[3];
    r && a && e
}

/// f(x,y) = f(y,x)
fn commutative_p<T: Eq>(a: &Vec<T>, b: &Vec<T>) -> bool {
    assert!(a.len() == 2 && b.len() == 2, "length must be equal to 2!");
    a[0] == b[1] && a[1] == b[0]
}

/// f(x,x,y) = f(x,y,x) = f(y,x,x) = x
fn majority_p<T: Eq + Clone>(a: &Vec<T>, b: &Vec<T>) -> bool {
    assert!(a.len() == 3 && b.len() == 3, "length must be equal to 3!");
    let v = major_elem_p(a);
    let w = major_elem_p(b);
    v.clone()
        .and(w)
        .and_then(|x| Some(x == v.unwrap()))
        .unwrap_or(false)
}

/// Returns an element if it occurs more often than all others, None otherwise.
fn major_elem_p<T: Eq + Clone>(x: &Vec<T>) -> Option<T> {
    if x[0] == x[1] {
        return Some(x[0].clone());
    } else if x[1] == x[2] {
        return Some(x[1].clone());
    } else if x[2] == x[0] {
        return Some(x[2].clone());
    } else {
        return None;
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
///     .predicate(commutative)
///     .conservative(true)
///     .find(graph, &ac3_precolour);
/// ```
/// [`PolymorphismFinder::find`]: ./struct.PolymorphismFinder.html#method.find
#[allow(missing_debug_implementations)]
pub struct PolymorphismFinder {
    arity: Arity,
    predicate: Option<fn(arity: &Arity, num: u32) -> Vec<Vec<Vec<u32>>>>,
    conservative: bool,
    idempotent: bool,
    majority: bool,
}

impl PolymorphismFinder {
    pub fn new(arity: Arity) -> PolymorphismFinder {
        PolymorphismFinder {
            arity,
            predicate: None,
            conservative: false,
            idempotent: false,
            majority: false,
        }
    }

    pub fn predicate(
        mut self,
        predicate: fn(arity: &Arity, num: u32) -> Vec<Vec<Vec<u32>>>,
    ) -> Self {
        self.predicate = Some(predicate);
        self
    }

    /// Whether the polymorphism should be conservative
    pub fn conservative(mut self, c: bool) -> Self {
        self.conservative = c;
        self
    }

    /// Whether the polymorphism should be idempotent
    pub fn idempotent(mut self, i: bool) -> Self {
        self.idempotent = i;
        self
    }

    /// Whether the polymorphism should be a majority operation
    pub fn majority(mut self, m: bool) -> Self {
        self.majority = m;
        self
    }

    pub fn find<A>(
        &self,
        list: &AdjacencyList<u32>,
        algorithm: &A,
        linear: bool,
    ) -> Option<Polymorphism<u32>>
    where
        A: LocalConsistency<Vec<u32>, u32>,
    {
        let mut product = match self.arity {
            Arity::Single(k) => list.power(k),
            Arity::Dual(k, l) => list.power(k).union(&list.power(l)),
        };

        let mut map = HashMap::<Vec<u32>, Set<u32>>::new();

        if let Some(p) = self.predicate {
            let vecs = p(
                &self.arity,
                list.vertices().collect::<Vec<_>>().len() as u32,
            );
            for vec in vecs {
                for i in 1..vec.len() {
                    println!("Contracting: {:?} <- {:?}", &vec[0], &vec[i]);
                    product.contract_vertices(&vec[0], &vec[i]);
                }
                if self.majority {
                    let mut s = Set::new();
                    s.insert(vec[0][0]);
                    println!("Inserting: {:?} -> {:?}", &vec[0], &s);
                    map.insert(vec[0].clone(), s);
                }
            }
        }

        if self.conservative {
            for vec in product.vertices() {
                map.insert(vec.clone(), vec.iter().cloned().collect::<Set<_>>());
            }
        }

        if self.idempotent {
            for vec in product.vertices() {
                if is_all_same(&vec) {
                    let mut s = Set::new();
                    s.insert(vec[0].clone());
                    map.insert(vec.clone(), s);
                }
            }
        }
        println!("DEBUG");

        if let Some(map) = find_precolour(&product, list, map, algorithm, linear) {
            return Some(Polymorphism { map });
        } else {
            None
        }
    }
}

impl PolymorphismFinder {
    pub fn find_commutative<A>(
        &self,
        triad: &Triad,
        algorithm: &A,
        linear: bool,
    ) -> Option<Polymorphism<u32>>
    where
        A: LocalConsistency<Vec<u32>, u32>,
    {
        let list: AdjacencyList<u32> = triad.into();
        let mut product = AdjacencyList::<Vec<u32>>::new();
        if let Arity::Single(k) = self.arity {
            product = list.power(k);
        }
        if let Some(p) = self.predicate {
            // product.contract_if(p);
            let vecs = p(
                &self.arity,
                list.vertices().collect::<Vec<_>>().len() as u32,
            );
            for vec in vecs {
                for i in 1..vec.len() {
                    println!("Contracting: {:?} <- {:?}", &vec[0], &vec[i]);
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

        let mut map = HashMap::<Vec<u32>, Set<u32>>::new();

        if self.conservative {
            for vec in indicator.vertices() {
                map.insert(vec.clone(), vec.iter().cloned().collect::<Set<_>>());
            }
        }

        if self.idempotent {
            for vec in indicator.vertices() {
                if is_all_same(&vec) {
                    let mut s = Set::new();
                    s.insert(vec[0].clone());
                    map.insert(vec.clone(), s);
                }
            }
        }

        if let Some(map) = find_precolour(&indicator, &list, map, algorithm, linear) {
            return Some(Polymorphism { map });
        } else {
            None
        }
    }
}

fn is_all_same<T: PartialEq>(arr: &[T]) -> bool {
    arr.windows(2).all(|w| w[0] == w[1])
}

#[derive(Debug)]
pub enum Arity {
    Single(u32),
    Dual(u32, u32),
}

#[derive(Debug)]
pub enum PolymorphismKind {
    Commutative,
    Majority,
    Siggers,
    WNU34,
    WNU3,
}

impl fmt::Display for PolymorphismKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PolymorphismKind::Commutative => write!(f, "{}", "commutative"),
            PolymorphismKind::Majority => write!(f, "{}", "majority"),
            PolymorphismKind::Siggers => write!(f, "{}", "siggers"),
            PolymorphismKind::WNU34 => write!(f, "{}", "3/4 wnu"),
            PolymorphismKind::WNU3 => write!(f, "{}", "3 wnu"),
        }
    }
}

/// Returns None, if `list` does not have a polymorphism of kind `kind`,
/// otherwise a polymorphism of `list` is returned.
pub fn find_polymorphism(triad: &Triad, kind: &PolymorphismKind) -> Option<Polymorphism<u32>> {
    match kind {
        PolymorphismKind::Commutative => PolymorphismFinder::new(Arity::Single(2))
            .predicate(commutative)
            .find_commutative(triad, &ac3_precolour, false),

        PolymorphismKind::Majority => PolymorphismFinder::new(Arity::Single(3))
            .predicate(wnu)
            .majority(true)
            .find(&triad.into(), &ac3_precolour, false),

        PolymorphismKind::Siggers => find_polymorphism(triad, &PolymorphismKind::Commutative)
            .or_else(|| {
                PolymorphismFinder::new(Arity::Single(3))
                    .predicate(wnu)
                    .find(&triad.into(), &ac3_precolour, false)
                    .or_else(|| {
                        PolymorphismFinder::new(Arity::Single(4))
                            .predicate(siggers)
                            .find(&triad.into(), &ac3_precolour, false)
                    })
            }),

        PolymorphismKind::WNU34 => {
            // find_polymorphism(triad, &PolymorphismKind::Majority).or_else(|| {
            PolymorphismFinder::new(Arity::Dual(3, 4))
                .predicate(wnu)
                .find(&triad.into(), &ac3_precolour, false)
            // })
        }

        PolymorphismKind::WNU3 => PolymorphismFinder::new(Arity::Single(3))
            .predicate(wnu)
            .find(&triad.into(), &ac3_precolour, false),
    }
}
