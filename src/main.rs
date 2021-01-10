use triads::{
    arc_consistency::{ac3_pruning_search, commutative},
    core_triads::cores,
};
use triads::{configuration::Configuration, core_triads::Triad};

fn main() {
    let config = Configuration::parse();
    let triads = cores(config.length);

    for triad in triads.iter() {
        let l = triad.adjacency_list();
        let mut l2 = &l * &l;
        l2.contract_if(&commutative);
        let map = ac3_pruning_search(&l2, &l);
        if let Some(m) = map {
            println!("{:?}", &m);
        } else {
            println!(
                "Triad {:?} does not have a commutative polymorphism",
                &triad
            );
            return;
        }
    }
}
