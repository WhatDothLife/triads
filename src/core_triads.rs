// This program enumerates all core triads up to a fixed path-length.

use crate::arc_consistency::AdjacencyList;

// Wrapper struct around tuple of Vec<bool>
pub struct Triad(Vec<Vec<bool>>);

impl Triad {
    pub fn new() -> Triad {
        Triad(vec![vec![], vec![], vec![]])
    }

    pub fn from_vecs(a: Vec<bool>, b: Vec<bool>, c: Vec<bool>) -> Triad {
        Triad(vec![a, b, c])
    }

    pub fn from_strings(a: &str, b: &str, c: &str) -> Triad {
        let mut triad = Triad::new();
        for a in a.chars() {
            if a == '0' {
                triad.0.get_mut(0).unwrap().push(false);
            } else if a == '1' {
                triad.0.get_mut(0).unwrap().push(true);
            } else {
                panic!("Only 0s and 1s allowed!");
            }
        }
        for b in b.chars() {
            if b == '0' {
                triad.0.get_mut(1).unwrap().push(false);
            } else if b == '1' {
                triad.0.get_mut(1).unwrap().push(true);
            } else {
                panic!("Only 0s and 1s allowed!");
            }
        }
        for c in c.chars() {
            if c == '0' {
                triad.0.get_mut(2).unwrap().push(false);
            } else if c == '1' {
                triad.0.get_mut(2).unwrap().push(true);
            } else {
                panic!("Only 0s and 1s allowed!");
            }
        }
        triad
    }

    pub fn adjacency_list(&self) -> AdjacencyList<u32> {
        let mut list = AdjacencyList::<u32>::new();
        let mut node_id: u32 = 1;

        list.insert_vertex(0);

        for i in self.0.iter() {
            for (j, v) in i.iter().enumerate() {
                list.insert_vertex(node_id);

                if j == 0 {
                    if *v {
                        list.insert_edge(&node_id, &0);
                    } else {
                        list.insert_edge(&0, &node_id);
                    }
                } else {
                    if *v {
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
