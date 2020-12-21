use triads::{
    arc_consistency::{ac_3, arc_consistency, AdjacencyList},
    core_triads::Triad,
};

fn main() {
    // let mut h = AdjacencyList::<u32>::new();
    // h.insert_vertex(0);
    // h.insert_vertex(1);
    // h.insert_vertex(2);
    // h.insert_vertex(3);

    // h.insert_edge(&0, &1);
    // h.insert_edge(&1, &2);
    // h.insert_edge(&2, &3);

    // let mut g = AdjacencyList::<u32>::new();

    // g.insert_vertex(10);
    // g.insert_vertex(20);
    // g.insert_vertex(30);
    // g.insert_vertex(40);
    // g.insert_vertex(50);

    // g.insert_edge(&10, &20);
    // g.insert_edge(&30, &20);
    // g.insert_edge(&40, &30);
    // g.insert_edge(&50, &40);

    // let l = ac_3(&h, &g);
    // let l2 = arc_consistency(&h, &g);
    // println!("{:?}", &l);
    // println!("{:?}", &l2);

    // let triad = Triad::from_vecs(
    //     vec![true, true],
    //     vec![false],
    //     vec![true, false, false, false],
    // );

    let triad = Triad::from_strings("0100", "01100", "000100");
    let list = triad.adjacency_list();

    let l3 = ac_3(&list, &list);
    println!("{:?}", &l3);
    let l2 = arc_consistency(&list, &list);
    println!("{:?}", &l2);
}
