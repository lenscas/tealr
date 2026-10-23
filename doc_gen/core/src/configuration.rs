use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

/// Configuration for a definition file.
///
/// Use `Teal` for the default, teal definition file
///
/// Otherwise use `Custom(PathToLuaFile)`
#[cfg_attr(
    feature = "lua_support",
    derive(tealr::mlu::FromToLua, tealr::ToTypename)
)]
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DefTemplateKind {
    Teal,
    Custom(PathBuf),
}

/// The configuration for the definition files that get generated.
///
/// Contain the information needed to link to them
#[cfg_attr(
    feature = "lua_support",
    derive(tealr::mlu::FromToLua, tealr::ToTypename)
)]
#[derive(Clone, Serialize, Deserialize)]
pub struct DefTemplateConfig {
    /// File extension used
    pub extension: String,
    /// template used
    pub template: DefTemplateKind,
}

/// Main configuration
#[derive(Clone)]
pub struct Paths {
    /// Name of the library/project you want to document.
    /// This ends up as the main type in the definition file
    pub name: String,
    /// For html, what the "root" path is. This is used to generate links
    pub root: String,
    /// What folder to use as the output.
    pub build_dir: String,
    /// If the exposed lua API is self contained or gets put in the global namespace
    ///
    /// Under normal circumstances, when embedding Lua into a rust program this should be true
    /// and false when creating a lua module.
    pub is_global: bool,
    /// Configuration for the definition files
    ///
    /// The key is the language name, the key the configuration
    pub definition_files: HashMap<String, DefTemplateConfig>,
}
