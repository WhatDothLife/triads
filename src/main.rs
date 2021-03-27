use std::{
    fs::{self, OpenOptions},
    io::Write,
};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use triads::{
    adjacency_list::{from_dot, write_dot, AdjacencyList},
    configuration::{Configuration, Globals, Run},
    consistency::{ac1_precolour, ac3, pc2, sac1, sac2},
    polymorphism::{commutative, PolymorphismRegistry},
    triads::{cores_length, cores_nodes, rooted_core_arms, Cache, Triad},
};

fn main() {
    let config = Configuration::parse();

    match config.run {
        Run::DOT => {
            let triad = config.triad.parse::<Triad>().unwrap();
            let mut product = triad.adjacency_list().power(2);
            product.contract_if(&commutative);
            write_dot(&product);
            write_dot(&triad.adjacency_list());
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
            let polymorphism = PolymorphismRegistry::get::<u32>(&config.polymorphism);

            println!("> Generating arms...");
            // 1. range is exclusive
            // 2. triad has a middle vertex
            // save time and subtract by 4
            let arm_list = rooted_core_arms(config.nodes.end - 4);
            println!("\x1b[32m\t✔ Generated arms\x1b[00m");

            // Cached pairs of RCAs that cannot form a core triad
            let mut cache = Cache::new();

            for i in config.nodes.clone() {
                println!("> Generating core triads with {} nodes...", i);
                let triads = cores_nodes(&arm_list, &mut cache, i);
                println!("\x1b[32m\t✔ Generated core triads\x1b[00m");

                println!("> Checking polymorphism...");

                triads.par_iter().for_each(|triad| {
                    if polymorphism(&triad.adjacency_list()).is_none() {
                        println!(
                            "\x1b[32m\t✘ {:?} doesn't have a {} polymorphism!\x1b[00m",
                            triad, config.polymorphism
                        );

                        let path = format!(
                            "{}/nodes/triads_{}_{}",
                            Globals::get().data,
                            &config.polymorphism,
                            i
                        );

                        if let Ok(mut file) =
                            OpenOptions::new().append(true).create(true).open(path)
                        {
                            if let Err(e) = writeln!(file, "{}", &triad) {
                                eprintln!("Couldn't write to file: {}", e);
                            }
                        }
                    }
                })
            }
        }

        Run::Length => {
            let polymorphism = PolymorphismRegistry::get::<u32>(&config.polymorphism);

            println!("> Generating arms...");
            let arm_list = rooted_core_arms(config.length.end - 1);
            println!("\x1b[32m\t✔ Generated arms\x1b[00m");

            // Cached pairs of RCAs that cannot form a core triad
            let mut cache = Cache::new();

            for i in config.length.clone() {
                println!("> Generating core triads with armlength {}...", i);
                let triads = cores_length(&arm_list, &mut cache, i);
                println!("\x1b[32m\t✔ Generated core triads\x1b[00m");

                println!("> Checking polymorphism...");

                triads.par_iter().for_each(|triad| {
                    if polymorphism(&triad.adjacency_list()).is_none() {
                        println!(
                            "\x1b[32m\t✘ {:?} doesn't have a {} polymorphism!\x1b[00m",
                            &triad, config.polymorphism
                        );

                        let path = format!(
                            "{}/length/triads_{}_{}",
                            Globals::get().data,
                            &config.polymorphism,
                            i
                        );

                        if let Ok(mut file) =
                            OpenOptions::new().append(true).create(true).open(path)
                        {
                            if let Err(e) = writeln!(file, "{}", &triad) {
                                eprintln!("Couldn't write to file: {}", e);
                            }
                        }
                    }
                })
            }
        }
    }
}

// fn main() {
//     // let triad = Triad::from("101000", "100", "111");
//     // let polymorphism = PolymorphismRegistry::get::<u32>("commutative_backtrack");

//     // if polymorphism(&triad.adjacency_list()).is_none() {
//     //     println!("No polymorphism or polymorphism without backtrack!");
//     // } else {
//     //     println!("Polymorphism despite backtrack!");
//     // }

//     // let triad = Triad::from("1", "1", "0");
//     // let triad2 = Triad::from("1", "1", "0");
//     // let triad2 = Triad::from("101000", "100", "111");

//     // let list = triad.adjacency_list();
//     // let list2 = triad2.adjacency_list();
//     // let map = pc2(&list, &list2);
//     // println!("{:?}", map);

//     // let mut list = AdjacencyList::<u32>::new();
//     // list.insert_vertex(0);
//     // list.insert_vertex(1);
//     // list.insert_vertex(2);
//     // list.insert_vertex(3);
//     // list.insert_vertex(4);
//     // list.insert_edge(&0, &1);
//     // list.insert_edge(&1, &2);
//     // list.insert_edge(&2, &3);
//     // list.insert_edge(&3, &4);
//     // list.insert_edge(&4, &0);

//     // let mut list2 = AdjacencyList::<u32>::new();
//     // list2.insert_vertex(0);
//     // list2.insert_vertex(1);
//     // list2.insert_vertex(2);
//     // list2.insert_vertex(3);
//     // list2.insert_edge(&0, &1);
//     // list2.insert_edge(&1, &2);
//     // list2.insert_edge(&2, &3);
//     // list2.insert_edge(&3, &0);

//     let triad = Triad::from("101000", "100", "111");
//     let list = triad.adjacency_list();
//     let list2 = triad.adjacency_list();
//     println!("{:?}", list.edge_vec());
//     let res = pc2(&list, &list2);
//     println!("{:?}", res);
// }
