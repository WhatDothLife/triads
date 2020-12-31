use triads::{
    arc_consistency::arc_consistency,
    configuration::Configuration,
    core_triads::{cores, Triad},
};

fn main() {
    // let cores = cores(5);
    // println!("{:?}", cores);

    let config = Configuration::parse();
    // let triad = Triad::from_strs("10111", "11000", "11011");
    // let list = triad.adjacency_list();
    // let l2 = arc_consistency(&list, &list);
    // println!("{:?}", &l2);
}
