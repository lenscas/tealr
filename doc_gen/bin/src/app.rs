use std::{collections::HashMap, fs::read_to_string, path::PathBuf};

use anyhow::Context;
use clap::{Arg, ArgGroup, Command};
use tealr_doc_gen_lib::{
    RUNNER,
    core::configuration::{DefTemplateConfig, DefTemplateKind},
    definition_files::{self, TEAL_TEMPLATE},
};

#[derive(serde::Serialize, serde::Deserialize, Clone, Default)]
pub enum TemplateKind {
    #[default]
    Builtin,
    FromTemplate(PathBuf),
}

#[derive(Clone)]
pub(crate) struct Config {
    pub(crate) json: PathBuf,
    pub(crate) name: String,
    pub(crate) root: String,
    pub(crate) build_dir: String,
    pub(crate) template_kind: TemplateKind,
    pub(crate) custom_html_runner: Option<PathBuf>,
    pub(crate) def_config: TypeDefFile,
    pub(crate) is_global: bool,
    pub(crate) lua_addon: Option<LuaAddon>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum DefTemplateRunnerKind {
    Builtin,
    Custom(PathBuf),
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct TypeDefFile {
    pub(crate) runner: DefTemplateRunnerKind,
    pub(crate) templates: HashMap<String, DefTemplateConfig>,
}

impl Default for TypeDefFile {
    fn default() -> Self {
        Self {
            runner: DefTemplateRunnerKind::Builtin,
            templates: {
                let mut files = HashMap::new();
                files.insert(
                    "teal".into(),
                    DefTemplateConfig {
                        extension: ".d.tl".into(),
                        template: DefTemplateKind::Teal,
                    },
                );
                files
            },
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum LuaAddon {
    False,
    Create {
        words: Vec<String>,
        files: Vec<String>,
        settings: HashMap<String, serde_json::Value>,
    },
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ConfigFile {
    doc_template: TemplateKind,
    page_root: String,
    store_in: String,
    name: String,
    type_def_files: TypeDefFile,
    lua_addon: Option<LuaAddon>,
    is_global: bool,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            doc_template: Default::default(),
            page_root: Default::default(),
            is_global: true,
            store_in: "pages".into(),
            name: "Your_API".into(),
            type_def_files: Default::default(),
            lua_addon: Some(LuaAddon::Create {
                words: Vec::new(),
                files: Vec::new(),
                settings: HashMap::new(),
            }),
        }
    }
}
pub(crate) enum Modes {
    Credits,
    GenerateDocs(Box<Config>),
    SelfDocTemplate {
        build_dir: String,
    },
    SelfDefTemplate {
        build_dir: String,
    },
    GenFile {
        file: String,
        location: &'static str,
    },
    Nothing,
}

pub(crate) fn get_paths() -> Result<Modes, anyhow::Error> {
    let matches =
        Command::new("tealr doc gen")
            .subcommand(
                Command::new("run")
                    .alias("gen")
                    .about("Generates the documentation pages"),
            )
            .subcommand(
                Command::new("gen-self")
                    .about("Generates files used to add custom behavior to tealr_doc_gen")
                    .arg(Arg::new("docs_documentation_template").long("docs-documentation-template").help(
                        "Generates the documentation for the lua api exposed to the template used to generate documentation pages.",
                    ).action(clap::ArgAction::SetTrue)).arg(
                        Arg::new("docs_definition_template")
                        .long("docs-definition-template")
                        .help("Generates the documentation for the lua api exposed to the template used to generate definition files.").action(clap::ArgAction::SetTrue)
                    )
                    .arg(
                        Arg::new("config")
                            .long("config")
                            .help("Generates a new config file")
                            .action(clap::ArgAction::SetTrue),
                    )
                    .arg(Arg::new("doc_template").long("doc-template").help(
                        "Generates the default template used to generate the documentation pages",
                    ).action(clap::ArgAction::SetTrue))
                    .arg(
                        Arg::new("lua_runner")
                            .long("lua-runner")
                            .help("Generate the lua code that loads and executes the template.").action(clap::ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("definition_template")
                            .long("definition-template")
                            .help("Generates the default template used to generate the teal definition file.").action(clap::ArgAction::SetTrue)
                    )
                    .arg(
                        Arg::new("print")
                            .long("print")
                            .short('p')
                            .help("Prints the file instead of writing it directly to a file").action(clap::ArgAction::SetTrue),
                    ).group(
                        ArgGroup::new("to_gen")
                        .args([
                            "docs_documentation_template",
                            "config",
                            "doc_template",
                            "lua_runner",
                            "definition_template",
                            "docs_definition_template"
                        ])
                        .required(true)
                    )
                        ,
            )
            .arg(
                Arg::new("credits")
                    .long("credits")
                    .short('c')
                    .help("Shows who worked on tealr_doc_gen"),
            )
            .get_matches();
    if matches.contains_id("credits") {
        return Ok(Modes::Credits);
    }

    if matches.subcommand_matches("run").is_some() {
        let config = read_config()?;
        return Ok(Modes::GenerateDocs(Box::new(Config {
            is_global: config.is_global,
            json: (config.name.clone() + ".json").into(),
            build_dir: config.store_in,
            name: config.name,
            root: config.page_root,
            template_kind: config.doc_template,
            def_config: config.type_def_files,
            lua_addon: config.lua_addon,
            custom_html_runner: None,
        })));
    }
    if let Some(x) = matches.subcommand_matches("gen-self") {
        if x.get_flag("docs_documentation_template") {
            let config = read_config()?;
            return Ok(Modes::SelfDocTemplate {
                build_dir: config.store_in,
            });
        } else if x.get_flag("docs_definition_template") {
            let config = read_config()?;
            return Ok(Modes::SelfDefTemplate {
                build_dir: config.store_in,
            });
        }
        let (text, location) = if x.get_flag("config") {
            (
                serde_json::to_string_pretty(&ConfigFile::default())?,
                "./tealr_doc_gen_config.json",
            )
        } else if x.get_flag("doc_template") {
            (
                definition_files::TEAL_TEMPLATE.to_string(),
                "./template.etlua",
            )
        } else if x.get_flag("definition_template") {
            (
                TEAL_TEMPLATE.to_string(),
                "./teal_definition_template.etlua",
            )
        } else if x.get_flag("lua_runner") {
            (RUNNER.to_string(), "./run_template.lua")
        } else {
            return Err(anyhow::anyhow!("Missing argument"));
        };
        if x.get_flag("print") {
            println!("{}", text);
            return Ok(Modes::Nothing);
        }
        return Ok(Modes::GenFile {
            file: text,
            location,
        });
    }
    Ok(Modes::Nothing)
}

fn read_config() -> Result<ConfigFile, anyhow::Error> {
    serde_json::from_str(
        &read_to_string("./tealr_doc_gen_config.json")
            .context("Could not read tealr_doc_gen_config.json in current directory. Maybe generate one using `tealr_doc_gen gen-self --config`?")?
    ).context("Error while parsing the config file. Use `tealr_doc_gen gen_self --config` to generate an example")
}
