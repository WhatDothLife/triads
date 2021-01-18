use std::{fmt::Debug, hash::Hash};

use crate::arc_consistency::{ac3_pruning_search, AdjacencyList};

pub fn commutative<T: Eq>(x: &Vec<T>, y: &Vec<T>) -> bool {
    x[0] == y[1] && x[1] == y[0]
}

pub fn siggers<T: Eq>(x: &Vec<T>, y: &Vec<T>) -> bool {
    let r = x[1] == y[0] && x[1] == y[2];
    let a = x[0] == x[3] && x[0] == y[1];
    let e = x[2] == y[3];
    r && a && e
}

pub fn majority<T: Eq + Clone>(x: &(T, T, T), y: &(T, T, T)) -> bool {
    let v = major(x);
    let w = major(y);
    v.and(w.clone())
        .and_then(|x| Some(x == w.unwrap()))
        .unwrap_or(false)
}

fn major<T: Eq + Clone>(x: &(T, T, T)) -> Option<T> {
    if x.0 == x.1 {
        return Some(x.0.clone());
    } else if x.1 == x.2 {
        return Some(x.1.clone());
    } else if x.2 == x.0 {
        return Some(x.2.clone());
    } else {
        return None;
    }
}

pub fn polymorphism<T: Eq + Hash + Clone + Debug>(
    polymorphism: &str,
) -> impl Fn(AdjacencyList<T>) -> bool {
    match polymorphism {
        "commutative" => |list: AdjacencyList<T>| {
            println!("> Checking polymorphism...");
            let mut product: AdjacencyList<Vec<T>> = list.power(2);
            product.contract_if(&commutative);
            ac3_pruning_search(&product, &list).is_some()
        },

        "siggers" => |list: AdjacencyList<T>| {
            println!("> Checking polymorphism...");
            let mut product: AdjacencyList<Vec<T>> = list.power(2);
            product.contract_if(&commutative);
            if ac3_pruning_search(&product, &list).is_none() {
                let mut product: AdjacencyList<Vec<T>> = list.power(4);
                product.contract_if(&siggers);
                return ac3_pruning_search(&product, &list).is_some();
            }
            true
        },

        &_ => |_: AdjacencyList<T>| false, // TODO Fix me
    }
}
