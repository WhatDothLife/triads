use std::{
    collections::{HashMap, HashSet},
    ops::Add,
};

use crate::arc_consistency::{ac3, ac3_precolor, AdjacencyList, Set};

#[derive(Debug)]
pub struct Triad(Vec<String>);

impl Triad {
    fn new() -> Triad {
        Triad(Vec::<String>::new())
    }

    pub fn from_str(a: &str) -> Triad {
        Triad(vec![a.to_string()])
    }

    pub fn from_strs(a: &str, b: &str, c: &str) -> Triad {
        Triad(vec![a.to_string(), b.to_string(), c.to_string()])
    }

    fn merge(t1: &Triad, t2: &Triad) -> Triad {
        let mut vec = Vec::<String>::new();
        vec.append(&mut t1.0.clone());
        vec.append(&mut t2.0.clone());
        Triad(vec)
    }

    // Adds an arm to the triad
    fn push_arm(&mut self, arm: String) {
        if self.0.len() == 3 {
            panic!("Triad already has 3 arms!");
        } else {
            self.0.push(arm);
        }
    }

    fn is_rcp(&self) -> bool {
        let list = self.adjacency_list();
        let map = ac3_precolor_root(&list).unwrap();
        for (k, v) in map {
            if v.size() != 1 {
                return false;
            }
        }
        true
    }

    fn is_core(&self) -> bool {
        let list = self.adjacency_list();
        let map = ac3(&list, &list).unwrap();
        for (_, v) in map {
            if v.size() != 1 {
                return false;
            }
        }
        true
    }

    pub fn adjacency_list(&self) -> AdjacencyList<u32> {
        let mut list = AdjacencyList::<u32>::new();
        let mut node_id: u32 = 1;

        list.insert_vertex(0);

        for i in self.0.iter() {
            for (j, v) in i.chars().enumerate() {
                list.insert_vertex(node_id);

                if j == 0 {
                    if v == '1' {
                        list.insert_edge(&node_id, &0);
                    } else {
                        list.insert_edge(&0, &node_id);
                    }
                } else {
                    if v == '1' {
                        list.insert_edge(&node_id, &(node_id - 1));
                    } else {
                        list.insert_edge(&(node_id - 1), &node_id);
                    }
                }
                node_id += 1;
            }
        }
        list
    }
}

// impl Add for Triad {
//     type Output = Triad;

//     fn add(self, other: Triad) -> Triad {
//         let vec = self.0.append(&mut (other.0.clone()));
//         Triad(vec)
//     }
// }

// A variant of precoloured AC, that restricts
// the domain of vertex 0 to itself
fn ac3_precolor_root(g: &AdjacencyList<u32>) -> Option<HashMap<u32, Set<u32>>> {
    let mut set = Set::<u32>::new();
    set.insert(0);

    let mut pre_color = HashMap::<u32, Set<u32>>::new();
    pre_color.insert(0, set);

    ac3_precolor(g, g, pre_color)
}

// TODO Think of consistent names
pub fn cores(maxlength: u32) -> Vec<Triad> {
    // Find a list of RCPs
    let mut pathlist = Vec::<Triad>::new();

    for len in 1..=maxlength {
        let arms = arms(len);

        for arm in arms.iter() {
            let triad = Triad::from_str(arm);
            if triad.is_rcp() {
                println!("Pushing {:?} on the pathlist.", &triad);
                pathlist.push(triad);
            }
        }
    }
    println!("Calculated pathlist!");

    // Assemble the RCPs to core triads
    let mut triadlist = Vec::<Triad>::new();
    // Cached pairs of RCPs that cannot form a core triad
    let mut cache = Cache::new();

    let pairs = pairs(pathlist.len() as u32);
    for (a, b) in pairs.iter() {
        let ref t1 = &pathlist[*a as usize];
        let ref t2 = &pathlist[*b as usize];
        let t = Triad::merge(&t1, &t2);
        if !t.is_rcp() {
            cache.insert((*a, *b));
        }
    }

    let triplets = triplets(pathlist.len() as u32);

    for (a, b, c) in triplets.iter() {
        if cache.cached((*a, *b, *c)) {
            continue;
        } else {
            let ref t1 = &pathlist[*a as usize];
            let ref t2 = &pathlist[*b as usize];
            let ref t3 = &pathlist[*c as usize];
            let triad = Triad::merge(&Triad::merge(t1, t2), t3);
            if triad.is_core() {
                println!("Pushed {:?} onto the triadlist!", &triad);
                triadlist.push(triad);
            }
        }
    }

    // triadlist = triadlist.into_iter().filter(|t| t.is_core()).collect();

    triadlist
}

struct Cache(HashSet<(u32, u32)>);

impl Cache {
    fn new() -> Cache {
        Cache(HashSet::<(u32, u32)>::new())
    }

    fn insert(&mut self, value: (u32, u32)) {
        self.0.insert(value);
    }

    fn cached(&self, t: (u32, u32, u32)) -> bool {
        if self.0.contains(&(t.0, t.1)) {
            return true;
        } else if self.0.contains(&(t.0, t.2)) {
            return true;
        } else if self.0.contains(&(t.1, t.2)) {
            return true;
        }
        false
    }
}

// Returns all sorted triplets of all numbers smaller than n
// without duplicate elements e.g. let n = 3 then
// only [(0, 1, 2)] is returned
pub fn triplets(n: u32) -> Vec<(u32, u32, u32)> {
    assert!(n > 2, "n must be greater than 2!");
    let mut triplets = Vec::<(u32, u32, u32)>::new();
    for a in 0..(n - 2) {
        for b in (a + 1)..(n - 1) {
            for c in (b + 1)..n {
                triplets.push((a, b, c));
            }
        }
    }
    triplets
}

// Merge with function above
fn pairs(n: u32) -> Vec<(u32, u32)> {
    assert!(n > 1, "n must be greater than 1!");
    let mut pairs = Vec::<(u32, u32)>::new();
    for a in 0..(n - 1) {
        for b in (a + 1)..(n) {
            pairs.push((a, b));
        }
    }
    pairs
}

// Returns all arms of length len
pub fn arms(len: u32) -> Vec<String> {
    let mut vec = Vec::<String>::new();
    vec.push("".to_string());
    arms_h(vec, len)
}

// Lazy recursive implementation by cloning the n-1 strings
// and appending 0 and 1 to each of them
fn arms_h(a: Vec<String>, mut len: u32) -> Vec<String> {
    if len == 0 {
        return a;
    }
    let mut b = Vec::<String>::new();
    for elem in a.iter() {
        let mut el = elem.clone();
        el.push('0');
        b.push(el);

        el = elem.clone();
        el.push('1');
        b.push(el);
    }
    len -= 1;
    return arms_h(b, len);
}
