//! CirBinius verifier — re-exports verification from the prover crate.
//!
//! The `cirbinius-verifier` crate provides the public verification API.
//! The actual verification logic lives in `cirbinius-prover` to ensure
//! the prover and verifier share the same constraint evaluation code.

pub use cirbinius_prover::verify;
