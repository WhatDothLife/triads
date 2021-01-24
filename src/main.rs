use std::{fs::OpenOptions, io::Write};

use rayon::prelude::*;
use triads::{
    arc_consistency::AdjacencyList,
    configuration::Configuration,
    cores::{cores_max_length, cores_nodes, Triad},
};
use triads::{configuration::Run, polymorphism::PolymorphismRegistry};

fn main() {
    let config = Configuration::parse();

    match config.run {
        Run::Triad => {
            let triad = config.triad.parse::<Triad>().unwrap();
            let polymorphism = PolymorphismRegistry::get(&config.polymorphism);

            println!("> Checking polymorphism...");

            if let Some(map) = polymorphism(&triad.adjacency_list()) {
                println!(
                    "\x1b[32m\t✔ {:?} does have a {} polymorphism!\x1b[00m",
                    triad, config.polymorphism
                );
                let mut path = String::from("data/");
                path.push_str(&config.polymorphism);
                path.push_str(&triad.as_string());
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

        Run::UpToNodes => {
            let triads = cores_nodes(config.nodes);
            let polymorphism = PolymorphismRegistry::get::<u32>(&config.polymorphism);

            println!("> Checking polymorphism...");

            triads.par_iter().for_each(|triad| {
                if polymorphism(&triad.adjacency_list()).is_none() {
                    println!(
                        "\x1b[32m\t✘ {:?} doesn't have a {} polymorphism!\x1b[00m",
                        triad, config.polymorphism
                    );
                }
            })
        }

        Run::UpToLength => {
            let triads = cores_max_length(config.length);
            let polymorphism = PolymorphismRegistry::get::<u32>(&config.polymorphism);

            println!("> Checking polymorphism...");

            triads.par_iter().for_each(|triad| {
                println!("\t- Checking {:?}", &triad);
                if polymorphism(&triad.adjacency_list()).is_none() {
                    println!(
                        "\x1b[32m\t✘ {:?} doesn't have a {} polymorphism!\x1b[00m",
                        triad, config.polymorphism
                    );
                }
            })
        }
    }
}
