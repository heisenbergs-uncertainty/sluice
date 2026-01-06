//! Generated protobuf code for Sluice protocol.
//!
//! This crate provides the protocol buffer definitions and generated
//! gRPC code for the Sluice message broker.

// Include the generated proto code
pub mod sluice {
    pub mod v1 {
        tonic::include_proto!("sluice.v1");
    }
}

// Convenience re-exports for easier imports
pub use sluice::v1::*;
