use triads::{
    arc_consistency::ac3,
    configuration::Configuration,
    cores::{cores_max_length, cores_nodes, triplets, Triad},
    polymorphism::{commutative, polymorphism},
};
use triads::{configuration::Run, polymorphism::majority};

fn main() {
    let config = Configuration::parse();

    match config.run {
        Run::CheckTriad => {
            // let arms: Vec<String> = config.triad.split(',').map(|x| x.to_owned()).collect();
            // let triad = Triad::from(&arms[0], &arms[1], &arms[2]);
            // let polymorphism = polymorphism(&config.polymorphism);
            // let l = triad.adjacency_list();
            // if let Some(m) = polymorphism(l) {
            //     println!(
            //         "Triad: {:?} does have a {} polymorphism:",
            //         triad, config.polymorphism
            //     );
            // } else {
            //     println!(
            //         "Triad: {:?} doesn't have a {} polymorphism!",
            //         triad, config.polymorphism
            //     );
            // }
        }

        Run::CheckUpToNodes => {
            let triads = cores_nodes(config.nodes);
            println!("Generated Triads!");
            let polymorphism = polymorphism(&config.polymorphism);
            println!("Got polymorphism!");

            for triad in triads.iter() {
                let l = triad.adjacency_list();
                if !polymorphism(l) {
                    println!(
                        "Triad: {:?} doesn't have a {} polymorphism!",
                        triad, config.polymorphism
                    );
                }
            }
        }

        Run::CheckUpToLength => {
            let triads = cores_max_length(config.length);
            println!("Generated Triads!");
            let polymorphism = polymorphism(&config.polymorphism);
            println!("Got polymorphism!");

            for triad in triads.iter() {
                println!("Iterating triad {:?}", &triad);
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
