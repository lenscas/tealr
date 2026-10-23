//! This module contains functions that check for possible problems with the exposed types.

mod missing_exports;

pub use missing_exports::{find_types_missing_export, warn_about_missing_exports};
