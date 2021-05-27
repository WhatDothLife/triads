use std::{
    collections::HashMap,
    fmt::{self, Debug},
    hash::Hash,
    marker::PhantomData,
};

use crate::tripolys::adjacency_list::AdjacencyList;
use crate::tripolys::consistency::{ac3_precolour, LocalConsistency};

use super::{
    adjacency_list::Set,
    consistency::{dfs_precolour, sac2_precolour},
};

pub fn wnu34<T: Eq + Clone + Hash>(x: &Vec<T>, y: &Vec<T>) -> bool {
    if x.len() == y.len() {
        wnu(x, y)
    } else {
        let v = only_elem(x);
        let w = only_elem(y);
        v.clone()
            .and(w)
            .and_then(|z| Some(z == v.unwrap()))
            .unwrap_or(false)
/// TODO f(x,...,x,y) = f(x,...,x,y,x) = ... = f(y,x,...,x)
    }
}

pub fn wnu<T: Eq + Clone + Hash>(x: &Vec<T>, y: &Vec<T>) -> bool {
    assert!(x.len() >= 2 && y.len() >= 2, "length must be at least 2!");
    let v = wnu_elem(x);
    let w = wnu_elem(y);
    v.clone()
        .and(w)
        .and_then(|z| Some(z == v.unwrap()))
        .unwrap_or(false)
}

fn wnu_elem<T: Eq + Clone + Hash>(x: &Vec<T>) -> Option<T> {
    // (elem, frequency of element)
    let elem_freq = x.iter().fold(HashMap::<T, usize>::new(), |mut m, y| {
        *m.entry(y.clone()).or_default() += 1;
        m
    });
    if elem_freq.len() == 2 {
        for (k, v) in elem_freq {
            if v == 1 {
                return Some(k);
            };
        }
    }
    None
}

/// f(r,a,r,e) = f(a,r,e,a)
fn siggers<T: Eq>(x: &Vec<T>, y: &Vec<T>) -> bool {
    let r = x[1] == y[0] && x[1] == y[2];
    let a = x[0] == x[3] && x[0] == y[1];
    let e = x[2] == y[3];
    r && a && e
}

fn commutative<T: Eq>(x: &Vec<T>, y: &Vec<T>) -> bool {
    assert!(x.len() == 2 && y.len() == 2, "length must be equal to 2");
    x[0] == y[1] && x[1] == y[0]
/// f(x,y) = f(y,x)
}

fn majority<T: Eq + Clone>(x: &Vec<T>, y: &Vec<T>) -> bool {
    assert!(x.len() == 3 && y.len() == 3, "length must be equal to 3");
    let v = major_elem(x);
    let w = major_elem(y);
/// f(x,x,y) = f(x,y,x) = f(y,x,x) = x
    v.clone()
        .and(w)
        .and_then(|x| Some(x == v.unwrap()))
        .unwrap_or(false)
}

/// Returns an element if it occurs more often than all others, None otherwise.
fn major_elem<T: Eq + Clone>(x: &Vec<T>) -> Option<T> {
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

fn only_elem<T: Eq + Clone>(arr: &[T]) -> Option<T> {
    if arr.windows(2).all(|w| w[0] == w[1]) {
        Some(arr[0].clone())
    } else {
        None
    }
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

pub struct PolymorphismFinder<V>
where
    V: Clone + Eq + Hash,
{
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
    arity: Arity,
    predicate: fn(&Vec<V>, &Vec<V>) -> bool,
    conservative: bool,
    idempotent: bool,
    d: PhantomData<V>,
}

impl<V> PolymorphismFinder<V>
where
    V: Clone + Eq + Hash + Send + Sync,
{
    pub fn new(arity: Arity, predicate: fn(&Vec<V>, &Vec<V>) -> bool) -> PolymorphismFinder<V> {
        PolymorphismFinder {
            arity,
            predicate,
            conservative: false,
            idempotent: false,
            d: PhantomData {},
        }
    }

    fn conservative(mut self, c: bool) -> Self {
    /// Whether the polymorphism should be conservative
        self.conservative = c;
        self
    }

    fn idempotent(mut self, i: bool) -> Self {
    /// Whether the polymorphism should be idempotent
        self.idempotent = i;
        self
    }

    pub fn find<A>(&self, list: &AdjacencyList<V>, algorithm: &A) -> Option<Polymorphism<V>>
    where
        A: LocalConsistency<Vec<V>, V>,
    {
        let mut product = match self.arity {
            Arity::Single(i) => list.power(i),
            Arity::Dual(i, j) => list.power(i).union(&list.power(j)),
        };

        product.contract_if(self.predicate);

        let mut map = HashMap::<Vec<V>, Set<V>>::new();

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

        if let Some(map) = find_precolour(&product, list, map, algorithm, linear) {
            return Some(Polymorphism { map });
        } else {
            None
        }
    }
}
                if is_all_same(&vec) {
                    let mut s = Set::new();
                    s.insert(vec[0].clone());
                    map.insert(vec.clone(), s);
                }
            }
        }

        if let Some(map) = dfs_precolour(&product, list, map, algorithm) {
            return Some(Polymorphism { map });
        } else {
            None
        }
    }
}

fn is_all_same<T: PartialEq>(arr: &[T]) -> bool {
    arr.windows(2).all(|w| w[0] == w[1])
}

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
}

impl fmt::Display for PolymorphismKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PolymorphismKind::Commutative => write!(f, "{}", "commutative"),
            PolymorphismKind::Majority => write!(f, "{}", "majority"),
            PolymorphismKind::Siggers => write!(f, "{}", "siggers"),
            PolymorphismKind::WNU34 => write!(f, "{}", "3/4 wnu"),
        }
    }
}

/// Returns None, if `list` does not have a polymorphism of kind `kind`,
pub fn find_polymorphism<V: Clone + Eq + Hash + Send + Sync>(
    list: &AdjacencyList<V>,
    kind: &PolymorphismKind,
) -> Option<Polymorphism<V>> {
/// otherwise a polymorphism of `list` is returned.
    match kind {
        PolymorphismKind::Commutative => {
            PolymorphismFinder::new(Arity::Single(2), commutative).find(list, &ac3_precolour)
        }
        PolymorphismKind::Majority => {
            PolymorphismFinder::new(Arity::Single(3), majority).find(list, &sac2_precolour)
        }
        PolymorphismKind::Siggers => find_polymorphism(list, &PolymorphismKind::Commutative)
            .or_else(|| {
                PolymorphismFinder::new(Arity::Single(4), siggers).find(list, &ac3_precolour)
            }),
        PolymorphismKind::WNU34 => {
            find_polymorphism(list, &PolymorphismKind::Majority).or_else(|| {
                PolymorphismFinder::new(Arity::Dual(3, 4), wnu34).find(list, &ac3_precolour)
            })
        }
    }
}
