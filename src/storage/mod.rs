pub mod traits;
pub mod hash_index;
pub mod project_registry;
pub mod ref_counter;
pub mod deduplication;
pub mod fs_utils;
pub mod migration;

#[cfg(feature = "lancedb-backend")]
pub mod lancedb;
#[cfg(feature = "lancedb-backend")]
pub mod lancedb_impl;

#[cfg(feature = "ruvector")]
pub mod ruvector_impl;

pub use traits::*;
pub use hash_index::*;
pub use project_registry::*;
pub use ref_counter::*;
pub use deduplication::*;
pub use fs_utils::*;
pub use migration::*;

#[cfg(feature = "lancedb-backend")]
pub use lancedb_impl::LanceDbStorage;

#[cfg(feature = "ruvector")]
pub use ruvector_impl::RuVectorStorage;
