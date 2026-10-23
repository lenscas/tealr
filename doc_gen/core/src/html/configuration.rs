//! This module contains the types used to configure the html template.
use std::{collections::HashMap, path::PathBuf};

use tealr::{GlobalInstance, TypeGenerator};

use crate::{
    configuration::DefTemplateConfig,
    find_users::User,
    html::{CustomPage, IndexPage, sidebar::SideBar},
};

/// Everything needed to create a type page.
#[cfg_attr(
    feature = "lua_support",
    derive(tealr::mlu::FromToLua, tealr::ToTypename)
)]
#[derive(Clone)]
pub struct TypeDesc {
    pub type_members: TypeGenerator,
    pub type_name: String,
    pub used_by: Vec<User>,
}
/// The type or custom page that has to be turned into html.
#[cfg_attr(
    feature = "lua_support",
    derive(tealr::mlu::FromToLua, tealr::ToTypename)
)]
#[derive(Clone)]
pub enum TypeOrPage {
    /// A page for a type
    Type(TypeDesc),
    /// The index page
    IndexPage(IndexPage),
    /// A custom page. Just markdown that is not linked to a specific type
    CustomPage(CustomPage),
}

/// Everything that gets passed to the html template
pub struct PassToHtmlGenerator {
    /// Sidebar items
    pub sidebar: Vec<SideBar>,
    /// The page being generated
    pub page: TypeOrPage,
    /// the root path of the site.
    pub link_path: PathBuf,
    /// Every type in the project
    pub all_types: Vec<TypeGenerator>,
    /// All the globals that exist
    pub globals: Vec<GlobalInstance>,
    /// All the generated definition files
    pub def_files: HashMap<String, DefTemplateConfig>,
    /// The name of the project/library
    pub library_name: String,
}

impl Default for PassToHtmlGenerator {
    fn default() -> Self {
        Self {
            sidebar: Default::default(),
            page: TypeOrPage::CustomPage(CustomPage {
                name: "default".into(),
                markdown_content: "<h1>Default</h1>".into(),
            }),
            link_path: Default::default(),
            all_types: Default::default(),
            globals: Default::default(),
            def_files: Default::default(),
            library_name: Default::default(),
        }
    }
}

#[cfg(feature = "lua_support")]
impl tealr::mlu::ExportInstances for PassToHtmlGenerator {
    fn add_instances<T: tealr::mlu::InstanceCollector>(
        self,
        instance_collector: &mut T,
    ) -> tealr::mlu::mlua::Result<()> {
        instance_collector
            .add_instance("sidebar", |_| Ok(self.sidebar))?
            .add_instance("page", |_| Ok(self.page))?
            .add_instance("link_path", |_| Ok(self.link_path))?
            .add_instance("all_types", |_| Ok(self.all_types))?
            .add_instance("globals", |_| Ok(self.globals))?
            .add_instance("def_files", |_| Ok(self.def_files))?
            .add_instance("library_name", |_| Ok(self.library_name))?;
        Ok(())
    }
}
