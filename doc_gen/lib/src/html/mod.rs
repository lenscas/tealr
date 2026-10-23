//! This module contains the types and functions that are used to generate the html files

use std::{collections::HashMap, path::PathBuf};

use tealr::{
    TypeGenerator,
    mlu::{
        TypedFunction,
        mlua::{self},
    },
    type_to_string,
};
use tealr_doc_gen_core::{
    SingleTypeNoConsume,
    configuration::DefTemplateConfig,
    html::{configuration::PassToHtmlGenerator, type_to_link_url},
};

use crate::{
    html::markdown::{
        MarkdownCodeBlockKindCreator, MarkdownEvent, MarkdownEventCreator, MarkdownTagCreator,
        MarkdownTagEndCreator, MarkdownTransformation, parse_markdown_lua,
    },
    setup_lua, shared_globals,
};

pub mod markdown;

/// The default template to create html pages.
#[cfg(feature = "default_templates")]
pub static TEAL_TEMPLATE: &str = include_str!("../../base_template_html.etlua");

/// The fields in this type get shared with lua, each field its own global. Refer to the lua documentation for more information
#[derive(Default)]
pub struct SharedWithLua {
    pub pass_through: PassToHtmlGenerator,
    pub definition_file_folder: PathBuf,
    pub template: String,
    pub template_lib: String,
    pub lua_runner: String,
    pub def_files: HashMap<String, DefTemplateConfig>,
}

impl tealr::mlu::ExportInstances for SharedWithLua {
    fn add_instances<T: tealr::mlu::InstanceCollector>(
        self,
        instance_collector: &mut T,
    ) -> mlua::Result<()> {
        let link_path = self.pass_through.link_path.clone();
        let link_path2 = self.pass_through.link_path.clone();
        let all_types2 = self.pass_through.all_types.clone();
        instance_collector.add_instance("type_to_link", move |lua| {
            let link = link_path2;
            let all_types = all_types2;

            TypedFunction::from_rust_mut(
                move |_, ty: SingleTypeNoConsume| {
                    let link = link.clone();
                    Ok(type_to_link_url(ty, &all_types, link))
                },
                lua,
            )
        })?;
        let all_types = self.pass_through.all_types.clone();
        instance_collector.add_instance("create_link", move |lua| {
            let link = link_path;
            let all_types = all_types;
            TypedFunction::from_rust(
                move |_, name: String| {
                    let (name, hash) = name
                        .find('#')
                        .map(|x| name.split_at(x))
                        .unwrap_or((&name, ""));
                    let is_index = all_types.iter().any(|x| match x {
                        TypeGenerator::Record(x) => {
                            x.should_be_inlined && type_to_string(&x.ty, false) == name
                        }
                        TypeGenerator::Enum(_) => false,
                    });
                    let name = if is_index { "index" } else { name };

                    Ok(link
                        .join(name.to_string() + ".html" + hash)
                        .to_string_lossy()
                        .to_string())
                },
                lua,
            )
        })?;
        let definition_files_folder = self.definition_file_folder;
        instance_collector
            .add_instance("definition_file_folder", |_| Ok(definition_files_folder))?;

        instance_collector.add_instance("markdown_codeblock_kind_creator", |_| {
            Ok(MarkdownCodeBlockKindCreator {})
        })?;
        instance_collector
            .add_instance("markdown_event_creator", |_| Ok(MarkdownEventCreator {}))?;
        instance_collector.add_instance("markdown_tag_creator", |_| Ok(MarkdownTagCreator {}))?;
        instance_collector
            .add_instance("markdown_tag_end_creator", |_| Ok(MarkdownTagEndCreator {}))?;

        let library_name = self.pass_through.library_name.clone();
        instance_collector.add_instance("library_name", |_| Ok(library_name))?;
        instance_collector.add_instance("definition_config", |_| Ok(self.def_files))?;

        instance_collector.add_instance("parse_markdown", |lua| {
            TypedFunction::from_rust(
                |_,
                 (markdown, func): (
                    String,
                    Option<TypedFunction<MarkdownEvent, MarkdownTransformation>>,
                )| match func {
                    Some(x) => parse_markdown_lua(
                        markdown,
                        |event| -> Result<Vec<MarkdownEvent>, tealr::mlu::mlua::Error> {
                            let z = x.call(event)?;
                            match z {
                                MarkdownTransformation::MarkdownEventTable(x) => Ok(x),
                                MarkdownTransformation::OptionalMarkdownEvent(Some(x)) => {
                                    Ok(vec![x])
                                }
                                MarkdownTransformation::OptionalMarkdownEvent(None) => {
                                    Ok(Default::default())
                                }
                            }
                        },
                    ),
                    None => parse_markdown_lua(markdown, |v| Ok(vec![v])),
                },
                lua,
            )
        })?;
        shared_globals(instance_collector, &self.template_lib, &self.template)?;
        self.pass_through.add_instances(instance_collector)?;
        Ok(())
    }
}

/// Creates the entire html site. The template runs once for each page.
///
/// `lua_runner` is the lua code that will start the template. This runs once per page, creating a single page in the process.
///
/// `template` is the template as a string. It gets passed to lua as the global "template"
pub fn create_html_site(
    paths: &tealr_doc_gen_core::configuration::Paths,
    type_walker: &tealr::TypeWalker,
    lua_runner: &str,
    template: &str,
) -> Result<
    std::collections::HashSet<tealr_doc_gen_core::html::WarningType>,
    tealr_doc_gen_core::html::Error<mlua::Error>,
> {
    let definition_file_folder = tealr_doc_gen_core::get_definition_file_path(paths);
    tealr_doc_gen_core::html::generate_html(paths, type_walker, |v| {
        let shared = SharedWithLua {
            pass_through: v,
            definition_file_folder: definition_file_folder.clone(),
            template: template.to_string(),
            template_lib: include_str!("../../etlua.lua").to_string(),
            lua_runner: lua_runner.to_string(),
            def_files: paths.definition_files.clone(),
        };
        setup_lua(shared, lua_runner, "Create html file")
    })
}
