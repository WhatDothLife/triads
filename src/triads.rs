use crate::arc_consistency::{ac_3, ac_3_precolour};
use crate::configuration::Globals;
use core::fmt::Debug;
use rayon::prelude::*;
use std::{
    cmp::min,
    collections::{HashMap, HashSet},
    env,
    fs::{self, OpenOptions},
    hash::Hash,
    io::Write,
    str::FromStr,
    sync::Mutex,
};

use crate::adjacency_list::{AdjacencyList, Set};

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

    /// Returns the `Triad` as a string.
    ///
    /// # Examples
    ///
    /// Basic usage:
    /// ```
    /// let t = Triad::from_strs("0", "1", "00");
    /// asserteq!("0,1,00", t.as_string());
    /// ```
    pub fn as_string(&self) -> String {
        let mut string = String::new();
        let mut i = 0;
        for arm in self.0.iter() {
            if i > 0 {
                string.push(',');
            }
            string.push_str(arm);
            i += 1;
        }
        string
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
        let map = ac_3(&list, &list).unwrap();
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

impl FromStr for Triad {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let arms: Vec<String> = s.split(',').map(|x| x.to_owned()).collect();
        let triad = Triad::from(&arms[0], &arms[1], &arms[2]); // TODO make safe by returning Err

        Ok(triad)
    }
}

