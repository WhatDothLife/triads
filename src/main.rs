use triads::{
    arc_consistency::arc_consistency,
    core_triads::{cores, Triad},
};

fn main() {
    let cores = cores(4);
    println!("{:?}", cores);

    // let triad = Triad::from_strs("1111", "110000", "1110111");
    // let list = triad.adjacency_list();
    // let l2 = arc_consistency(&list, &list);
    // println!("{:?}", &l2);
}
