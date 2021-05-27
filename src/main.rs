//! # Tripolys
//!
//! `tripolys` is a program for generating triads and checking polymorphisms on
//! them.
//!
//! For a given digraph H the complexity of the constraint satisfaction problem
//! for H, also called CSP(H), only depends on the set of polymorphisms of H.
//! The program aims to study the structure of oriented trees with CSPs of
//! varying complexity.
//! To do this we focus on the case where H is a triad, e.g., an orientation of
//! a tree which has a single vertex of degree 3 and otherwise only vertices of
//! degree 2 and 1.

#![deny(clippy::missing_docs)]
// #![deny(clippy::missing_doc_code_examples)]
#![deny(clippy::all)]
#![deny(clippy::missing_debug_implementations)]

use colored::*;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    fs::{File, OpenOptions},
    io::{self, Write},
};
use tripolys::adjacency_list::AdjacencyList;

mod tripolys;

use crate::tripolys::configuration::{Constraint, Globals, Run, TripolysOptions};
use crate::tripolys::polymorphism::find_polymorphism;
use crate::tripolys::triad::{cores_length_range, cores_nodes_range};

/// Print error message to stderr and terminate
fn error(message: &str) -> ! {
    eprintln!("{} {}", "error:".red(), message);
    std::process::exit(1);
}

/// Runs the program based on the given configuration and options
fn run(options: TripolysOptions) -> io::Result<()> {
    match options.run {
        Run::Dot => {
            if let Some(triad) = &options.triad {
                let mut f = File::create(&options.dot.unwrap()).unwrap();
                AdjacencyList::<u32>::from(triad).to_dot(&mut f);
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

                    if let Some(map) = find_polymorphism(&triad, polymorphism) {
                        let msg =
                            format!("\t✔ {} does have a {} polymorphism!", triad, &polymorphism);
                        println!("{}", msg.green());
                        let path = format!("{}/{}_{}", Globals::get().data, &polymorphism, triad);
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
                            triad, &polymorphism
                        );
                        println!("{}", msg.red());
                    }
                } else if let Some(constraint) = &options.constraint {
                    let range = options.range.unwrap();
                    println!("> Generating triads...");
                    let triads = match constraint {
                        Constraint::Length => cores_length_range(range.clone()),
                        Constraint::Nodes => cores_nodes_range(range.clone()),
                    };
                    println!("{}", "\t✔ Generated triads!".green());

                    for (i, vec) in triads.iter().enumerate() {
                        println!(
                            "> Checking polymorphism for triads with {} {}...",
                            constraint.identity(),
                            range.start() + i as u32
                        );
                        vec.par_iter().for_each(|triad| {
                            if find_polymorphism(&triad, polymorphism).is_none() {
                                let msg = format!(
                                    "\t✘ {} doesn't have a {} polymorphism!",
                                    &triad, polymorphism
                                );
                                println!("{}", msg.red());

                                let path = format!(
                                    "{}/{}/triads_{}_{}",
                                    Globals::get().data,
                                    &constraint,
                                    polymorphism,
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

fn main() {
    let options = TripolysOptions::parse();

    let res = match options {
        Ok(opts) => run(opts),
        Err(ref e) => error(&e.to_string()),
    };

    match res {
        Ok(_) => {}
        Err(e) => error(&e.to_string()),
    }
}
