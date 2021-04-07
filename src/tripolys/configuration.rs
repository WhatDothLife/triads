use std::{
    fmt::{self, Display},
    ops::{Deref, RangeInclusive},
};

use std::error::Error;

use std::hash::Hash;

use clap::{App, Arg};
use lazy_static::lazy_static;
use std::sync::{RwLock, RwLockReadGuard};

use crate::tripolys::consistency::{
    ac1_precolour, ac3_precolour, sac1_precolour, sac2_precolour, LocalConsistency,
};
use crate::tripolys::polymorphism::PolymorphismKind;
use crate::tripolys::triad::Triad;

/// A set of options for tripolys
pub struct TripolysOptions {
    /// Constraint to use for triad generation (length or nodes)
    pub constraint: Option<Constraint>,

    /// Range in which to look for core triads
    pub range: Option<RangeInclusive<u32>>,

    /// Triad to operate on
    pub triad: Option<Triad>,

    /// Polymorphism to check
    pub polymorphism: Option<PolymorphismKind>,

    /// Algorithm to use
    pub algorithm: Option<Box<dyn LocalConsistency<Vec<u32>, u32>>>,
}

impl TripolysOptions {
    // TODO return Errors
    pub fn new(config: &Configuration) -> Result<TripolysOptions, OptionsError> {
        let constraint = if config.nodes.is_some() {
            Some(Constraint::Nodes)
        } else if config.length.is_some() {
            Some(Constraint::Length)
        } else {
            None
        };
        let range = if let Some(s) = config.nodes.as_ref().or(config.length.as_ref()) {
            Some(parse_range(s)?)
        } else {
            None
        };
        let triad = config.triad.as_ref().map(|s| s.parse::<Triad>().unwrap());
        let polymorphism = if let Some(m) = &config.polymorphism {
            if m == "commutative" {
                Some(PolymorphismKind::Commutative)
            } else if m == "majority" {
                Some(PolymorphismKind::Majority)
            } else if m == "siggers" {
                Some(PolymorphismKind::Siggers)
            } else {
                return Err(OptionsError::PolymorphismNotFound);
            }
        } else {
            None
        };
        let algorithm = if let Some(a) = &config.algorithm {
            Some(AlgorithmRegistry::get::<Vec<u32>, u32>(&a)?)
        } else {
            None
        };

        Ok(TripolysOptions {
            constraint,
            range,
            triad,
            polymorphism,
            algorithm,
        })
    }
}

#[derive(Debug)]
pub enum OptionsError {
    EmptyRange,
    PolymorphismNotFound,
    AlgorithmNotFound,
    FlawedTriad,
}

impl fmt::Display for OptionsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            OptionsError::EmptyRange => write!(f, "Range is empty"),
            OptionsError::PolymorphismNotFound => {
                write!(f, "No polymorphism registered with that name")
            }
            OptionsError::AlgorithmNotFound => {
                write!(f, "No algorithm registered with that name")
            }
            OptionsError::FlawedTriad => write!(f, "Unable to parse triad from argument"),
        }
    }
}

impl Error for OptionsError {}
#[derive(Debug)]
pub enum Run {
    /// Write triad to dot-format
    Dot,

    /// Check whether a triad is a core
    Core,

    /// Check whether a given polymorphism exists
    Polymorphism,
}

#[derive(Debug)]
pub struct Configuration {
    pub triad: Option<String>,
    pub nodes: Option<String>,
    pub length: Option<String>,
    pub polymorphism: Option<String>,
    pub algorithm: Option<String>,

    /// How the program should run
    pub run: Run,
}

