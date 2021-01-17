use crate::arc_consistency::{ac3, ac3_precolour, AdjacencyList, Set};
use std::{
    cmp::{max, min},
    collections::{HashMap, HashSet},
    fs::{self, OpenOptions},
    io::Write,
};

/// A triad graph implemented as a wrapper struct around a `Vec<String>`.
///
/// Each String in the Vector represents a path that leaves the
/// vertex of degree 3 of the triad. `'0'` stands for
/// forward edge and `'1'` for backward edge.
///
/// Note that we don't restrict the triad to have exactly three arms.
/// Instead there must be at most three arms, and every triad that has less
/// can be considered a "partial triad".
#[derive(Debug, Clone)]
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
}

pub fn rooted_core_arms(max_length: u32) -> Vec<Vec<String>> {
    let mut arm_list = vec![vec![String::from("")]];
    let mut arms = vec![String::from("")];

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
            for arm in arms.iter() {
                arm_list_len.push(format!("{}{}", '0', arm.clone()));
                arm_list_len.push(format!("{}{}", '1', arm.clone()));
            }

            arm_list_len = arm_list_len
                .iter()
                .cloned()
                .filter(|arm| {
                    let mut triad = Triad::new();
                    triad.add_arm(arm);

                    if triad.is_rooted_core() {
                        println!("Adding {:?} to armlist!", triad);
                        if let Err(e) = writeln!(file, "{}", arm) {
                            eprintln!("Couldn't write to file: {}", e);
                        }
                        return true;
                    }
                    false
                })
                .collect();
        } else {
            // TODO
        };
        arms = arm_list_len.clone();
        arm_list.push(arm_list_len)
    }
    arm_list
}

/// Returns a list of core triads up to a maximal arm length.
pub fn cores_max_length(max_length: u32) -> Vec<Triad> {
    let arm_list = rooted_core_arms(max_length);

    println!("armlist done!");
    // Cached pairs of RCAs that cannot form a core triad
    let mut cache = Cache::new();
    for len in 0..=max_length {
        // TODO Rewrite with iter()
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

    println!("Cache done!");
    let mut triadlist = Vec::<Triad>::new();

    for len in 0..=max_length {
        // TODO Single node as edge case? len = 0
        println!("Iterating len = {}", len);
        let path = format!("data/cores{}", len);

        if let Ok(file) = fs::read(&path) {
            println!("Read cores with len = {} from file", len);
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
                for j in 0..=i {
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

pub fn cores_nodes(num_nodes: u32) -> Vec<Triad> {
    let mut arm_list = rooted_core_arms(num_nodes - 4);

    println!("armlist done!");

    // Cached pairs of RCAs that cannot form a core triad
    let mut cache = Cache::new();
    for len in 0..arm_list.len() {
        for i in 0..=min(len, num_nodes as usize - len - 1) {
            // TODO ugly ^
            for (a, arm1) in arm_list[len as usize].iter().enumerate() {
                for (b, arm2) in arm_list[i as usize].iter().enumerate() {
                    let mut t = Triad::new();
                    t.add_arm(arm1);
                    t.add_arm(arm2);
                    if !t.is_rooted_core() {
                        cache.insert(((len as u32, a), (i as u32, b)));
                        println!("Inserting {} {}", len, i);
                    }
                }
            }
        }
    }

    println!("Cache done!");

    let mut triadlist = Vec::<Triad>::new();

    for num in 1..=num_nodes {
        let path = format!("data/cores_nodes{}", num);

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
            println!("Initializing triadlist!");
            for (i, j, k) in triplets(num).iter() {
                for (a, arm1) in arm_list[*i as usize].iter().enumerate() {
                    for (b, arm2) in arm_list[*j as usize].iter().enumerate() {
                        for (c, arm3) in arm_list[*k as usize].iter().enumerate() {
                            if cache.cached((*i, a), (*j, b), (*k, c)) {
                                continue;
                            } else {
                                let triad = Triad::from(arm1, arm2, arm3);
                                if triad.is_core() {
                                    println!("Added {:?} to triadlist", &triad);
                                    triadlist.push(triad);
                                    if let Err(e) = writeln!(file, "{},{},{}", arm1, arm2, arm3) {
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

    // fn initialize() TODO

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

pub fn triplets(num_nodes: u32) -> Vec<(u32, u32, u32)> {
    let mut vec = Vec::<(u32, u32, u32)>::new();
    if num_nodes < 8 {
        return vec;
    }

    let num = ((num_nodes - 1) as f32 / 3.0).ceil() as u32;

    for i in num..num_nodes - 3 {
        for j in 1..=min(num_nodes - i - 1, i) {
            let k = num_nodes - i - j - 1;
            if k <= j && k != 0 {
                vec.push((i, j, num_nodes - i - j - 1));
            }
        }
    }
    println!("Triplets: {:?}", vec); // TODO remove
    vec
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
