use std::{
    collections::HashMap,
    fmt::{self, Debug},
    hash::Hash,
};

use crate::{
    adjacency_list::AdjacencyList,
    consistency::{ac3_precolour, dfs, dfs_sac_backtrack, LocalConsistency},
    errors::OptionsError,
};

pub fn siggers<T: Eq>(x: &Vec<T>, y: &Vec<T>) -> bool {
    let r = x[1] == y[0] && x[1] == y[2];
    let a = x[0] == x[3] && x[0] == y[1];
    let e = x[2] == y[3];
    r && a && e
}

pub fn commutative<T: Eq>(x: &Vec<T>, y: &Vec<T>) -> bool {
    assert!(x.len() == 2 && y.len() == 2, "Vertex without length 2");
    x[0] == y[1] && x[1] == y[0]
}

pub fn majority<T: Eq + Clone>(x: &Vec<T>, y: &Vec<T>) -> bool {
    let v = major(x);
    let w = major(y);
    v.and(w.clone())
        .and_then(|x| Some(x == w.unwrap()))
        .unwrap_or(false)
}

fn major<T: Eq + Clone>(x: &Vec<T>) -> Option<T> {
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

#[derive(Debug)]
pub struct Polymorphism<U>
where
    U: Clone + Eq + Hash,
{
    map: HashMap<Vec<U>, U>,
}

impl<T> Polymorphism<T>
where
    T: Clone + Eq + Hash + Sync + Send + Debug,
{
    fn find<A: LocalConsistency<Vec<T>, T>, P: Fn(&Vec<T>, &Vec<T>) -> bool>(
        list: &AdjacencyList<T>,
        arity: u32,
        predicate: &P,
        algorithm: A,
    ) -> Option<Polymorphism<T>> {
        let mut product: AdjacencyList<Vec<T>> = list.power(arity);
        product.contract_if(predicate);

        if let Some(map) = dfs(&product, list, algorithm) {
            return Some(Polymorphism { map });
        } else {
            None
        }
    }

    fn find_sac_backtrack<P: Fn(&Vec<T>, &Vec<T>) -> bool>(
        list: &AdjacencyList<T>,
        arity: u32,
        predicate: &P,
    ) -> Option<Polymorphism<T>> {
        let mut product: AdjacencyList<Vec<T>> = list.power(arity);
        product.contract_if(predicate);

        if let Some(map) = dfs_sac_backtrack(&product, list) {
            return Some(Polymorphism { map });
        } else {
            None
        }
    }
}

impl<T> fmt::Display for Polymorphism<T>
where
    T: Hash + Clone + Eq + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str = String::new();
        for (k, v) in self.map.iter() {
            str.push_str(format!("{:?} -> {:?}\n", k, v).as_str());
        }
        write!(f, "{}", str)
    }
}

fn binary_commutative<T: Eq + Hash + Clone + Send + Sync + Debug>(
    list: &AdjacencyList<T>,
) -> Option<Polymorphism<T>> {
    Polymorphism::<T>::find(list, 2, &commutative, ac3_precolour)
}

fn binary_commutative_backtrack<T: Eq + Hash + Clone + Send + Sync + Debug>(
    list: &AdjacencyList<T>,
) -> Option<Polymorphism<T>> {
    Polymorphism::<T>::find_sac_backtrack(list, 2, &commutative)
}

fn ternary_majority<T: Eq + Hash + Clone + Send + Sync + Debug>(
    list: &AdjacencyList<T>,
) -> Option<Polymorphism<T>> {
    Polymorphism::<T>::find(list, 3, &majority, ac3_precolour)
}

fn quaternary_siggers<T: Eq + Hash + Clone + Send + Sync + Debug>(
    list: &AdjacencyList<T>,
) -> Option<Polymorphism<T>> {
    if let Some(m) = Polymorphism::<T>::find(list, 2, &commutative, ac3_precolour) {
        Some(m)
    } else {
        Polymorphism::<T>::find(list, 4, &siggers, ac3_precolour)
    }
}

pub struct PolymorphismFinder;

impl PolymorphismFinder {
    pub fn get<T: Clone + Eq + Hash + Sync + Send + Debug + 'static>(
        polymorphism: &str,
    ) -> Result<Box<dyn Fn(&AdjacencyList<T>) -> Option<Polymorphism<T>> + Sync>, OptionsError>
    {
        match polymorphism {
            "commutative" => Ok(Box::new(binary_commutative)),
            "commutative_backtrack" => Ok(Box::new(binary_commutative_backtrack)),
            "majority" => Ok(Box::new(ternary_majority)),
            "siggers" => Ok(Box::new(quaternary_siggers)),
            &_ => Err(OptionsError::PolymorphismNotFound),
        }
    }
}