impl Configuration {
    pub fn parse() -> Configuration {
        let args = App::new("Triads")
            .version("1.0")
            .author("Michael W. <michael.wernthaler@posteo.de>")
            .about("A program for generating core triads and checking polymorphisms on them.")
            .arg(
                Arg::with_name("length")
                    .short("l")
                    .long("length")
                    .takes_value(true)
                    .conflicts_with_all(&["nodes", "triad"])
                    .value_name("NUM or RANGE")
                    .help("Arm length of triads, e.g. 5 or 3-6"),
            )
            .arg(
                Arg::with_name("nodes")
                    .short("n")
                    .long("nodes")
                    .takes_value(true)
                    .conflicts_with_all(&["length", "triad"])
                    .value_name("NUM or RANGE")
                    .help("Maximum number of nodes of triads, e.g. 10 or 5-9"),
            )
            .arg(
                Arg::with_name("triad")
                    .short("t")
                    .long("triad")
                    .conflicts_with_all(&["nodes", "length"])
                    .takes_value(true)
                    .value_name("TRIAD")
                    .help("Triad to operate on, e.g. 111,011,01"),
            )
            .arg(
                Arg::with_name("core")
                    .short("c")
                    .long("core")
                    .requires("triad")
                    .help("Checks if triad is a core"),
            )
            .arg(
                Arg::with_name("dot")
                    .short("D")
                    .long("dot")
                    .requires("triad")
                    .help("Prints triad in dot format"),
            )
            .arg(
                Arg::with_name("polymorphism")
                    .short("p")
                    .long("polymorphism")
                    .value_name("NAME")
                    .help("Polymorphism to check, e.g. commutative")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("algorithm")
                    .short("a")
                    .long("algorithm")
                    .value_name("NAME")
                    .default_value("ac3")
                    .help("Algorithm for polymorphism search")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("data")
                    .short("d")
                    .long("data")
                    .value_name("PATH")
                    .default_value("data")
                    .help("Where to store the data")
                    .takes_value(true), // .required(true)
            )
            .get_matches();

        if !args.is_present("triad") && !args.is_present("length") && !args.is_present("nodes") {
            panic!("You must provide exactly one of the following arguments: triad, length, nodes");
        }

        let triad = args.value_of("triad").map(|s| s.to_string());
        let nodes = args.value_of("nodes").map(|s| s.to_string());
        let length = args.value_of("length").map(|s| s.to_string());
        let polymorphism = args.value_of("polymorphism").map(|s| s.to_string());
        let algorithm = args.value_of("algorithm").map(|s| s.to_string());

        let run = if args.is_present("dot") {
            Run::Dot
        } else if args.is_present("core") {
            Run::Core
        } else {
            Run::Polymorphism
        };

        let data = args.value_of("data").unwrap_or("data").to_string();
        Globals::set(Globals { data });

        Configuration {
            triad,
            nodes,
            length,
            polymorphism,
            algorithm,
            run,
        }
    }
}

#[derive(Debug)]
pub enum Constraint {
    Nodes,
    Length,
}

impl Display for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Constraint::Nodes => write!(f, "nodes"),
            Constraint::Length => write!(f, "length"),
        }
    }
}

impl Constraint {
    pub fn identity(&self) -> &str {
        match self {
            Constraint::Length => "maximal armlength",
            Constraint::Nodes => "node-number",
        }
    }
}

#[derive(Default)]
pub struct Globals {
    pub data: String,
}

impl Globals {
    pub fn new(data: &str) -> Self {
        Globals { data: data.into() }
    }
}

lazy_static! {
    static ref GLOBALS: RwLock<Option<Globals>> = RwLock::new(Some(Globals {
        data: String::new()
    }));
}

impl Globals {
    pub fn get() -> impl Deref<Target = Globals> {
        struct Guard(RwLockReadGuard<'static, Option<Globals>>);
        impl Deref for Guard {
            type Target = Globals;
            fn deref(&self) -> &Globals {
                self.0.as_ref().expect("Initialize globals first")
            }
        }
        Guard(GLOBALS.read().expect("RwLock is poisoned"))
    }

    pub fn set(value: Globals) {
        *GLOBALS.write().expect("RwLock is poisoned") = Some(value);
    }
}

fn parse_range(s: &str) -> Result<RangeInclusive<u32>, OptionsError> {
    let v = s.split('-').collect::<Vec<_>>();
    let begin = v.get(0).unwrap().parse::<u32>().unwrap();
    let end = if let Some(s) = v.get(1) {
        s.parse::<u32>().unwrap()
    } else {
        begin
    };
    let r = begin..=end;
    if r.is_empty() {
        Err(OptionsError::EmptyRange)
    } else {
        Ok(r)
    }
}

pub struct AlgorithmRegistry;

impl AlgorithmRegistry {
    pub fn get<V0, V1>(algo: &str) -> Result<Box<dyn LocalConsistency<V0, V1>>, OptionsError>
    where
        V0: Eq + Clone + Hash + 'static,
        V1: Eq + Clone + Hash + 'static,
    {
        match algo {
            "ac1" => Ok(Box::new(ac1_precolour)),
            "ac3" => Ok(Box::new(ac3_precolour)),
            "sac1" => Ok(Box::new(sac1_precolour)),
            "sac2" => Ok(Box::new(sac2_precolour)),
            // "pc2" => Ok(Box::new(pc2)),
            &_ => Err(OptionsError::AlgorithmNotFound),
        }
    }
}
