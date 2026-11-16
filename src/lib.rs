//! `rtree` provides the traversal and rendering engine behind the command-line tool.
//!
//! It is intentionally usable as a library so applications can build or render trees
//! without shelling out to the binary.

mod cli;
mod error;
mod model;
mod render;

pub use cli::Cli;
pub use error::Error;
pub use model::{Node, NodeKind, SortBy, Tree, TreeOptions};
