use std::ops::{Deref, Range};

use clap::{App, Arg};
use lazy_static::lazy_static;
use std::sync::{RwLock, RwLockReadGuard};

pub enum Run {
    DOT,
    Triad,
    Length,
    Nodes,
}

pub struct Configuration {
    pub verbose: bool,
    pub length: Range<u32>,
    pub nodes: Range<u32>,
    pub polymorphism: String,
    pub triad: String,
    pub run: Run,
}

impl Configuration {
    pub fn parse() -> Configuration {
        let args = App::new("Triads")
            .version("1.0")
            .author("Michael W. <michael.wernthaler@posteo.de>")
            .about("A program for generating core triads and checking polymorphisms on them.")
            .arg(
                Arg::with_name("verbose")
                    .short("v")
                    .takes_value(false)
                    .help("Be verbose"),
            )
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
                    .value_name("NUM")
                    .help("Maximum number of nodes of triads"),
            )
            .arg(
                Arg::with_name("triad")
                    .short("t")
                    .long("triad")
                    .conflicts_with_all(&["nodes", "length"])
                    .takes_value(true)
                    .value_name("TRIAD")
                    .help("Check a polymorphism on the given triad, e.g. 111,011,01"),
            )
            .arg(
                Arg::with_name("dot")
                    .short("D")
                    .long("dot")
                    .requires("triad")
                    .help("Prints triad in dot format."),
            )
            .arg(
                Arg::with_name("polymorphism")
                    .short("p")
                    .long("polymorphism")
                    .value_name("NAME")
                    .help("Polymorphism to check")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("data")
                    .short("d")
                    .long("data")
                    .value_name("PATH")
                    .default_value("data")
                    .help("Where to store data like cores or polymorphisms")
                    .takes_value(true), // .required(true)
            )
            .get_matches();

        if !args.is_present("triad") && !args.is_present("length") && !args.is_present("nodes") {
            panic!("You must provide exactly one of the following arguments: triad, length, nodes");
        }

        let verbose = args
            .value_of("verbose")
            .unwrap_or("false")
            .parse::<bool>()
            .unwrap();
        let nodes = parse_range(args.value_of("nodes").unwrap_or("0-0"));
        let length = parse_range(args.value_of("length").unwrap_or("0-0"));
        let triad = args.value_of("triad").unwrap_or("").to_owned();
        let polymorphism = args.value_of("polymorphism").unwrap_or("").to_owned();
        let data = args.value_of("data").unwrap_or("data").to_owned();

        let run = if args.is_present("dot") {
            Run::DOT
        } else if args.is_present("triad") {
            Run::Triad
        } else if args.is_present("nodes") {
            Run::Nodes
        } else {
            Run::Length
        };

        Globals::set(Globals { data });

        Configuration {
            verbose,
            length,
            nodes,
            polymorphism,
            triad,
            run,
        }
    }
}

fn parse_range(s: &str) -> Range<u32> {
    let length_vec = s.split('-').collect::<Vec<_>>();
    let begin = length_vec.get(0).unwrap().parse::<u32>().unwrap();
    let end = if let Some(s) = length_vec.get(1) {
        s.parse::<u32>().unwrap()
    } else {
        begin
    };
    let length = begin..end + 1;
    length
}

#[derive(Default)]
pub struct Globals {
    pub data: String,
}

impl Globals {
    pub fn new(data: String) -> Self {
        Globals { data }
    }
}

lazy_static! {
    static ref GLOBALS: RwLock<Option<Globals>> = RwLock::new(Some(Globals {
        data: String::new()
    }));
}

impl Globals {
    pub fn get() -> impl Deref<Target = Globals> {
        // Unfortunately because RwLockReadGuard::map does not exist, we have
        // to create our own mapped version
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
