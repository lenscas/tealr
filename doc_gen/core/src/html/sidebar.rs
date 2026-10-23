//! This module contains everything needed to generate the sidebar
//!
//! Aside from its types you are unlikely to need to interact with it.
use std::path::Path;

use tealr::{NameContainer, TypeWalker};

use crate::{configuration::Paths, html::type_to_link_url, type_should_be_inlined};

#[cfg_attr(
    feature = "lua_support",
    derive(tealr::mlu::FromToLua, tealr::ToTypename)
)]
#[derive(Clone)]
/// An element in the sidebar
pub struct SideBar {
    /// What url to link to
    pub link_to: String,
    /// Name of the type in the sidebar
    pub name: String,
    /// The members of the type
    pub members: Vec<Member>,
}
#[cfg_attr(
    feature = "lua_support",
    derive(tealr::mlu::FromToLua, tealr::ToTypename)
)]
#[derive(Clone)]
/// A member as shown in the sidebar
pub struct Member {
    /// The name of the member
    pub name: NameContainer,
}

pub fn generate_sidebar_data(
    type_defs: &TypeWalker,
    paths: &Paths,
    link_path: &Path,
) -> Vec<SideBar> {
    type_defs
        .given_types
        .iter()
        .map(|v| {
            let members = match v {
                tealr::TypeGenerator::Record(x) => x
                    .methods
                    .iter()
                    .map(|x| x.name.clone())
                    .chain(x.fields.iter().map(|x| x.name.clone()))
                    .chain(x.static_fields.iter().map(|x| x.name.clone()))
                    .chain(x.methods.iter().map(|x| x.name.clone()))
                    .map(|x| Member { name: x })
                    .collect(),
                tealr::TypeGenerator::Enum(x) => x
                    .variants
                    .iter()
                    .map(|v| Member { name: v.clone() })
                    .collect(),
            };
            let (name, link_to) = if type_should_be_inlined(v) {
                (paths.name.to_string(), link_path.join("index.html"))
            } else {
                let x = tealr::type_to_string(v.type_name(), false);
                let url = match v.type_name() {
                    tealr::Type::Single(single_type) => type_to_link_url(
                        single_type.to_owned(),
                        &type_defs.given_types,
                        link_path.to_path_buf(),
                    ),
                    _ => None,
                }
                .unwrap_or_else(|| {
                    link_path
                        .join(sanitize_filename::sanitize(&x))
                        .with_extension("html")
                });
                (x, url)
            };
            SideBar {
                link_to: link_path.join(link_to).to_string_lossy().into_owned(),
                name,
                members,
            }
        })
        .collect()
}
