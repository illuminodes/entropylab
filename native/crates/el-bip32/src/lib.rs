//! BIP-32 key derivation for `EntropyLab`.
//!
//! The BIP-32 primitives come from `bitcoin::bip32`. Use `Xpub`, `Xpriv`,
//! `DerivationPath`, and `ChildNumber` from there directly. This crate adds
//! only what that module leaves out.
//!
//! `bitcoin::bip32::DerivationPath` parses both hardened markers, writes the
//! apostrophe form through `Display`, and writes the `h` form through the
//! alternate flag, `{:#}`. It omits the leading `m/`.

pub mod slip132;

pub use slip132::{KeyVersion, Slip132Error, Slip132Family, Slip132Network, Slip132Scope};
