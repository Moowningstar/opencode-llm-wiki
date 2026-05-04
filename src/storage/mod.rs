pub mod traits;

#[cfg(feature = "lancedb-backend")]
pub mod lancedb;
#[cfg(feature = "lancedb-backend")]
pub mod lancedb_impl;

#[cfg(feature = "ruvector")]
pub mod ruvector_impl;

pub use traits::*;

#[cfg(feature = "lancedb-backend")]
pub use lancedb_impl::LanceDbStorage;

#[cfg(feature = "ruvector")]
pub use ruvector_impl::RuVectorStorage;
