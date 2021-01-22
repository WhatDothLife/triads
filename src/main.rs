use triads::{
    configuration::Configuration,
    cores::{cores_max_length, cores_nodes, Triad},
};
use triads::{configuration::Run, polymorphism::PolymorphismRegistry};

fn main() {
    let config = Configuration::parse();

    match config.run {
        Run::Triad => {
            let arms: Vec<String> = config.triad.split(',').map(|x| x.to_owned()).collect();
            let triad = Triad::from(&arms[0], &arms[1], &arms[2]);
            let polymorphism = PolymorphismRegistry::get(&config.polymorphism).unwrap();

            println!("> Checking polymorphism...");

            if let Some(map) = polymorphism(&triad.adjacency_list()) {
                println!(
                    "\x1b[32m\t✔ {:?} does have a {} polymorphism!\x1b[00m",
                    triad, config.polymorphism
                );
            } else {
                println!(
                    "\x1b[32m\t✘ {:?} doesn't have a {} polymorphism!\x1b[00m",
                    triad, config.polymorphism
                );
            }
        }

        Run::UpToNodes => {
            let triads = cores_nodes(config.nodes);
            let polymorphism = PolymorphismRegistry::get(&config.polymorphism).unwrap();

            println!("> Checking polymorphism...");

            for triad in triads.iter() {
                println!("Iterating {:?}", &triad);
                if polymorphism(&triad.adjacency_list()).is_none() {
                    println!(
                        "\x1b[32m\t✘ {:?} doesn't have a {} polymorphism!\x1b[00m",
                        triad, config.polymorphism
                    );
                }
            }
        }

        Run::UpToLength => {
            let triads = cores_max_length(config.length);
            let polymorphism = PolymorphismRegistry::get(&config.polymorphism).unwrap();

            println!("> Checking polymorphism...");

            for triad in triads.iter() {
                if polymorphism(&triad.adjacency_list()).is_none() {
                    println!(
                        "\x1b[32m\t✘ {:?} doesn't have a {} polymorphism!\x1b[00m",
                        triad, config.polymorphism
                    );
                }
            }
        }
    }
}
