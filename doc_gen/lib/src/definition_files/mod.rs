//! This module contains the types and functions used to generate the definition file using lua templates
use tealr::mlu::mlua;
use tealr_doc_gen_core::{configuration::Paths, definition_files::DefinitionFileConfig};

use crate::{ETLUA, setup_lua, shared_globals};

/// The default template to create html pages.
#[cfg(feature = "default_templates")]
pub static TEAL_TEMPLATE: &str = include_str!("../../base_template_def_teal.etlua");

/// The fields in this type get exported to lua as separate globals. Read the lua API for more information
#[derive(Default)]
pub struct ShareWithLua<'a> {
    config: DefinitionFileConfig<'a>,
    etlua: &'a str,
    template: &'a str,
}

impl<'a> tealr::mlu::ExportInstances for ShareWithLua<'a> {
    fn add_instances<T: tealr::mlu::InstanceCollector>(
        self,
        instance_collector: &mut T,
    ) -> mlua::Result<()> {
        shared_globals(instance_collector, self.etlua, self.template)?;
        self.config.add_instances(instance_collector)
    }
}

fn run_template(
    etlua: &str,
    lua_runner: &str,
    template: &str,
) -> impl Fn(DefinitionFileConfig) -> Result<String, mlua::Error> {
    |v| {
        setup_lua(
            ShareWithLua {
                config: v,
                etlua,
                template,
            },
            lua_runner,
            "Create definition file",
        )
    }
}
/// Creates the definition file according to the given lua template, returning the result as a string
///
/// `lua_runner` is the lua code that will start the template. Refer to the lua api to see what globals this script gets given.
///
/// `template` is the template as a string. It gets passed to lua as the global "template"
///
/// When you want to use tealr_doc_gen_lib to generate a documentation site, prefer using [create_definition_file]
/// as it already puts the file in the right spot.
pub fn create_definition(
    paths: &Paths,
    type_walker: &tealr::TypeWalker,
    lua_runner: &str,
    template: &str,
) -> Result<String, tealr_doc_gen_core::definition_files::Error<mlua::Error>> {
    tealr_doc_gen_core::definition_files::create_definition(
        paths,
        type_walker,
        run_template(ETLUA, lua_runner, template),
    )
}
/// Creates the definition file according to the given lua template and stores it at the correct location
/// for the site to be able to link to it.
///
/// `lua_runner` is the lua code that will start the template. Refer to the lua api to see what globals this script gets given.
///
/// `template` is the template as a string. It gets passed to lua as the global "template"
///
/// `extension` is the file extension for the definition file. The name of the file will be equal to the name of the project ([Paths].name)
pub fn create_definition_file(
    paths: &Paths,
    type_walker: &tealr::TypeWalker,
    lua_runner: &str,
    template: &str,
    extension: &str,
) -> Result<(), tealr_doc_gen_core::definition_files::Error<mlua::Error>> {
    tealr_doc_gen_core::definition_files::create_def_file(
        paths,
        type_walker,
        extension,
        run_template(ETLUA, lua_runner, template),
    )
}
