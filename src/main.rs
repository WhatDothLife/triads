#![deny(missing_docs)]
#![deny(clippy::all)]
#![deny(missing_debug_implementations)]

//! # Tripolys
//!
//! `tripolys` is a program for generating core triads and checking
//! polymorphisms on them. A triad is an orientation of a tree which has a
//! single vertex of degree 3 and otherwise only vertices of degree 2 and 1.

use colored::*;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    fs::OpenOptions,
    io::{self, Write},
};

mod tripolys;

use crate::tripolys::adjacency_list::write_dot;
use crate::tripolys::configuration::{Configuration, Constraint, Globals, Run, TripolysOptions};
use crate::tripolys::polymorphism::find_polymorphism;
use crate::tripolys::triads::{cores_length_range, cores_nodes_range, is_core};

/// Print error message to stderr and terminate
fn error(message: &str) -> ! {
    eprintln!("{} {}", "Error:".red(), message);
    std::process::exit(1);
}

fn run(config: Configuration, options: TripolysOptions) -> io::Result<()> {
    match config.run {
        Run::Dot => {
            if let Some(triad) = &options.triad {
                write_dot(&triad.into());
            }
        }

        Run::Core => {
            if let Some(triad) = &options.triad {
                if is_core(&triad) {
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

                    if let Some(map) = find_polymorphism(&triad.into(), polymorphism) {
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
                            triad
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
                            if find_polymorphism(&triad.into(), polymorphism).is_none() {
                                let msg =
                                    format!("\t✘ {} doesn't have a TODO polymorphism!", &triad);
                                println!("{}", msg.red());

                                let path = format!(
                                    "{}/{}/triads_{}_{}",
                                    Globals::get().data,
                                    &constraint,
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
