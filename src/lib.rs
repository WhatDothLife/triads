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
// #![warn(clippy::correctness, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::use_self)]

pub mod adjacency_list;
pub mod configuration;
pub mod consistency;
pub mod metrics;
pub mod polymorphism;
pub mod triad;
