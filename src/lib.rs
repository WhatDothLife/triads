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

// #![deny(missing_docs)]
// #![deny(missing_doc_code_examples)]
// #![deny(missing_debug_implementations)]
#![feature(array_value_iter)]
// #![warn(
//     clippy::all,
//     clippy::restriction,
//     clippy::pedantic,
//     clippy::nursery,
//     clippy::cargo
// )]

pub mod adjacency_list;
pub mod configuration;
pub mod consistency;
pub mod metrics;
pub mod polymorphism;
pub mod triad;
