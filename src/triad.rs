//! The simplest form of an orientation of a tree that is not a path.
use std::{
    cmp::min,
    collections::HashSet,
    convert::TryFrom,
    fmt, fs,
    hash::Hash,
    io::{self, Write},
    str::FromStr,
    sync::Mutex,
};

use crate::{adjacency_list::AdjacencyList, configuration::Globals, list};
use rayon::prelude::*;

use super::consistency::{ac3, ac3_precolour, Lists};

/// A triad graph implemented as a wrapper struct around a `Vec<String>`.
///
/// Each String in the Vector represents a path that leaves the
/// vertex of degree 3 of the triad. `'0'` stands for
/// forward edge and `'1'` for backward edge.
///
/// Note that we don't restrict the triad to have exactly three arms.
/// Instead there must be at most three arms, and every triad that has less
/// can be considered a "partial triad".
#[derive(Debug, Clone, Hash, Default)]
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
    /// let t = Triad::from("0", "1", "00");
    /// ```
    pub fn from_strs(a: &str, b: &str, c: &str) -> Triad {
        Triad(vec![a.into(), b.into(), c.into()])
    }

    /// Adds an arm to the triad.
    ///
    /// # Panics
    ///
    /// Panics, if the triad already has 3 arms.
    pub fn add_arm(&mut self, arm: &str) {
        if self.0.len() == 3 {
            panic!("Triad already has 3 arms!");
        }
        self.0.push(String::from(arm));
    }

    /// Returns `true` if the triad is a core, and `false` otherwise.  A graph G is
    /// called a core if every endomorphism of G is an automorphism.
    ///
    /// # Examples
    /// ```
    /// let triad = Triad::from_strs("1000", "11", "0");
    ///
    /// asserteq!(true, triad.is_core());
    /// ```
    pub fn is_core(&self) -> bool {
        for (_, v) in ac3(&self.into(), &self.into()).unwrap() {
            if v.size() != 1 {
                return false;
            }
        }
        true
    }

    /// Returns `true` if the triad is a rooted core, and `false` otherwise.
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
        let map = ac3_precolour_0(&self.into(), &self.into()).unwrap();
        for (_, v) in map {
            if v.size() != 1 {
                return false;
            }
        }
        true
    }
}

/// A modification of `ac3-precolour` that restricts the domain of vertex 0 to {0}. It
/// is used to determine whether a partial triad is a rooted core.
fn ac3_precolour_0(g0: &AdjacencyList<u32>, g1: &AdjacencyList<u32>) -> Option<Lists<u32, u32>> {
    let mut lists = Lists::new();
    lists.insert(0, list![0]);
    ac3_precolour(g0, g1, lists)
}

impl fmt::Display for Triad {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        for (i, arm) in self.0.iter().enumerate() {
            if i > 0 {
                s.push('_');
            }
            s.push_str(arm);
        }
        write!(f, "{}", s)
    }
}

impl FromStr for Triad {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let arms: Vec<String> = s.split(',').map(|x| x.into()).collect();
        if arms.len() > 3 {
            return Err("Too many arms were given!");
        }
        for arm in arms.iter() {
            if !arm.is_empty() {
                let res: Vec<bool> = arm.chars().map(|c| c == '0' || c == '1').collect();
                if res.contains(&false) {
                    return Err("Only 0s and 1s allowed!");
                }
            }
        }

        if let Some(arm1) = arms.get(0) {
            if let Some(arm2) = arms.get(1) {
                if let Some(arm3) = arms.get(2) {
                    return Ok(Triad::from_strs(arm1, arm2, arm3));
                }
            }
        }

        Err("Unable to parse triad from the given string slice!")
    }
}

