use clap::{App, Arg};

pub enum Run {
    Triad,
    UpToLength,
    UpToNodes,
    // GenerateUpToLength,
    // GenerateUpToNodes,
}

pub struct Configuration {
    pub verbose: bool,
    pub length: u32,
    pub nodes: u32,
    pub polymorphism: String,
    pub triad: String,
    pub data: String,
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
                    .value_name("NUM")
                    .help("Maximum arm length of triads"),
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
                Arg::with_name("polymorphism")
                    .short("p")
                    .long("polymorphism")
                    .value_name("NAME")
                    .help("Polymorphism to check")
                    .takes_value(true)
                    .required(true),
            )
            .arg(
                Arg::with_name("data")
                    .short("d")
                    .long("data")
                    .value_name("PATH")
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
        let length = args
            .value_of("length")
            .unwrap_or("0")
            .parse::<u32>()
            .unwrap();
        let nodes = args
            .value_of("nodes")
            .unwrap_or("0")
            .parse::<u32>()
            .unwrap();
        let triad = args.value_of("triad").unwrap_or("").to_owned();
        let polymorphism = args.value_of("polymorphism").unwrap_or("").to_owned();
        let data = args.value_of("data").unwrap_or("").to_owned();

        let run = if args.is_present("triad") {
            Run::Triad
        } else if args.is_present("nodes") {
            Run::UpToNodes
        } else {
            Run::UpToLength
        };

        Configuration {
            verbose,
            length,
            nodes,
            polymorphism,
            triad,
            data,
            run,
        }
    }
}
