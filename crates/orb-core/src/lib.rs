//! Shared types, errors, configuration and paths for orb.

pub mod error;
pub mod ids;
pub mod media;
pub mod paths;

pub use error::{Error, Result};
pub use ids::{AssetId, EmbeddingId, FileId, PersonId, RootId, SegmentId};
pub use media::MediaKind;

/// Initialise tracing from `ORB_LOG` (falls back to `info`).
pub fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};
    let filter = EnvFilter::try_from_env("ORB_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = fmt().with_env_filter(filter).with_target(false).try_init();
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