/// Builds an adjacencylist from a triad. The root is labeled with 0.
impl From<&Triad> for AdjacencyList<u32> {
    fn from(triad: &Triad) -> Self {
        let mut list = AdjacencyList::<u32>::new();
        let mut node_id = 1;

        list.add_vertex(0);

        for i in triad.0.iter() {
            for (j, v) in i.chars().enumerate() {
                list.add_vertex(node_id);

                if j == 0 {
                    if v == '1' {
                        list.add_edge(&node_id, &0);
                    } else {
                        list.add_edge(&0, &node_id);
                    }
                } else if v == '1' {
                    list.add_edge(&node_id, &(node_id - 1));
                } else {
                    list.add_edge(&(node_id - 1), &node_id);
                }

                node_id += 1;
            }
        }
        list
    }
}

impl TryFrom<AdjacencyList<u32>> for Triad {
    type Error = &'static str;

    fn try_from(list: AdjacencyList<u32>) -> Result<Self, Self::Error> {
        let mut edges = list.edges().into_iter().collect::<HashSet<_>>();
        let mut triad_vec = Vec::<(u32, String)>::new();

        for u in list.vertices() {
            if list.degree(u) == 3 {
                for (v, w) in list.edges() {
                    if *u == v {
                        edges.remove(&(v.clone(), w.clone()));
                        let s = arm_string(w.clone(), &mut edges, String::new());
                        triad_vec.push((w, String::from("0") + &s));
                    } else if *u == w {
                        edges.remove(&(v.clone(), w.clone()));
                        let s = arm_string(v.clone(), &mut edges, String::new());
                        triad_vec.push((v, String::from("1") + &s));
                    }
                }
            }
        }

        triad_vec.sort_by_key(|(i, _)| *i);
        if let Some((_, arm1)) = triad_vec.get(0) {
            if let Some((_, arm2)) = triad_vec.get(1) {
                if let Some((_, arm3)) = triad_vec.get(2) {
                    return Ok(Triad::from_strs(arm1, arm2, arm3));
                }
            }
        }

        Err("Unable to parse triad from the given adjacencylist")
    }
}

/// Helper function that encodes a set of edges as a String of '0's and '1's.
fn arm_string<T>(u: T, vec: &mut HashSet<(T, T)>, mut s: String) -> String
where
    T: Eq + Hash + Clone,
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

/// Returns all arms with maximal length max_len that are rooted cores. For each
/// index i the `Vec` at position i holds all rooted core arms of
/// length i (`Vec` at index 0 is empty).
fn rooted_core_arms(max_len: u32) -> Vec<Vec<String>> {
    let mut arm_list = vec![vec![String::new()]];
    let mut last = vec![String::new()];

    for len in 1..=max_len {
        let path = format!("{}/arms/arms{}", Globals::get().data, len);
        let mut arm_list_len = Vec::new();

        if let Ok(file) = fs::read(&path) {
            let arms = String::from_utf8_lossy(&file)
                .split_terminator('\n')
                .map(|x| x.to_string())
                .collect();
            arm_list_len = arms;
        } else if let Ok(mut file) = fs::OpenOptions::new().append(true).create(true).open(&path) {
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
            panic!("Could not create file: {}", &path);
        };
        last = arm_list_len.clone();
        arm_list.push(arm_list_len)
    }
    arm_list
}

// A cache to speed up the generation of core triads
struct Cache {
    pairs: HashSet<((u32, usize), (u32, usize))>,
    counter: u32,
}

impl Cache {
    fn new() -> Cache {
        Cache {
            pairs: HashSet::<((u32, usize), (u32, usize))>::new(),
            counter: 0,
        }
    }

    fn cached(&self, a: (u32, usize), b: (u32, usize), c: (u32, usize)) -> bool {
        if self.pairs.contains(&(a, b))
            || self.pairs.contains(&(a, c))
            || self.pairs.contains(&(b, c))
        {
            return true;
        }
        false
    }

    fn populate_to(&mut self, num: u32, arm_list: &[Vec<String>], cons: &Constraint) {
        for i in self.counter..=num {
            self.populate(i, arm_list, cons);
        }
        self.counter = num;
    }

