use std::collections::HashMap;

use triads::{
    arc_consistency::{ac3_precolor, arc_consistency, AdjacencyList, Set},
    core_triads::{cores, triplets, Triad},
};

fn main() {
    // let mut h = AdjacencyList::<u32>::new();
    // h.insert_vertex(0);
    // h.insert_vertex(1);
    // h.insert_vertex(2);
    // h.insert_vertex(3);
    // h.insert_vertex(4);
    // h.insert_vertex(5);
    // h.insert_vertex(6);
    // h.insert_vertex(7);
    // h.insert_vertex(8);
    // h.insert_vertex(9);
    // h.insert_vertex(10);
    // h.insert_vertex(11);
    // h.insert_vertex(12);
    // h.insert_vertex(13);
    // h.insert_vertex(14);
    // h.insert_vertex(15);

    // h.insert_edge(&0, &1);
    // h.insert_edge(&2, &1);
    // h.insert_edge(&2, &3);
    // h.insert_edge(&3, &4);
    // h.insert_edge(&0, &5);
    // h.insert_edge(&6, &5);
    // h.insert_edge(&7, &6);
    // h.insert_edge(&7, &8);
    // h.insert_edge(&8, &9);
    // h.insert_edge(&0, &10);
    // h.insert_edge(&10, &11);
    // h.insert_edge(&11, &12);
    // h.insert_edge(&13, &12);
    // h.insert_edge(&13, &14);
    // h.insert_edge(&14, &15);

    // let triad = Triad::from_strs("01", "101", "11");
    // let list = triad.adjacency_list();
    // let edges = list.edge_vec();
    // println!("{:?}", &edges);

    // let lists_h = arc_consistency(&h, &h);
    // println!("{:?}", &lists_h);
    // let lists2_h = ac_3(&h, &h).unwrap();
    // println!("{:?}", &lists2_h);

    // let triad = Triad::from_vecs(
    //     vec![true, true],
    //     vec![false],
    //     vec![true, false, false, false],
    // );

    // let l2 = arc_consistency(&list, &list);
    // println!("{:?}", &l2);
    // let l3 = ac_3(&list, &list, HashMap::new()).unwrap();
    // println!("{:?}", &l3);

    // let paths = cores(3);
    // println!("{:?}", paths);

    // let cores = cores(7);
    // println!("{:?}", cores);

    let triad = Triad::from_strs("1111", "110000", "1110111");
    let list = triad.adjacency_list();
    let l2 = arc_consistency(&list, &list);
    println!("{:?}", &l2);
}
