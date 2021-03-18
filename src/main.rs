use std::{
    fs::{self, OpenOptions},
    io::Write,
};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use triads::{
    adjacency_list::from_dot,
    arc_consistency::ac_3,
    configuration::{Configuration, Globals, Run},
    polymorphism::PolymorphismRegistry,
    triads::{cores_length_range, cores_nodes_range, Triad},
};

fn main() {
    let config = Configuration::parse();

    match config.run {
        Run::DOT => {
            let triad = config.triad.parse::<Triad>().unwrap();
            // let mut product = triad.adjacency_list().power(2);
            // product.contract_if(&commutative);
            // write_dot(&product);
            // write_dot(&triad.adjacency_list());

            if let Ok(file) = fs::read("comp_2") {
                let comp = from_dot(&String::from_utf8_lossy(&file));
                let map = ac_3(&comp, &triad.adjacency_list());
                println!("{:?}", &map);
            }
        }
        Run::Triad => {
            let triad = config.triad.parse::<Triad>().unwrap();
            let polymorphism = PolymorphismRegistry::get::<u32>(&config.polymorphism);

            println!("> Checking polymorphism...");

            if let Some(map) = polymorphism(&triad.adjacency_list()) {
                println!(
                    "\x1b[32m\t✔ {:?} does have a {} polymorphism!\x1b[00m",
                    triad, config.polymorphism
                );
                let path = String::from(format!(
                    "{}/{}_{}",
                    Globals::get().data,
                    &config.polymorphism,
                    triad.as_string()
                ));
                if let Ok(mut file) = OpenOptions::new().append(true).create(true).open(path) {
                    if let Err(e) = writeln!(file, "{:?}", map) {
                        eprintln!("Couldn't write to file: {}", e);
                    }
                }
            } else {
                println!(
                    "\x1b[32m\t✘ {:?} doesn't have a {} polymorphism!\x1b[00m",
                    triad, config.polymorphism
                );
            }
        }

        Run::Nodes => {
            let triads = cores_nodes_range(config.nodes.clone());
            let polymorphism = PolymorphismRegistry::get::<u32>(&config.polymorphism);

            println!("> Checking polymorphism...");

            triads.par_iter().for_each(|triad| {
                if polymorphism(&triad.adjacency_list()).is_none() {
                    println!(
                        "\x1b[32m\t✘ {:?} doesn't have a {} polymorphism!\x1b[00m",
                        triad, config.polymorphism
                    );

                    let path = format!(
                        "{}/nodes/triads_{}{}-{}",
                        Globals::get().data,
                        &config.polymorphism,
                        &config.nodes.start,
                        &config.nodes.end
                    );

                    if let Ok(mut file) = OpenOptions::new().append(true).create(true).open(path) {
                        if let Err(e) = writeln!(file, "{:?}", &triad) {
                            eprintln!("Couldn't write to file: {}", e);
                        }
                    }
                }
            })
        }

        Run::Length => {
            let triads = cores_length_range(config.length.clone());
            let polymorphism = PolymorphismRegistry::get::<u32>(&config.polymorphism);

            println!("> Checking polymorphism...");

            triads.par_iter().for_each(|triad| {
                if polymorphism(&triad.adjacency_list()).is_none() {
                    println!(
                        "\x1b[32m\t✘ {:?} doesn't have a {} polymorphism!\x1b[00m",
                        &triad, config.polymorphism
                    );

                    let path = format!(
                        "{}/length/triads_{}{}-{}",
                        Globals::get().data,
                        &config.polymorphism,
                        &config.length.start,
                        &config.length.end
                    );

                    if let Ok(mut file) = OpenOptions::new().append(true).create(true).open(path) {
                        if let Err(e) = writeln!(file, "{:?}", &triad) {
                            eprintln!("Couldn't write to file: {}", e);
                        }
                    }
                }
            })
        }
    }
}

// fn main() {
//     // let triad = Triad::from("01011111", "1010", "1111");
//     // let polymorphism = PolymorphismRegistry::get::<u32>("commutative_backtrack");

//     // if polymorphism(&triad.adjacency_list()).is_none() {
//     //     println!("No polymorphism!");
//     // } else {
//     //     println!("Polymorphism despite backtrack!");
//     // }

//     // let triad = Triad::from("101000", "100", "111");
//     // let list = triad.adjacency_list();
//     // let map = sac(&list, &list).unwrap();
//     // println!("{:?}", map);

//     println!("{:?}", triplets_nodes(9));
// }
