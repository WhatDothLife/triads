use std::error::Error;
use std::fmt;

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
