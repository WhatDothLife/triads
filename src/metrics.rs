use std::{
    fs::OpenOptions,
    io::{Error, Write},
    time::Duration,
};

use colored::Colorize;

use crate::{configuration::TripolysOptions, polymorphism::Polymorphism, triad::Triad};

#[derive(Debug)]
pub struct SearchLog {
    log: Vec<(Triad, Metrics)>,
    path: String,
}

impl SearchLog {
    pub fn new(path: String) -> SearchLog {
        SearchLog {
            log: Vec::<(Triad, Metrics)>::new(),
            path,
        }
    }

    pub fn add(&mut self, triad: Triad, metrics: Metrics) {
        self.log.push((triad, metrics));
    }

    pub fn write(&self) -> Result<(), Error> {
        if let Ok(mut file) = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.path)
        {
            writeln!(
                file,
                "triad,polymorphism,backtracked,indicator_time,ac_time,search_time",
            )?;
            for (triad, metrics) in &self.log {
                writeln!(file, "{},{}", triad, metrics.format())?;
            }
        }
        Ok(())
    }
}

/// Metrics is a struct which allows to store some information about
/// polymorphism search.
#[derive(Debug)]
pub struct Metrics {
    pub backtracked: u32,
    pub indicator_time: Duration,
    pub ac_time: Duration,
    pub search_time: Duration,
    pub polymorphism: Option<Polymorphism<u32>>,
}

impl Metrics {
    pub fn new() -> Metrics {
        Metrics {
            backtracked: 0,
            indicator_time: Duration::default(),
            ac_time: Duration::default(),
            search_time: Duration::default(),
            polymorphism: None,
        }
    }

    pub fn format(&self) -> String {
        format!(
            "{},{},{:?},{:?},{:?}",
            if self.polymorphism.is_some() {
                'y'
            } else {
                'n'
            },
            self.backtracked,
            self.indicator_time,
            self.ac_time,
            self.search_time
        )
    }

    pub fn print_console(&self, options: &TripolysOptions, triad: &Triad) -> Result<(), Error> {
        if self.polymorphism.is_some() {
            let msg = format!(
                "\t✔ {} does have a {} polymorphism!\n",
                triad,
                options.polymorphism.as_ref().unwrap()
            );
            println!("{}", msg.green());
        } else {
            let msg = format!(
                "\t✘ {} doesn't have a {} polymorphism!\n",
                triad,
                options.polymorphism.as_ref().unwrap()
            );
            println!("{}", msg.red());
        };
        println!("backtracked: {}", self.backtracked);
        println!("indicator_time: {:?}", self.indicator_time);
        println!("ac_time: {:?}", self.ac_time);
        println!("search_time: {:?}", self.search_time);

        Ok(())
    }
}