pub fn rooted_core_arms(max_length: u32) -> Vec<Vec<String>> {
    let mut arm_list = vec![vec![String::new()]];
    let mut last = vec![String::new()];

    for len in 1..=max_length {
        let path = format!("{}/arms/arms{}", Globals::get().data, len);
        let mut arm_list_len = Vec::new();

        if let Ok(file) = fs::read(&path) {
            let arms: Vec<String> = String::from_utf8_lossy(&file)
                .split_terminator('\n')
                .map(|x| x.to_string())
                .collect();
            arm_list_len = arms;
        } else if let Ok(mut file) = OpenOptions::new().append(true).create(true).open(&path) {
            for arm in last.iter() {
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
        last = arm_list_len.clone();
        arm_list.push(arm_list_len)
    }
    arm_list
}

// Estimates number of cores with armlength len for Vector allocation
fn num_cores_length(len: u32) -> usize {
    (0.005 * (9 as u32).pow(len) as f32) as usize
}

pub fn cores_length(arm_list: &Vec<Vec<String>>, cache: &mut Cache, len: u32) -> Vec<Triad> {
    let path = format!("{}/length/cores_length{}", Globals::get().data, len);
    let triadlist = Mutex::new(Some(Vec::<Triad>::with_capacity(num_cores_length(len))));

    if let Ok(file) = fs::read(&path) {
        let triads: Vec<String> = String::from_utf8_lossy(&file)
            .split_terminator('\n')
            .map(|x| x.to_owned())
            .collect();
        for triad in triads {
            triadlist.lock().unwrap().as_mut().unwrap().push(Triad(
                triad.split(',').map(|x| x.to_owned()).collect::<Vec<_>>(),
            ));
        }
    } else if let Ok(file) = OpenOptions::new().append(true).create(true).open(&path) {
        let file_locked = Mutex::new(file);
        cache.populate_to_length(len, &arm_list);

        triplets_length(len).par_iter().for_each(|[i, j, k]| {
            for (a, arm1) in arm_list[*i as usize].iter().enumerate() {
                for (b, arm2) in arm_list[*j as usize].iter().enumerate() {
                    for (c, arm3) in arm_list[*k as usize].iter().enumerate() {
                        let mut count = 0;
                        for arm in [arm1, arm2, arm3].iter() {
                            if arm.chars().next().unwrap() == '1' {
                                count += 1;
                            }
                        }
                        if count > 1 {
                            continue;
                        }
                        if cache.cached((*i, a), (*j, b), (*k, c)) {
                            continue;
                        } else {
                            let triad = Triad::from(arm1, arm2, arm3);
                            if triad.is_core() {
                                triadlist.lock().unwrap().as_mut().unwrap().push(triad);
                                if let Err(e) = writeln!(
                                    file_locked.lock().unwrap(),
                                    "{},{},{}",
                                    arm1,
                                    arm2,
                                    arm3
                                ) {
                                    eprintln!("Could not write to file: {}", e);
                                }
                            }
                        }
                    }
                }
            }
        });
    } else {
        eprintln!("Could not find file: {}", &path);
    }
    let list = triadlist.lock().unwrap().take().unwrap();
    list
}

pub fn cores_nodes(arm_list: &Vec<Vec<String>>, cache: &mut Cache, num: u32) -> Vec<Triad> {
    cache.populate_to_nodes(num, &arm_list);

    let path = format!("{}/nodes/cores_nodes{}", Globals::get().data, num);
    let triadlist = Mutex::new(Some(Vec::<Triad>::new()));

    if let Ok(file) = fs::read(&path) {
        let triads: Vec<String> = String::from_utf8_lossy(&file)
            .split_terminator('\n')
            .map(|x| x.to_owned())
            .collect();
        for triad in triads {
            triadlist.lock().unwrap().as_mut().unwrap().push(Triad(
                triad.split(',').map(|x| x.to_owned()).collect::<Vec<_>>(),
            ));
        }
    } else if let Ok(file) = OpenOptions::new().append(true).create(true).open(&path) {
        let file_locked = Mutex::new(file);
        triplets_nodes(num).par_iter().for_each(|[i, j, k]| {
            for (a, arm1) in arm_list[*i as usize].iter().enumerate() {
                for (b, arm2) in arm_list[*j as usize].iter().enumerate() {
                    for (c, arm3) in arm_list[*k as usize].iter().enumerate() {
                        let mut count = 0;
                        for arm in [arm1, arm2, arm3].iter() {
                            if arm.chars().next().unwrap() == '1' {
                                count += 1;
                            }
                        }
                        if count > 1 {
                            continue;
                        }
                        if cache.cached((*i, a), (*j, b), (*k, c)) {
                            continue;
                        } else {
                            let triad = Triad::from(arm1, arm2, arm3);
                            if triad.is_core() {
                                triadlist.lock().unwrap().as_mut().unwrap().push(triad);
                                if let Err(e) = writeln!(
                                    file_locked.lock().unwrap(),
                                    "{},{},{}",
                                    arm1,
                                    arm2,
                                    arm3
                                ) {
                                    eprintln!("Could not write to file: {}", e);
                                }
                            }
                        }
                    }
                }
            }
        });
    }
    let list = triadlist.lock().unwrap().take().unwrap();
    list
}

// A variant of precoloured AC, that restricts
// the domain of vertex 0 to itself.
fn ac3_precolor_0(g: &AdjacencyList<u32>) -> Option<HashMap<u32, Set<u32>>> {
    let mut set = Set::<u32>::new();
    set.insert(0);

    let mut pre_color = HashMap::<u32, Set<u32>>::new();
    pre_color.insert(0, set);

    ac_3_precolour(g, g, pre_color)
}

// Cache to store pairs of RCAs that cannot form a core triad
pub struct Cache {
    pairs: HashSet<((u32, usize), (u32, usize))>,
    nodes: u32,
    length: u32,
}

impl Cache {
    pub fn new() -> Cache {
        Cache {
            pairs: HashSet::<((u32, usize), (u32, usize))>::new(),
            nodes: 0,
            length: 0,
        }
    }

    fn cached(&self, a: (u32, usize), b: (u32, usize), c: (u32, usize)) -> bool {
        if self.pairs.contains(&(a, b)) {
            return true;
        } else if self.pairs.contains(&(a, c)) {
            return true;
        } else if self.pairs.contains(&(b, c)) {
            return true;
        }
        false
    }

    fn populate_to_nodes(&mut self, nodes: u32, arm_list: &Vec<Vec<String>>) {
        for i in self.nodes..=nodes {
            self.populate_nodes(i, arm_list);
        }
        self.nodes = nodes;
    }

    fn populate_nodes(&mut self, num: u32, arm_list: &Vec<Vec<String>>) {
        let cache_path = format!("{}/nodes/pairs_nodes{}", Globals::get().data, num);

        if let Ok(file) = fs::read(&cache_path) {
            let pairs = String::from_utf8_lossy(&file)
                .split_terminator('\n')
                .map(|x| {
                    x.to_owned()
                        .split_terminator(',')
                        .map(|y| y.to_owned())
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();

            for pair in pairs {
                let len = pair[0].parse::<u32>().unwrap();
                let a = pair[1].parse::<usize>().unwrap();
                let i = pair[2].parse::<u32>().unwrap();
                let b = pair[3].parse::<usize>().unwrap();
                self.pairs.insert(((len, a), (i, b)));
            }
        } else if let Ok(file) = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&cache_path)
        {
            let file_locked = Mutex::new(file);
            let pairs_locked = Mutex::new(Some(Vec::<_>::new()));
            pairs_nodes(num).par_iter().for_each(|[i, j]| {
                for (a, arm1) in arm_list[*i as usize].iter().enumerate() {
                    for (b, arm2) in arm_list[*j as usize].iter().enumerate() {
                        let mut t = Triad::new();
                        t.add_arm(arm1);
                        t.add_arm(arm2);
                        // First condition excludes permutations of arms with the same length
                        if (i == j && a < b) || !t.is_rooted_core() {
                            pairs_locked
                                .lock()
                                .unwrap()
                                .as_mut()
                                .unwrap()
                                .push(((*i as u32, a), (*j as u32, b)));
                            if let Err(e) =
                                writeln!(file_locked.lock().unwrap(), "{},{},{},{}", i, a, j, b)
                            {
                                eprintln!("Could not write to file: {}", e);
                            }
                        }
                    }
                }
            });
            let pairs = pairs_locked.lock().unwrap().take().unwrap();
            pairs.iter().for_each(|&pair| {
                self.pairs.insert(pair);
            });
        }
    }

    fn populate_to_length(&mut self, len: u32, arm_list: &Vec<Vec<String>>) {
        for i in self.length..=len {
            self.populate_length(i, arm_list);
        }
        self.length = len;
    }

    fn populate_length(&mut self, len: u32, arm_list: &Vec<Vec<String>>) {
        let cache_path = format!("{}/length/pairs_length{}", Globals::get().data, len);

        if let Ok(file) = fs::read(&cache_path) {
            let pairs = String::from_utf8_lossy(&file)
                .split_terminator('\n')
                .map(|x| {
                    x.to_owned()
                        .split_terminator(',')
                        .map(|y| y.to_owned())
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();

            for pair in pairs {
                let len = pair[0].parse::<u32>().unwrap();
                let a = pair[1].parse::<usize>().unwrap();
                let i = pair[2].parse::<u32>().unwrap();
                let b = pair[3].parse::<usize>().unwrap();
                self.pairs.insert(((len, a), (i, b)));
            }
        } else if let Ok(file) = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&cache_path)
        {
            let file_locked = Mutex::new(file);
            let pairs_locked = Mutex::new(Some(Vec::<_>::new()));
            pairs_length(len).par_iter().for_each(|[i, j]| {
                for (a, arm1) in arm_list[*i as usize].iter().enumerate() {
                    for (b, arm2) in arm_list[*j as usize].iter().enumerate() {
                        let mut t = Triad::new();
                        t.add_arm(arm1);
                        t.add_arm(arm2);
                        if !t.is_rooted_core() {
                            pairs_locked
                                .lock()
                                .unwrap()
                                .as_mut()
                                .unwrap()
                                .push(((*i, a), (*j, b)));
                            if let Err(e) =
                                writeln!(file_locked.lock().unwrap(), "{},{},{},{}", i, a, j, b)
                            {
                                eprintln!("Could not write to file: {}", e);
                            }
                        }
                    }
                }
            });
            let pairs = pairs_locked.lock().unwrap().take().unwrap();
            pairs.iter().for_each(|&pair| {
                self.pairs.insert(pair);
            });
        }
    }
}

// Triplets of arm lengths of triads with num nodes
pub fn triplets_nodes(num: u32) -> Vec<[u32; 3]> {
    let mut vec = Vec::<[u32; 3]>::new();
    if num < 8 {
        return vec;
    }

    let max = ((num - 1) as f32 / 3.0).ceil() as u32;

    for i in max..num - 2 {
        for j in 1..=min(num - i - 1, i) {
            let k = num - i - j - 1;
            if k <= j && k != 0 {
                vec.push([i, j, num - i - j - 1]);
            }
        }
    }
    vec
}

// Triplets of arm lengths of triads with maximum arm length len
pub fn triplets_length(len: u32) -> Vec<[u32; 3]> {
    let mut vec = Vec::<[u32; 3]>::new();

    for i in 1..=len {
        for j in 1..=i {
            vec.push([len, i, j]);
        }
    }
    vec
}

// Pairs of arm lengths of triads with num nodes
pub fn pairs_nodes(num: u32) -> Vec<[u32; 2]> {
    let mut pairs = Vec::<[u32; 2]>::new();
    if num < 4 {
        return pairs;
    } else {
        for i in ((num as f32 / 2.0).ceil() - 1.0) as u32..num - 2 {
            pairs.push([i, num - i - 2]);
        }
    }
    pairs
}

// Pairs of arm lengths of triads with maximum arm length len
pub fn pairs_length(len: u32) -> Vec<[u32; 2]> {
    let mut pairs = Vec::<[u32; 2]>::new();
    for i in 1..=len {
        pairs.push([len, i]);
    }
    pairs
}

pub fn from_mini() {
    let arguments: Vec<String> = env::args().collect();
    let path = arguments[1].clone();
    // let path = "../small-trees/data/core-triads8.trees";

    if let Ok(file) = fs::read(&path) {
        let trees: Vec<String> = String::from_utf8_lossy(&file)
            .split_terminator('\n')
            .map(|x| x.to_string())
            .collect();
        let trees2 = trees
            .iter()
            .map(|x| {
                x.split(&[',', '[', ']', ' '][..])
                    .filter(|&x| !x.is_empty())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        for tree in trees2 {
            let mut list = AdjacencyList::<u32>::new();
            for (i, _) in tree.iter().enumerate().step_by(2) {
                let v1 = tree[i].parse::<u32>().unwrap();
                let v2 = tree[i + 1].parse::<u32>().unwrap();

                list.insert_vertex(v1);
                list.insert_vertex(v2);
                list.insert_edge(&v1, &v2);
            }
            println!("{:?}", list.edge_vec());

            let mut edges = list.edge_vec().into_iter().collect::<HashSet<_>>();
            let mut triad_vec = Vec::<String>::new();

            for u in list.vertex_iter() {
                if list.degree(u) == 3 {
                    println!("{}", &u);
                    for (v, w) in list.edge_vec().iter() {
                        if u == v {
                            edges.remove(&(*v, *w));
                            let s = arm_string(w.clone(), &mut edges, String::new());
                            triad_vec.push(String::from("0") + &s);
                        } else if u == w {
                            edges.remove(&(*v, *w));
                            let s = arm_string(v.clone(), &mut edges, String::new());
                            triad_vec.push(String::from("1") + &s);
                        }
                    }
                }
            }

            triad_vec.sort_by(|a, b| b.len().cmp(&a.len()));
            println!("{:?}", triad_vec);

            let mut path2 = path.clone();
            path2.push('2');

            if let Ok(mut file) = OpenOptions::new().append(true).create(true).open(path2) {
                if let Err(e) = writeln!(file, "{},{},{}", triad_vec[0], triad_vec[1], triad_vec[2])
                {
                    eprintln!("Could not write to file: {}", e);
                }
            }
        }
    }
}

fn arm_string<T>(u: T, vec: &mut HashSet<(T, T)>, mut s: String) -> String
where
    T: Eq + Hash + Clone + Debug,
{
    for (v, w) in vec.clone().iter() {
        if u == *v {
            s.push('0');
            vec.remove(&(v.clone(), w.clone()));
            return arm_string(w.clone(), vec, s);
        } else if u == *w {
            s.push('1');
            vec.remove(&(v.clone(), w.clone()));
            return arm_string(v.clone(), vec, s);
        }
    }
    s
}
