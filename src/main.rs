#![deny(missing_docs)]
// #![deny(clippy::all)]
// #[deny(missing_debug_implementations)]

//! # Tripolys
//!
//! `tripolys` is a program for generating core triads and checking
//! polymorphisms on them. A triad is an orientation of a tree which has a
//! single vertex of degree 3 and otherwise only vertices of degree 2 and 1.

use std::{
    fs::OpenOptions,
    io::{self, Write},
};

use colored::*;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use triads::{
    adjacency_list::write_dot,
    configuration::{Configuration, Globals, Run, TripolysOptions},
    triads::{cores_length_range, cores_nodes_range, Constraint},
};

fn run(config: Configuration, options: TripolysOptions) -> io::Result<()> {
    match config.run {
        Run::Dot => {
            if let Some(triad) = &options.triad {
                write_dot(&triad.into());
            }
        }

        Run::Core => {
            if let Some(triad) = &options.triad {
                if triad.is_core() {
                    println!("{}", format!("✔ {} is a core!", triad).green());
                } else {
                    println!("{}", format!("✘ {} is not a core!", triad).red());
                }
            }
        }

        Run::Polymorphism => {
            if let Some(polymorphism) = &options.polymorphism {
                if let Some(ref triad) = options.triad {
                    println!("> Checking polymorphism...");

                    if let Some(map) = polymorphism(&triad.into()) {
                        let msg = format!(
                            "\t✔ {} does have a {} polymorphism!",
                            triad,
                            config.polymorphism.as_ref().unwrap()
                        );
                        println!("{}", msg.green());
                        let path = format!(
                            "{}/{}_{}",
                            Globals::get().data,
                            config.polymorphism.as_ref().unwrap(),
                            triad.as_string()
                        );
                        if let Ok(mut file) =
                            OpenOptions::new().append(true).create(true).open(path)
                        {
                            if let Err(e) = writeln!(file, "{:?}", map) {
                                eprintln!("Couldn't write to file: {}", e);
                            }
                        }
                    } else {
                        let msg = format!(
                            "\t✘ {} doesn't have a {} polymorphism!",
                            triad,
                            &config.polymorphism.unwrap()
                        );
                        println!("{}", msg.red());
                    }
                } else if let Some(constraint) = &options.constraint {
                    let range = options.range.unwrap();
                    let triads = match constraint {
                        Constraint::Length => cores_length_range(range.0..range.1),
                        Constraint::Nodes => cores_nodes_range(range.0..range.1),
                    };

                    for (i, vec) in triads.iter().enumerate() {
                        println!("> Checking polymorphism for triads with {}...", i);
                        vec.par_iter().for_each(|triad| {
                            if polymorphism(&triad.into()).is_none() {
                                let msg =
                                    format!("\t✘ {} doesn't have a TODO polymorphism!", &triad);
                                println!("{}", msg.red());

                                let path = format!(
                                    "{}/length/triads_{}_{}",
                                    Globals::get().data,
                                    config.polymorphism.as_ref().unwrap(),
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
                        });
                    }
                } else {
                    // TODO return Error
                }
            }
        }
    }
    Ok(())
}

/// Print error message to stderr and terminate
pub fn error(message: &str) -> ! {
    eprintln!("{} {}", "Error:".red(), message);
    std::process::exit(1);
}

fn main() {
    let config = Configuration::parse();
    let options = TripolysOptions::new(&config);

    let res = match options {
        Ok(opts) => run(config, opts),
        Err(ref e) => error(&e.to_string()),
    };

    match res {
        Ok(_) => {}
        Err(e) => error(&e.to_string()),
    }
}
