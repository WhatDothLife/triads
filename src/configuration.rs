use clap::{App, Arg};

#[derive(Debug)]
pub struct Configuration {
    pub verbose: bool,
    pub length: u32,
    pub polymorphism: String,
}

impl Configuration {
    pub fn parse() -> Configuration {
        let args = App::new("triads")
            .version("1.0")
            .author("Michael W. <michael.wernthaler@posteo.de>")
            .about("A program for generating core triads and checking polymorphisms.")
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
                    .required(true)
                    .value_name("NUM")
                    .help("Maximum arm length of triad"),
            )
            .arg(
                Arg::with_name("polymorphism")
                    .short("p")
                    .long("polymorphism")
                    .value_name("NAME")
                    .help("Polymorphism to check")
                    .takes_value(true),
            )
            .get_matches();

        let verbose = args
            .value_of("verbose")
            .unwrap_or("false")
            .parse::<bool>()
            .unwrap();
        let length = args
            .value_of("length")
            .expect("length must be specified!")
            .parse::<u32>()
            .unwrap();
        let polymorphism = args.value_of("polymorphism").unwrap_or("").to_string();

        Configuration {
            verbose,
            length,
            polymorphism,
        }
    }
}
