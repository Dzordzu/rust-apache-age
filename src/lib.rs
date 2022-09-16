#![doc = include_str!("docs.md")]

#[macro_use]
mod constants;
mod age_types;

#[cfg(feature = "tokio")]
pub mod tokio;

#[cfg(feature = "sync")]
pub mod sync;

pub use postgres::NoTls;
pub use age_types::{Edge, Vertex, AgType};
