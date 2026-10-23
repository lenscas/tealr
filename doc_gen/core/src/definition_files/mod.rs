//! This module contains the types and functions that are used to generate the definition files.
use std::borrow::Cow;

use crate::{configuration::Paths, get_definition_file_path, lua_addon::Context};

/// This is passed to the template to generate the definition file
#[cfg_attr(feature = "lua_support", derive(Default))]
pub struct DefinitionFileConfig<'a> {
    /// Name of the project/library
    pub name: String,
    /// The entire module
    pub module: Cow<'a, tealr::TypeWalker>,
    /// Wether the exposed lua api gets directly put in the global namespace or if it stays local
    pub is_global: bool,
}

#[cfg(feature = "lua_support")]
impl<'a> tealr::mlu::ExportInstances for DefinitionFileConfig<'a> {
    fn add_instances<T: tealr::mlu::InstanceCollector>(
        self,
        instance_collector: &mut T,
    ) -> tealr::mlu::mlua::Result<()> {
        instance_collector
            .document_instance("The name of the project/library")
            .add_instance("name", |_| Ok(self.name))?
            .document_instance("Type and documentation of this module")
            .add_instance("module", |_| Ok(Cow::into_owned(self.module)))?
            .document_instance("Wether it is a local or global record that wraps the module")
            .add_instance("global_or_local", |_| {
                Ok(if self.is_global { "global" } else { "local" })
            })?;

        Ok(())
    }
}
/// Error type
#[derive(Debug)]
pub struct Error<E> {
    /// Extra context for the error
    context: String,
    /// The actual error
    kind: ErrorKind<E>,
}

impl<T, E, Err: Into<ErrorKind<E>>> Context<Error<E>> for Result<T, Err> {
    type Result = T;

    fn context(self, message: impl Into<String>) -> Result<Self::Result, Error<E>> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => Err(Error {
                context: message.into(),
                kind: e.into(),
            }),
        }
    }
}

impl<E: std::fmt::Display> std::fmt::Display for Error<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\nOriginal error:\n {}", self.context, self.kind)
    }
}
impl<E: std::error::Error> std::error::Error for Error<E> {}
#[derive(Debug)]
pub enum ErrorKind<E> {
    /// Problems while writing to the file
    Io(std::io::Error),
    /// Problems while running the template
    DuringTemplate(E),
}
impl<E: std::fmt::Display> std::fmt::Display for ErrorKind<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::Io(error) => error.fmt(f),
            ErrorKind::DuringTemplate(error) => error.fmt(f),
        }
    }
}

impl<E> From<std::io::Error> for ErrorKind<E> {
    fn from(value: std::io::Error) -> Self {
        ErrorKind::Io(value)
    }
}
impl<E: std::error::Error> std::error::Error for ErrorKind<E> {}
/// Creates the definition file, saving it at the correct spot
pub fn create_def_file<E>(
    paths: &Paths,
    type_walker: &tealr::TypeWalker,
    extension: &str,
    run_template: impl FnOnce(DefinitionFileConfig) -> Result<String, E>,
) -> Result<(), Error<E>> {
    let def_file = create_definition(paths, type_walker, run_template)?;
    let write_path = get_definition_file_path(paths);
    let extension = extension.strip_prefix(".").unwrap_or(extension);
    let name = &paths.name;
    let page_path = write_path.join(format!("{name}.{extension}"));
    std::fs::write(&page_path, def_file).context("Failed to write definition file")?;
    Ok(())
}
/// Same as [create_def_file], but returns it as a string _instead_ of writing it to a file
pub fn create_definition<E>(
    paths: &Paths,
    type_walker: &tealr::TypeWalker,
    create_definition: impl FnOnce(DefinitionFileConfig) -> Result<String, E>,
) -> Result<String, Error<E>> {
    create_definition(DefinitionFileConfig {
        name: paths.name.clone(),
        module: Cow::Borrowed(type_walker),
        is_global: paths.is_global,
    })
    .map_err(ErrorKind::DuringTemplate)
    .context("Could not generate definition file")
}
