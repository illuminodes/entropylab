//! BIP-32 hierarchical deterministic key derivation for `EntropyLab`.

pub mod child_number;
pub mod derivation_path;

pub use child_number::{ChildNumber, ChildNumberError, HARDENED_OFFSET};
pub use derivation_path::{DerivationPath, DerivationPathError};