    fn populate(&mut self, num: u32, arm_list: &[Vec<String>], cons: &Constraint) {
        let path = format!("{}/nodes/pairs_{}", Globals::get().data, num);

        if let Ok(pairs_vec) = FileParser::read_pairs(&path) {
            for pair in pairs_vec.into_iter() {
                self.pairs.insert(pair);
            }
        } else if let Ok(file) = fs::OpenOptions::new().append(true).create(true).open(&path) {
            let file_locked = Mutex::new(file);
            let pairs_locked = Mutex::new(Some(Vec::<_>::new()));

            cons.pairs(num).par_iter().for_each(|[i, j]| {
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
        } else {
            panic!("Could not create file: {}", &path);
        }
    }
}

#[derive(Debug)]
enum Constraint {
    Nodes,
    Length,
}

impl Constraint {
    fn triplets(&self, num: u32) -> Vec<[u32; 3]> {
        match self {
            // Triplets of arm lengths of triads with num nodes
            Constraint::Nodes => {
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
            // Triplets of arm lengths of triads with maximum arm length num
            Constraint::Length => {
                let mut vec = Vec::<[u32; 3]>::new();

                for i in 1..=num {
                    for j in 1..=i {
                        vec.push([num, i, j]);
                    }
                }
                vec
            }
        }
    }

    fn pairs(&self, num: u32) -> Vec<[u32; 2]> {
        match self {
            // Pairs of arm lengths of triads with num nodes
            Constraint::Nodes => {
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
            Constraint::Length => {
                let mut pairs = Vec::<[u32; 2]>::new();
                for i in 1..=num {
                    pairs.push([num, i]);
                }
                pairs
            }
        }
    }

    fn max_armlength(&self, num: u32) -> u32 {
        match self {
            Constraint::Nodes => num - 3,
            Constraint::Length => num,
        }
    }
}

impl fmt::Display for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Constraint::Nodes => write!(f, "nodes"),
            Constraint::Length => write!(f, "length"),
        }
    }
}

/// Returns all core triads whose longest arm has length `len`.
pub fn cores_length(len: u32) -> Vec<Triad> {
    cores(len, &Constraint::Length)
}

/// Returns all core triads with `num` nodes.
pub fn cores_nodes(num: u32) -> Vec<Triad> {
    cores(num, &Constraint::Nodes)
}

/// Returns all core triads whose longest arm has a length contained in `range`.
pub fn cores_length_range<R>(range: R) -> Vec<Vec<Triad>>
where
    R: RangeIter<u32>,
{
    cores_range(range, &Constraint::Length)
}

/// Returns all core triads whose number of nodes is contained in `range`.
pub fn cores_nodes_range<R>(range: R) -> Vec<Vec<Triad>>
where
    R: RangeIter<u32>,
{
    cores_range(range, &Constraint::Nodes)
}

fn cores_range<R>(range: R, cons: &Constraint) -> Vec<Vec<Triad>>
where
    R: RangeIter<u32>,
{
    let arm_list = rooted_core_arms(cons.max_armlength(range.end_bound()));
    let mut cache = Cache::new();
    let mut vec = Vec::<_>::new();
    for i in range {
        vec.push(_cores(&arm_list, &mut cache, i, cons));
    }
    vec
}

fn cores(num: u32, cons: &Constraint) -> Vec<Triad> {
    cores_range(num..=num, cons).into_iter().flatten().collect()
}

fn _cores(arm_list: &[Vec<String>], cache: &mut Cache, num: u32, cons: &Constraint) -> Vec<Triad> {
    cache.populate_to(num, &arm_list, &cons);

    let triadlist = Mutex::new(Some(Vec::<Triad>::new()));
    let path = format!("{}/{}/cores_{}", Globals::get().data, cons, num);

    if let Ok(triad_vec) = FileParser::read_triads(&path) {
        for triad in triad_vec.into_iter() {
            triadlist.lock().unwrap().as_mut().unwrap().push(triad);
        }
    } else if let Ok(file) = fs::OpenOptions::new().append(true).create(true).open(&path) {
        let file_locked = Mutex::new(file);

        cons.triplets(num).par_iter().for_each(|[i, j, k]| {
            for (a, arm1) in arm_list[*i as usize].iter().enumerate() {
                for (b, arm2) in arm_list[*j as usize].iter().enumerate() {
                    for (c, arm3) in arm_list[*k as usize].iter().enumerate() {
                        let mut count = 0;

                        for arm in [arm1, arm2, arm3].iter() {
                            if arm.starts_with('1') {
                                count += 1;
                            }
                        }
                        if count > 1 {
                            continue;
                        }
                        if cache.cached((*i, a), (*j, b), (*k, c)) {
                            continue;
                        } else {
                            let triad = Triad::from_strs(arm1, arm2, arm3);
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
        panic!("Could not create file: {}", &path);
    }
    let list = triadlist.lock().unwrap().take().unwrap();
    list
}

/// A `RangeIter` iterates over a finite range.
pub trait RangeIter<T: PartialOrd<T>>: Iterator<Item = T> {
    /// Returns the lower bound of the range (inclusive).
    fn start_bound(&self) -> T;
    /// Returns the upper bound of the range (inclusive).
    fn end_bound(&self) -> T;
}

impl RangeIter<u32> for std::ops::Range<u32> {
    fn start_bound(&self) -> u32 {
        self.start
    }

    fn end_bound(&self) -> u32 {
        self.end - 1
    }
}

impl RangeIter<u32> for std::ops::RangeInclusive<u32> {
    fn start_bound(&self) -> u32 {
        *self.start()
    }

    fn end_bound(&self) -> u32 {
        *self.end()
    }
}

struct FileParser;

impl FileParser {
    fn read_triads(path: &str) -> Result<Vec<Triad>, io::Error> {
        let file = fs::read(&path)?;
        let triads: Vec<String> = String::from_utf8_lossy(&file)
            .split_terminator('\n')
            .map(|x| x.into())
            .collect();

        Ok(triads
            .into_iter()
            .map(|t| Triad(t.split(',').map(|x| x.into()).collect::<Vec<_>>()))
            .collect::<Vec<_>>())
    }

    fn read_pairs(path: &str) -> Result<Vec<((u32, usize), (u32, usize))>, io::Error> {
        let file = fs::read(&path)?;
        let s: Vec<Vec<String>> = String::from_utf8_lossy(&file)
            .split_terminator('\n')
            .map(|x| {
                x.to_string()
                    .split_terminator(',')
                    .map(|y| y.into())
                    .collect()
            })
            .collect();

        let mut pairs = Vec::<_>::new();
        for pair in s {
            let len = pair[0].parse::<u32>().unwrap();
            let a = pair[1].parse::<usize>().unwrap();
            let i = pair[2].parse::<u32>().unwrap();
            let b = pair[3].parse::<usize>().unwrap();
            pairs.push(((len, a), (i, b)));
        }
        Ok(pairs)
    }

    #[allow(dead_code)]
    fn read_arms(len: u32) -> Result<Vec<String>, io::Error> {
        let path = format!("{}/arms/arms{}", Globals::get().data, len);
        let file = fs::read(&path)?;
        let arms = String::from_utf8_lossy(&file)
            .split_terminator('\n')
            .map(|x| x.to_string())
            .collect::<Vec<_>>();
        Ok(arms)
    }
}

/// Returns the level of the vertex v.
///
/// # Panics
///
/// Panics, if the vertex doesn't exist.
pub fn level(v: u32, t: &Triad) -> i32 {
    let mut level = 0;
    let mut count = v;
    for arm in t.0.clone() {
        if count <= (arm.len() as u32) {
            level = level_arm(count, &arm);
            break;
        } else {
            count -= arm.len() as u32;
        }
    }
    level
}

fn level_arm(mut count: u32, arm: &str) -> i32 {
    let mut level = 0;
    let mut chars = arm.chars();
    while count > 0 {
        let c = chars.next().unwrap();
        if c == '0' {
            level += 1;
        } else {
            level -= 1;
        }
        count -= 1;
    }
    level
}
