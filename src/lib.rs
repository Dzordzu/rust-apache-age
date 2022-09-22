//! Connector to the [apache age](https://age.apache.org/) database
//! based on the postgres and tokio-postgres crates.
//!
//! Crate source is currently
//! [in the standalone repository](https://github.com/Dzordzu/rust-apache-age),
//! but will be (eventually) merged into the
//! [apache age repository](https://github.com/Dzordzu/rust-apache-age)
//!
//! ## Features
//!
//! * pure cypher query and execution
//! * field constraints creation
//! * (unique) indexes creation
//! * graph creation / drop
//! * graph existance checks
//! * postgres/tokio-postgres client usage
//! * query fields builders
//!
//! ## Sync and async client
//!
//! Both sync and async client have similar AgeClient traits. The only difference is
//! obviously within async method declarations.
//!
//! * In order to use sync client: `use apache_age::sync::{AgeClient, Client}`
//! * In order to use async client: `use apache_age::tokio::{AgeClient, Client}`
//!
//! ## Usage
//! ```
#![doc = include_str!("../examples/all.rs")]
//! ```
//!
//! ## Features
//!
//! | Name        | Description                                     | Default |
//! |-------------|-------------------------------------------------|---------|
//! | sync        | `postgres` based client                         | true    |
//! | tokio       | `tokio-postgres based client                    | true    |
//! | serializers | serializers that can be used for query building | false   |

#[macro_use]
mod constants;
mod age_types;

/// Used for synchronous age connection. Requires `sync` feature
#[cfg(feature = "sync")]
pub mod sync;

/// Used for the asynchronous age connection. Requires `tokio` feature
#[cfg(feature = "tokio")]
pub mod tokio;

/// Used for query builing . Requires `serializers` feature
#[cfg(feature = "serializers")]
pub mod serializers;

/// Used for the asynchronous age connection with results behind arc / arc mutex. Requires `tokio-arc` feature
#[cfg(feature = "tokio-sync-only")]
pub mod tokio_sync_only;

pub use age_types::{AgType, Edge, Vertex};
pub use postgres::NoTls;
