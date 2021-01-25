use crate::arc_consistency::{ac3, ac3_precolour, AdjacencyList, Set};
use crate::configuration::Globals;
use std::{
    cmp::min,
    collections::{HashMap, HashSet},
    fs::{self, OpenOptions},
    io::Write,
    ops::Range,
    str::FromStr,
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

/// Returns a list of core triads up to a maximal arm length.
pub fn cores_max_length(max_length: u32) -> Vec<Triad> {
    println!("> Generating arms...");

    let arm_list = rooted_core_arms(max_length);
    println!("\x1b[32m\t✔ Generated armlist\x1b[00m");

    // Cached pairs of RCAs that cannot form a core triad
    let mut cache = Cache::new();
    let mut triadlist = Vec::<Triad>::new();

    println!("> Generating triads...");
    for len in 1..=max_length {
        cache.populate_length(len, &arm_list);

        let cores_path = format!("{}/length/cores_length{}", Globals::get().data, len);

        if let Ok(file) = fs::read(&cores_path) {
            let triads: Vec<String> = String::from_utf8_lossy(&file)
                .split_terminator('\n')
                .map(|x| x.to_owned())
                .collect();
            for triad in triads {
                triadlist.push(Triad(
                    triad.split(',').map(|x| x.to_owned()).collect::<Vec<_>>(),
                ));
            }
        } else if let Ok(mut file) = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&cores_path)
        {
            println!("\t- Generating cores with length {}", len);
            for i in 1..=len {
                for j in 1..=i {
                    for (a, arm1) in arm_list[len as usize].iter().enumerate() {
                        if arm1.chars().next().unwrap() == '0' {
                            continue;
                        };
                        for (b, arm2) in arm_list[i as usize].iter().enumerate() {
                            for (c, arm3) in arm_list[j as usize].iter().enumerate() {
                                if cache.cached((len, a), (i, b), (j, c)) {
                                    continue;
                                } else {
                                    let triad = Triad::from(arm1, arm2, arm3);
                                    if triad.is_core() {
                                        println!("\t- Adding {:?} to triadlist", &triad);
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
        } else {
            eprintln!("Could not read file: {}", &cores_path);
        }
    }
    println!("\x1b[32m\t✔ Generated triads\x1b[00m");
    triadlist
}

pub fn cores_range(range: Range<u32>) -> Vec<Triad> {
    println!("\x1b[32m> Generating armlist...\x1b[00m");
    let arm_list = rooted_core_arms(range.end - 1);
    println!("\x1b[32m\t✔ Generated armlist\x1b[00m");

    // Cached pairs of RCAs that cannot form a core triad
    let mut cache = Cache::new();
    let mut triadlist = Vec::<_>::new();

    for i in range {
        triadlist.push(cores_length(&arm_list, &mut cache, i));
    }

    triadlist.into_iter().flatten().collect()
}

// Estimates number of cores of length len for allocation
fn num_cores_length(len: u32) -> usize {
    (0.005 * (9 as u32).pow(len) as f32) as usize
}

fn cores_length(arm_list: &Vec<Vec<String>>, cache: &mut Cache, len: u32) -> Vec<Triad> {
    let cores_path = format!("{}/length/cores_length{}", Globals::get().data, len);
    let mut triad_list = Vec::<Triad>::with_capacity(num_cores_length(len));

    if let Ok(file) = fs::read(&cores_path) {
        let triads: Vec<String> = String::from_utf8_lossy(&file)
            .split_terminator('\n')
            .map(|x| x.to_owned())
            .collect();
        for triad in triads {
            triad_list.push(Triad(
                triad.split(',').map(|x| x.to_owned()).collect::<Vec<_>>(),
            ));
        }
    } else if let Ok(mut file) = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&cores_path)
    {
        println!("\t- Generating cores with length {}", len);
        cache.populate_to_length(len, &arm_list);

        for (i, j, k) in triplets_length(len) {
            println!("{}, {}, {}", i, j, k);
            for (a, arm1) in arm_list[i as usize].iter().enumerate() {
                if arm1.chars().next().unwrap() == '0' {
                    continue;
                };
                println!("{:?}", &arm1);
                for (b, arm2) in arm_list[j as usize].iter().enumerate() {
                    for (c, arm3) in arm_list[k as usize].iter().enumerate() {
                        if cache.cached((i, a), (j, b), (k, c)) {
                            continue;
                        } else {
                            let triad = Triad::from(arm1, arm2, arm3);
                            if triad.is_core() {
                                println!("\t- Adding {:?} to triadlist", &triad);
                                triad_list.push(triad);
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
    triad_list
}

pub fn cores_nodes(num_nodes: u32) -> Vec<Triad> {
    println!("> Generating arms...");

    let arm_list = rooted_core_arms(num_nodes - 3);

    println!("\x1b[32m\t✔ Generated armlist\x1b[00m");

    // Cached pairs of RCAs that cannot form a core triad
    let mut cache = Cache::new();

    let mut triadlist = Vec::<Triad>::new();

    println!("> Generating triads...");
    for num in 3..=num_nodes {
        cache.populate_nodes(num, &arm_list);

        let path = format!("{}/cores/cores_nodes{}", Globals::get().data, num);

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
            for (i, j, k) in triplets_nodes(num).iter() {
                for (a, arm1) in arm_list[*i as usize].iter().enumerate() {
                    if arm1.chars().next().unwrap() == '0' {
                        continue;
                    };
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
    println!("\x1b[32m\t✔ Generated triads\x1b[00m");
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
struct Cache {
    pairs: HashSet<((u32, usize), (u32, usize))>,
    nodes: u32,
    length: u32,
}

impl Cache {
    fn new() -> Cache {
        Cache {
            pairs: HashSet::<((u32, usize), (u32, usize))>::new(),
            nodes: 0,
            length: 0,
        }
    }

    fn insert(&mut self, value: ((u32, usize), (u32, usize))) {
        self.pairs.insert(value);
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
        } else if let Ok(mut file) = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&cache_path)
        {
            for i in (num as f32 / 2.0).ceil() as u32..num - 2 {
                for (a, arm1) in arm_list[i as usize].iter().enumerate() {
                    for (b, arm2) in arm_list[(num - i - 1) as usize].iter().enumerate() {
                        let mut t = Triad::new();
                        t.add_arm(arm1);
                        t.add_arm(arm2);
                        if !t.is_rooted_core() {
                            self.pairs.insert(((i as u32, a), (num - i - 1 as u32, b)));
                            if let Err(e) = writeln!(file, "{},{},{},{}", i, a, num - i - 1, b) {
                                eprintln!("Could not write to file: {}", e);
                            }
                        }
                    }
                }
            }
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
        } else if let Ok(mut file) = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&cache_path)
        {
            for i in 1..=len {
                for (a, arm1) in arm_list[len as usize].iter().enumerate() {
                    for (b, arm2) in arm_list[i as usize].iter().enumerate() {
                        let mut t = Triad::new();
                        t.add_arm(arm1);
                        t.add_arm(arm2);
                        if !t.is_rooted_core() {
                            self.pairs.insert(((len, a), (i, b)));
                            if let Err(e) = writeln!(file, "{},{},{},{}", len, a, i, b) {
                                eprintln!("Could not write to file: {}", e);
                            }
                        }
                    }
                }
            }
        }
    }
}

// Triplets of arm lengths of triads with num nodes
pub fn triplets_nodes(num: u32) -> Vec<(u32, u32, u32)> {
    let mut vec = Vec::<(u32, u32, u32)>::new();
    if num < 8 {
        return vec;
    }

    let max = ((num - 1) as f32 / 3.0).ceil() as u32;

    for i in max..num - 3 {
        for j in 1..=min(num - i - 1, i) {
            let k = num - i - j - 1;
            if k <= j && k != 0 {
                vec.push((i, j, num - i - j - 1));
            }
        }
    }
    // println!("Triplets: {:?}", vec); // TODO remove
    vec
}

// Triplets of arm lengths of triads with maximum arm length len
pub fn triplets_length(len: u32) -> Vec<(u32, u32, u32)> {
    let mut vec = Vec::<(u32, u32, u32)>::new();

    for i in 1..=len {
        for j in 1..=i {
            vec.push((len, i, j));
        }
    }
    // println!("Triplets: {:?}", vec); // TODO remove
    vec
}
