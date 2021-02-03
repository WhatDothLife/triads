use std::{fs::OpenOptions, io::Write};

use rayon::prelude::*;
use triads::{
    configuration::Run, cores::pairs_nodes, cores::triplets_nodes,
    polymorphism::PolymorphismRegistry,
};
use triads::{
    configuration::{Configuration, Globals},
    cores::{cores_length_range, cores_nodes_range, Triad},
};

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
