use std::{
    fs::OpenOptions,
    io::{Error, Write},
    time::Duration,
};

use colored::Colorize;

use crate::{
    polymorphism::{Polymorphism, PolymorphismConfiguration},
    triad::Triad,
};

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
                "triad,polymorphism,backtracked,indicator_time,ac_time,search_time,total_time",
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
#[derive(Debug, Default)]
pub struct Metrics {
    pub backtracked: u32,
    pub indicator_time: Duration,
    pub ac_time: Duration,
    pub search_time: Duration,
    pub total_time: Duration,
    pub polymorphism: Option<Polymorphism<u32>>,
}

impl Metrics {
    pub fn new() -> Metrics {
        Metrics {
            backtracked: 0,
            indicator_time: Duration::default(),
            ac_time: Duration::default(),
            search_time: Duration::default(),
            total_time: Duration::default(),
            polymorphism: None,
        }
    }

    pub fn format(&self) -> String {
        let total_time = self.indicator_time + self.ac_time + self.search_time;
        format!(
            "{},{},{:?},{:?},{:?},{:?}",
            if self.polymorphism.is_some() {
                'y'
            } else {
                'n'
            },
            self.backtracked,
            self.indicator_time,
            self.ac_time,
            self.search_time,
            total_time
        )
    }

    pub fn print_console(
        &self,
        config: &PolymorphismConfiguration,
        triad: &Triad,
    ) -> Result<(), Error> {
        if self.polymorphism.is_some() {
            let msg = format!(
                "\t\u{2714} {} does have a(n) {} polymorphism!\n",
                triad, config
            );
            println!("{}", msg.green());
        } else {
            let msg = format!(
                "\t\u{2718} {} doesn't have a(n) {} polymorphism!\n",
                triad, config
            );
            println!("{}", msg.red());
        };
        let total_time = self.indicator_time + self.ac_time + self.search_time;
        println!("backtracked: {}", self.backtracked);
        println!("indicator_time: {:?}", self.indicator_time);
        println!("ac_time: {:?}", self.ac_time);
        println!("search_time: {:?}", self.search_time);
        println!("total_time: {:?}", total_time);

        Ok(())
    }
}
