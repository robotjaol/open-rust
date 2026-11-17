//! Platform abstraction layer: where files live, what the host looks like, how to talk to
//! native dialogs, and a small filesystem service that applies the security policy.

pub mod dialogs;
pub mod fs;
pub mod info;
pub mod paths;

pub use fs::{FileEntry, FileService};
pub use info::PlatformInfo;
pub use paths::AppPaths;
