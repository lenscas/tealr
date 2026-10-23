use std::collections::HashMap;

use serde::{Deserialize, Serialize};

///This is used to generate the plugin.json file.
///
/// Each field is a 1 to 1 mapping to the original plugin.json
#[derive(Serialize, Deserialize)]
pub struct LuaAddonConfig {
    pub name: String,
    pub words: Vec<String>,
    pub files: Vec<String>,
    pub settings: HashMap<String, serde_json::Value>,
}

pub enum LuaAddon {
    False,
    Create {
        words: Vec<String>,
        files: Vec<String>,
        settings: HashMap<String, serde_json::Value>,
    },
}
