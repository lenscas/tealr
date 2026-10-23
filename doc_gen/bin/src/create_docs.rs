use std::path::Path;

use anyhow::Context;
use tealr::TypeWalker;
use tealr_doc_gen_lib::{
    RUNNER, core::lua_addon::configuration::LuaAddonConfig, definition_files, html,
};

use crate::app::Config;

fn load_runner(file: &Path) -> anyhow::Result<String> {
    std::fs::read_to_string(file)
        .with_context(|| format!("Failed reading supplied runner {}", file.display()))
}
fn load_template(file: &Path) -> anyhow::Result<String> {
    std::fs::read_to_string(file)
        .with_context(|| format!("Failed reading supplied runner {}", file.display()))
}

pub fn run_from_walker(type_walker: TypeWalker, config: Config) -> anyhow::Result<()> {
    let runner = match config.def_config.runner {
        crate::app::DefTemplateRunnerKind::Builtin => RUNNER.to_string(),
        crate::app::DefTemplateRunnerKind::Custom(x) => load_runner(&x)?,
    };
    let paths = tealr_doc_gen_lib::core::configuration::Paths {
        name: config.name,
        root: config.root,
        build_dir: config.build_dir,
        is_global: config.is_global,
        definition_files: config.def_config.templates,
    };
    tealr_doc_gen_lib::core::create_all_dirs(&paths)
        .context("Failed to create every needed directory")?;
    let definition_file_path = tealr_doc_gen_lib::core::get_definition_file_path(&paths);
    if let Some(crate::app::LuaAddon::Create {
        words,
        files,
        settings,
    }) = config.lua_addon
    {
        tealr_doc_gen_lib::core::lua_addon::create_lua_addon(
            LuaAddonConfig {
                name: paths.name.to_string(),
                words,
                files,
                settings,
            },
            &type_walker,
            definition_file_path,
            paths.is_global,
        )?;
    }
    for (name, definition_config) in paths.definition_files.iter() {
        let template = match &definition_config.template {
            tealr_doc_gen_lib::core::configuration::DefTemplateKind::Teal => {
                definition_files::TEAL_TEMPLATE.to_string()
            }
            tealr_doc_gen_lib::core::configuration::DefTemplateKind::Custom(x) => load_template(x)?,
        };
        tealr_doc_gen_lib::definition_files::create_definition_file(
            &paths,
            &type_walker,
            &runner,
            &template,
            &definition_config.extension,
        )
        .with_context(|| format!("Failed to create definition file for language: {name}"))?;
    }
    let template = match config.template_kind {
        crate::app::TemplateKind::Builtin => html::TEAL_TEMPLATE.to_string(),
        crate::app::TemplateKind::FromTemplate(x) => load_template(&x)?,
    };
    let html_runner = match config.custom_html_runner {
        None => RUNNER.to_string(),
        Some(x) => load_runner(&x)?,
    };
    let res =
        tealr_doc_gen_lib::html::create_html_site(&paths, &type_walker, &html_runner, &template)
            .context("Failed to create html site")?;
    for w in res {
        println!("{w}");
    }
    Ok(())
}

pub fn run_template(config: Config) -> anyhow::Result<()> {
    let type_walker = tealr_doc_gen_lib::core::load_typewalker_from_json(&config.json)
        .with_context(|| {
            format!(
                "Could not load provided types. File path: {}",
                config.json.display()
            )
        })?;
    run_from_walker(type_walker, config)
}
