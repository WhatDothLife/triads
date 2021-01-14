use crate::arc_consistency::{ac3, ac3_precolour, AdjacencyList, Set};
use std::{
    collections::{HashMap, HashSet},
    fs::{self, OpenOptions},
    io::Write,
};

use serde::{Deserialize, Serialize};

/// A triad graph implemented as a wrapper struct around a `Vec<String>`.
///
/// Each String in the Vector represents a path that leaves the
/// vertex of degree 3 of the triad. `'0'` stands for
/// forward edge and `'1'` for backward edge.
///
/// Note that we don't restrict the triad to have exactly three arms.
/// Instead there must be at most three arms, and every triad that has less
/// can be considered a "partial triad".
#[derive(Serialize, Deserialize, Debug, Clone)]
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
    pub fn from(a: &str, b: &str, c: &str) -> Triad {
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
    // fn merge(&self, other: &Triad) -> Triad {
    //     if self.0.len() + other.0.len() > 3 {
    //         panic!("Triad cannot have more than 3 arms!");
    //     }
    //     let mut vec = Vec::<String>::new();
    //     vec.append(&mut self.0.clone());
    //     vec.append(&mut other.0.clone());
    //     Triad(vec)
    // }
}

enum Cores {
    NODES(u32),
    LENGTH(u32),
}

/// Returns a list of core triads up to a maximal arm length.
pub fn cores(max_length: u32) -> Vec<Triad> {
    // Find a list of RCAs
    let mut arm_list = Vec::<Vec<String>>::new();

    for len in 1..=max_length {
        let path = format!("data/arms{}", len);
        let mut arm_list_len = Vec::new();

        if let Ok(file) = fs::read(&path) {
            let arms: Vec<String> = String::from_utf8_lossy(&file)
                .split_terminator('\n')
                .map(|x| x.to_string())
                .collect();
            arm_list_len = arms;
        } else if let Ok(mut file) = OpenOptions::new().append(true).create(true).open(&path) {
            let arms_len = arms(len);

            for arm in arms_len.iter() {
                let mut triad = Triad::new();
                triad.add_arm(&arm.clone());

                if triad.is_rooted_core() {
                    println!("Added {:?} to armlist", &triad);
                    arm_list_len.push(arm.clone());

                    if let Err(e) = writeln!(file, "{}", arm) {
                        eprintln!("Couldn't write to file: {}", e);
                    }
                }
            }
        } else {
            eprintln!("Unable to open file: {}", path);
        };
        arm_list.push(arm_list_len)
    }

    // Cached pairs of RCAs that cannot form a core triad
    let mut cache = Cache::new();
    for len in 0..max_length {
        for i in 0..=len {
            for (a, arm1) in arm_list[len as usize].iter().enumerate() {
                for (b, arm2) in arm_list[i as usize].iter().enumerate() {
                    let mut t = Triad::new();
                    t.add_arm(arm1);
                    t.add_arm(arm2);
                    if !t.is_rooted_core() {
                        cache.insert(((len, a), (i, b)));
                    }
                }
            }
        }
    }

    let mut triadlist = Vec::<Triad>::new();

    println!("{:?}", &arm_list);

    for len in 0..max_length {
        let path = format!("data/cores{}", len + 1);

        if let Ok(file) = fs::read(&path) {
            let triads: Vec<String> = String::from_utf8_lossy(&file)
                .split_terminator('\n')
                .map(|x| x.to_owned())
                .collect();
            for triad in triads {
                triadlist.push(Triad(
                    triad.split(',').map(|x| x.to_owned()).collect::<Vec<_>>(),
                ));
            }
        } else if let Ok(mut file) = OpenOptions::new().append(true).create(true).open(&path) {
            for i in 0..=len {
                for j in 0..=len {
                    for (a, arm1) in arm_list[len as usize].iter().enumerate() {
                        for (b, arm2) in arm_list[i as usize].iter().enumerate() {
                            for (c, arm3) in arm_list[j as usize].iter().enumerate() {
                                if cache.cached((len, a), (i, b), (j, c)) {
                                    continue;
                                } else {
                                    let triad = Triad::from(arm1, arm2, arm3);
                                    if triad.is_core() {
                                        println!("Added {:?} to triadlist", &triad);
                                        triadlist.push(triad);
                                        if let Err(e) = writeln!(file, "{},{},{}", arm1, arm2, arm3)
                                        {
                                            eprintln!("Could not write to file: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    triadlist
}

// A variant of precoloured AC, that restricts
// the domain of vertex 0 to itself.
fn ac3_precolor_0(g: &AdjacencyList<u32>) -> Option<HashMap<u32, Set<u32>>> {
    let mut set = Set::<u32>::new();
    set.insert(0);

    let mut pre_color = HashMap::<u32, Set<u32>>::new();
    pre_color.insert(0, set);

    ac3_precolour(g, g, pre_color)
}

// Cache to store pairs of RCAs that cannot form a core triad
struct Cache(HashSet<((u32, usize), (u32, usize))>);

impl Cache {
    fn new() -> Cache {
        Cache(HashSet::<((u32, usize), (u32, usize))>::new())
    }

    fn insert(&mut self, value: ((u32, usize), (u32, usize))) {
        self.0.insert(value);
    }

    fn cached(&self, a: (u32, usize), b: (u32, usize), c: (u32, usize)) -> bool {
        if self.0.contains(&(a, b)) {
            return true;
        } else if self.0.contains(&(a, c)) {
            return true;
        } else if self.0.contains(&(b, c)) {
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

fn rooted_core_arms(len: u32) -> Vec<Triad> {
    let mut arm_list = Vec::<Triad>::new(); // TODO use map filter here
    let arms = arms(len);

    for arm in arms.iter() {
        let mut triad = Triad::new();
        triad.add_arm(arm);
        if triad.is_rooted_core() {
            println!("Added {:?} to armlist", &triad);
            arm_list.push(triad);
        }
    }
    arm_list
}
