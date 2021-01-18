use triads::configuration::Run;
use triads::{
    configuration::Configuration,
    cores::{cores_max_length, cores_nodes, Triad},
    polymorphism::polymorphism,
};

fn main() {
    let config = Configuration::parse();

    match config.run {
        Run::CheckTriad => {
            let arms: Vec<String> = config.triad.split(',').map(|x| x.to_owned()).collect();
            let triad = Triad::from(&arms[0], &arms[1], &arms[2]);
            let polymorphism = polymorphism(&config.polymorphism);
            let l = triad.adjacency_list();
            if !polymorphism(l) {
                println!(
                    "\x1b[31m\t✘ {:?} doesn't have a {} polymorphism!\x1b[00m",
                    triad, config.polymorphism
                );
            } else {
                println!(
                    "\x1b[32m\t✔ {:?} does have a {} polymorphism!\x1b[00m",
                    triad, config.polymorphism
                );
            }
        }

        Run::CheckUpToNodes => {
            let triads = cores_nodes(config.nodes);
            let polymorphism = polymorphism(&config.polymorphism);

            for triad in triads.iter() {
                let l = triad.adjacency_list();
                if !polymorphism(l) {
                    println!(
                        "Triad: {:?} doesn't have a {} polymorphism!",
                        triad, config.polymorphism
                    );
                } else {
                    println!(
                        "Triad: {:?} does have a {} polymorphism!",
                        triad, config.polymorphism
                    );
                }
            }
        }

        Run::CheckUpToLength => {
            let triads = cores_max_length(config.length);
            let polymorphism = polymorphism(&config.polymorphism);

            for triad in triads.iter() {
                println!("Iterating: {:?}", &triad);
                let l = triad.adjacency_list();
                if !polymorphism(l) {
                    println!(
                        "Triad: {:?} doesn't have a {} polymorphism!",
                        triad, config.polymorphism
                    );
                }
            }
        }

        Run::GenerateUpToNodes => {
            cores_nodes(config.length);
        }

        Run::GenerateUpToLength => {
            cores_max_length(config.length);
        }
    }
}
