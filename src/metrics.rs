use std::{
    fs::OpenOptions,
    io::{Error, Write},
    time::Duration,
};

use colored::Colorize;

use crate::{
    configuration::{Globals, TripolysOptions},
    polymorphism::Polymorphism,
    triad::Triad,
};

/// Metrics is a struct which allows to store some information about
/// polymorphism search.
#[derive(Debug)]
pub struct Metrics {
    pub backtracked: u32,
    pub indicator_time: Duration,
    pub ac_time: Duration,
    pub search_time: Duration,
    pub triad: Triad,
    pub polymorphism: Option<Polymorphism<u32>>,
}

impl Metrics {
    pub fn new() -> Metrics {
        Metrics {
            triad: Triad::new(),
            polymorphism: None,
            backtracked: 0,
            indicator_time: Duration::default(),
            ac_time: Duration::default(),
            search_time: Duration::default(),
        }
    }

    pub fn write(&self, options: &TripolysOptions) -> Result<(), Error> {
        let path = format!(
            "{}/{}/triads_{}_{}y",
            Globals::get().data,
            options.constraint.as_ref().unwrap(),
            options.polymorphism.as_ref().unwrap(),
            100 //TODO replace 100 by more sophisticated logic
        );

        if let Ok(mut file) = OpenOptions::new().append(true).create(true).open(path) {
            writeln!(
                file,
                "triad,polymorphism,backtracked,indicator_time,ac_time,search_time",
            )?;
            writeln!(
                file,
                "{},{},{},{:?},{:?},{:?}",
                self.triad,
                if self.polymorphism.is_some() {
                    'y'
                } else {
                    'n'
                },
                self.backtracked,
                self.indicator_time,
                self.ac_time,
                self.search_time
            )?;
        }
        Ok(())
    }

    pub fn print(&self, options: &TripolysOptions) -> Result<(), Error> {
        if self.polymorphism.is_some() {
            let msg = format!(
                "\t✔ {} does have a {} polymorphism!",
                self.triad,
                options.polymorphism.as_ref().unwrap()
            );
            println!("{}", msg.green());
        } else {
            let msg = format!(
                "\t✘ {} doesn't have a {} polymorphism!",
                self.triad,
                options.polymorphism.as_ref().unwrap()
            );
            println!("{}", msg.red());
        };
        println!("\tbacktracked: {}", self.backtracked);
        println!("\tindicator_time: {:?}", self.indicator_time);
        println!("\tac_time: {:?}", self.ac_time);
        println!("\tsearch_time: {:?}", self.search_time);

        Ok(())
    }
}
