use Bachelorarbeit::{arc_consistency::*, core_triads::Triad};

fn main() {
    // let mut h = AdjacencyList::<u32>::new();
    // h.insert_vertex(0);
    // h.insert_vertex(1);
    // h.insert_vertex(2);

    // h.insert_edge(&0, &1);
    // h.insert_edge(&1, &2);
    // h.insert_edge(&1, &0);

    // let mut g = AdjacencyList::<u32>::new();

    // g.insert_vertex(10);
    // g.insert_vertex(20);
    // g.insert_vertex(30);
    // g.insert_vertex(40);

    // g.insert_edge(&10, &20);
    // g.insert_edge(&20, &30);
    // g.insert_edge(&20, &40);

    // let l = arc_consistency(&g, &h);
    // println!("{:?}", &l);

    let triad = Triad::from_vecs(
        vec![true, true],
        vec![false],
        vec![true, false, false, false],
    );

    let triad2 = Triad::from_strings("11", "0", "1001");

    let list = triad2.adjacency_list();
    let l2 = arc_consistency(&list, &list);
    println!("{:?}", &l2);
}
