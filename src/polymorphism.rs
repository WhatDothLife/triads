use std::{collections::HashMap, hash::Hash};

use crate::arc_consistency::{ac3_pruning_search, AdjacencyList, Set};

pub fn commutative<T: Eq>(x: &(T, T), y: &(T, T)) -> bool {
    x.0 == y.1 && x.1 == y.0
}

pub fn siggers<T: Eq>(x: &(T, T, T, T), y: &(T, T, T, T)) -> bool {
    let r = x.1 == y.0 && x.1 == y.2;
    let a = x.0 == x.3 && x.0 == y.1;
    let e = x.2 == y.3;
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

pub fn polymorphism<T: Eq + Hash + Copy>(polymorphism: &str) -> impl Fn(AdjacencyList<T>) -> bool {
    match polymorphism {
        "commutative" => |list: AdjacencyList<T>| {
            let mut product: AdjacencyList<(T, T)> = &list * &list;
            product.contract_if(&commutative);
            ac3_pruning_search(&product, &list).is_some()
        },

        "siggers" => |list: AdjacencyList<T>| {
            let mut product: AdjacencyList<(T, T, T, T)> = list.power_4();
            product.contract_if(&siggers);
            ac3_pruning_search(&product, &list).is_some()
        },
        &_ => |_: AdjacencyList<T>| false, // TODO Fix me
    }
}
