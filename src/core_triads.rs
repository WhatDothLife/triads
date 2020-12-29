use std::collections::{HashMap, HashSet};

use crate::arc_consistency::{ac3, ac3_precolor, AdjacencyList, Set};

/// A triad graph implemented as a wrapper around a `Vec<String>`.
///
/// Each String in the Vector represents a path that leaves the
/// vertex of degree 3 of the triad. '0' stands for
/// forward edge and '1' for backward edge.
///
/// Note that we don't restrict the triad to have exactly three arms.
/// Instead there must be at most three arms, and every triad that has less
/// can be considered a "partial triad".
#[derive(Debug)]
pub struct Triad(Vec<String>);

impl Triad {
    /// Creates a new empty `Triad`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    /// ```
    /// let t = Triad::new();
    /// ```
    pub fn new() -> Triad {
        Triad(Vec::<String>::new())
    }

    /// Creates a new `Triad` from `str`s.
    ///
    /// # Examples
    ///
    /// Basic usage:
    /// ```
    /// let t = Triad::from_strs("0", "1", "00");
    /// ```
    pub fn from_strs(a: &str, b: &str, c: &str) -> Triad {
        Triad(vec![a.to_string(), b.to_string(), c.to_string()])
    }

    /// Adds an arm to the triad.
    ///
    /// # Panics
    ///
    /// Panics, if the triad already has 3 arms.
    pub fn add_arm(&mut self, arm: &str) {
        if self.0.len() == 3 {
            panic!("Triad already has 3 arms!");
        } else {
            self.0.push(arm.to_string());
        }
    }

    /// Returns `true` if this triad is a rooted core, and `false` otherwise.
    ///
    /// A rooted core is a triad for which id is the only automorphism, if you restrict
    /// vertex 0 to be mapped to itself.
    ///
    /// # Examples
    /// ```
    /// let t = Triad::new();
    /// t.add_arm("100");
    /// asserteq!(false, t.is_core());
    /// asserteq!(true, t.is_rooted_core());
    /// ```
    pub fn is_rooted_core(&self) -> bool {
        let list = self.adjacency_list();
        let map = ac3_precolor_0(&list).unwrap();
        for (_, v) in map {
            if v.size() != 1 {
                return false;
            }
        }
        true
    }

    /// Returns `true` if this triad is a core, and `false` otherwise.
    ///
    /// # Examples
    /// ```
    /// let t = Triad::from_strs("0", "11", "1000");
    /// asserteq!(true, t.is_core());
    /// ```
    pub fn is_core(&self) -> bool {
        let list = self.adjacency_list();
        let map = ac3(&list, &list).unwrap();
        for (_, v) in map {
            if v.size() != 1 {
                return false;
            }
        }
        true
    }

    /// Returns an adjacency list of the triad. The root is labeled with 0.
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

    // Merges two triads and returns a new triad
    // TODO implement by overloading Add, Caution: Fancy
    fn merge(&self, other: &Triad) -> Triad {
        if self.0.len() + other.0.len() > 3 {
            panic!("Triad cannot have more than 3 arms!");
        }
        let mut vec = Vec::<String>::new();
        vec.append(&mut self.0.clone());
        vec.append(&mut other.0.clone());
        Triad(vec)
    }
}

// A variant of precoloured AC, that restricts
// the domain of vertex 0 to itself.
fn ac3_precolor_0(g: &AdjacencyList<u32>) -> Option<HashMap<u32, Set<u32>>> {
    let mut set = Set::<u32>::new();
    set.insert(0);

    let mut pre_color = HashMap::<u32, Set<u32>>::new();
    pre_color.insert(0, set);

    ac3_precolor(g, g, pre_color)
}

// TODO Think of consistent names
pub fn cores(max_length: u32) -> Vec<Triad> {
    // Find a list of RCAs
    let mut arm_list = Vec::<Triad>::new();

    for len in 1..=max_length {
        let arms = arms(len);

        for arm in arms.iter() {
            let mut triad = Triad::new();
            triad.add_arm(arm);
            if triad.is_rooted_core() {
                println!("Pushing {:?} on the pathlist.", &triad);
                arm_list.push(triad);
            }
        }
    }

    // Assemble the RCAs to core triads
    let pairs = pairs(arm_list.len() as u32);
    // Cached pairs of RCAs that cannot form a core triad
    let mut pair_cache = Cache::new();

    for (a, b) in pairs.iter() {
        let ref t1 = &arm_list[*a as usize];
        let ref t2 = &arm_list[*b as usize];
        let t = t1.merge(&t2);
        if !t.is_rooted_core() {
            pair_cache.insert((*a, *b));
        }
    }

    let triplets = triplets(arm_list.len() as u32);
    let mut triadlist = Vec::<Triad>::new();

    for (a, b, c) in triplets.iter() {
        if pair_cache.cached((*a, *b, *c)) {
            continue;
        } else {
            let ref t1 = &arm_list[*a as usize];
            let ref t2 = &arm_list[*b as usize];
            let ref t3 = &arm_list[*c as usize];
            let triad = t1.merge(&t2).merge(&t3);
            if triad.is_core() {
                println!("Pushed {:?} onto the triadlist!", &triad);
                triadlist.push(triad);
            }
        }
    }
    triadlist
}

// Cache to store pairs of RCAs that cannot form a core triad
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
// without duplicate elements
//
// # Example
//
// ```
// let t = triplets(3);
// asserteq!(t, [(0, 1, 2)]);
// ```
fn triplets(n: u32) -> Vec<(u32, u32, u32)> {
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

// Returns all sorted pairs of all numbers smaller than n
// without duplicate elements
//
// # Example
//
// ```
// let p = pairs(3);
// asserteq!(p, [(0, 1), (0, 2), (1, 2)]);
// ```
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

// Returns all strings of '0' and '1' of length len
fn arms(len: u32) -> Vec<String> {
    let mut vec = Vec::<String>::new();
    vec.push("".to_string());
    arms_h(vec, len)
}

// Recursive helper function that clones all of the n-1 strings
// and appends 0 and 1 to each of them
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
