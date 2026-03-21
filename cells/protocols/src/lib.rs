//! Shared protocol placeholders for KolibriOS AI application crates.
//!
//! This crate currently provides a minimal integration point so the Rust
//! workspace can resolve application dependencies while protocol definitions
//! are being formalized.

/// Shared protocol version exposed to application crates.
pub const PROTOCOL_VERSION: &str = "0.1.0-dev";
