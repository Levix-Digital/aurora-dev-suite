//! The core of Aurora Dev Suite: the domain, the use cases and the ports every engine
//! and surface goes through (ADR-0001). It does no I/O and compiles to WebAssembly, so
//! the interface shares its types.

pub mod capabilities;
pub mod domain;
pub mod ports;
pub mod usecases;

#[cfg(feature = "fixtures")]
pub mod fixtures;
