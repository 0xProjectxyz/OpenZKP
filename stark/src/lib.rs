// TODO: #![deny(missing_docs)]
#![warn(clippy::all)]
#![deny(warnings)]
mod channel;
mod fft;
pub mod fibonacci;
mod hash;
mod hashable;
mod masked_keccak;
mod merkle;
mod mmap_vec;
mod polynomial;
mod proofs;
mod trace_table;
mod utils;

pub use trace_table::TraceTable;

pub use merkle::verify;
pub use proofs::{stark_proof, ProofParams};

// Exports for benchmarking
// TODO: Avoid publicly exposing.
pub use fft::fft_cofactor_bit_reversed;
pub use merkle::make_tree;
