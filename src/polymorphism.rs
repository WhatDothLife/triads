use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
};

use crate::arc_consistency::{ac3_pruning_search, AdjacencyList};

pub fn siggers<T: Eq>(x: &Vec<T>, y: &Vec<T>) -> bool {
    let r = x[1] == y[0] && x[1] == y[2];
    let a = x[0] == x[3] && x[0] == y[1];
    let e = x[2] == y[3];
    r && a && e
}

pub fn commutative<T: Eq>(x: &Vec<T>, y: &Vec<T>) -> bool {
    x[0] == y[1] && x[1] == y[0]
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

pub struct Polymorphism<U>
where
    U: Clone + Eq + Hash,
{
    map: HashMap<Vec<U>, U>,
}

impl<T> Polymorphism<T>
where
    T: Clone + Eq + Hash,
{
    fn get<P: Fn(&Vec<T>, &Vec<T>) -> bool>(
        list: &AdjacencyList<T>,
        arity: u32,
        predicate: &P,
    ) -> Option<Polymorphism<T>> {
        let mut product: AdjacencyList<Vec<T>> = list.power(arity);
        product.contract_if(predicate);

        if let Some(map) = ac3_pruning_search(&product, list) {
            return Some(Polymorphism { map });
        } else {
            None
        }
    }
}

impl<T> Debug for Polymorphism<T>
where
    T: Display + Debug + Clone + Eq + Hash,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str = String::new();
        for (k, v) in self.map.iter() {
            str.push_str(format!("{:?} -> {}\n", k, v).as_str());
        }
        write!(f, "{}", str)
    }
}

pub fn quaternary_siggers_polymorphism<T: Eq + Hash + Clone>(
    list: &AdjacencyList<T>,
) -> Option<Polymorphism<T>> {
    if let Some(m) = Polymorphism::<T>::get(list, 2, &commutative) {
        Some(m)
    } else {
        Polymorphism::<T>::get(list, 4, &siggers)
    }
}

pub fn binary_commutative_polymorphism<T: Eq + Hash + Clone>(
    list: &AdjacencyList<T>,
) -> Option<Polymorphism<T>> {
    Polymorphism::<T>::get(list, 2, &siggers)
}

pub struct PolymorphismRegistry;

impl PolymorphismRegistry {
    pub fn get<T: Clone + Eq + Hash + 'static>(
        polymorphism: &str,
    ) -> Option<Box<dyn Fn(&AdjacencyList<T>) -> Option<Polymorphism<T>>>> {
        match polymorphism {
            "siggers" => Some(Box::new(quaternary_siggers_polymorphism)),
            "commutative" => Some(Box::new(binary_commutative_polymorphism)),
            &_ => None,
        }
    }
}
