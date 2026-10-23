//! This module contains everything you need if you want to generate the html site.

use std::{
    collections::HashSet,
    fmt::Display,
    path::{Path, PathBuf},
    sync::Mutex,
};

use tealr::{Name, RecordGenerator, TypeGenerator, TypeWalker};

use crate::{
    SingleTypeNoConsume,
    configuration::Paths,
    find_users::find_users,
    html::{
        configuration::{PassToHtmlGenerator, TypeDesc, TypeOrPage},
        sidebar::{SideBar, generate_sidebar_data},
    },
    is_type_compatible_helper, type_should_be_inlined,
};

pub mod configuration;
pub mod sidebar;

/// When generating links there can be some additional problems found.
/// This enum is used to track them
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum WarningType {
    /// A link was made that points to an unknown type
    UnknownType(Name),
    /// A link was made that points to multiple possible types
    MultipleTypes(Name),
}

impl Display for WarningType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WarningType::UnknownType(name) => {
                write!(f, "Required type `{name}`, but this type was not found")
            }
            WarningType::MultipleTypes(name) => write!(
                f,
                "`{name}` was exported multiple times. Ensure each type has a unique name and is only added to the typewalker once"
            ),
        }
    }
}

static EMITTED_WARNINGS: std::sync::LazyLock<Mutex<HashSet<WarningType>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashSet::new()));

fn reset_emitted_warnings() -> HashSet<WarningType> {
    let mut old = EMITTED_WARNINGS.lock().unwrap();
    let old: &mut HashSet<_> = &mut old;
    std::mem::take(old)
}

/// Creates a valid link to a given type, using `all_types` to ensure it is valid
pub fn type_to_link_url(
    ty: impl Into<SingleTypeNoConsume>,
    all_types: &[TypeGenerator],
    mut link_path: PathBuf,
) -> Option<PathBuf> {
    let ty = ty.into();
    let mut x = all_types
        .iter()
        .filter_map(|v| match v.type_name() {
            tealr::Type::Single(x) => Some((x, v)),
            _ => None,
        })
        .filter(|(v, _)| is_type_compatible_helper(v, &ty))
        .take(2);
    let Some((found_type, generator)) = x.next() else {
        if ty.name == Name::from("index") {
            return Some(link_path);
        }
        if EMITTED_WARNINGS
            .lock()
            .unwrap()
            .insert(WarningType::UnknownType(ty.name.clone()))
        {
            eprintln!("Tried making link to unknown type: {}", ty.name);
        }

        return None;
    };
    if x.next().is_some()
        && EMITTED_WARNINGS
            .lock()
            .unwrap()
            .insert(WarningType::MultipleTypes(found_type.name.clone()))
    {
        eprintln!("Multiple types with the same name: {}", found_type.name);
    }
    let name = if generator.is_inlined() {
        "index".to_string()
    } else {
        found_type.name.to_string()
    };
    link_path.push(name);
    link_path.set_extension("html");
    Some(link_path)
}

/// Contains everything needed to generate an index page.
#[cfg_attr(
    feature = "lua_support",
    derive(tealr::mlu::FromToLua, tealr::ToTypename)
)]
#[derive(Clone)]
pub struct IndexPage {
    pub all_types: Vec<TypeGenerator>,
    pub type_name: String,
    pub type_members: TypeGenerator,
}

/// Contains everything needed to generate a custom page
///
/// Custom pages are generated using markdown and do not represent a type
#[cfg_attr(
    feature = "lua_support",
    derive(tealr::mlu::FromToLua, tealr::ToTypename)
)]
#[derive(Clone)]
pub struct CustomPage {
    pub name: String,
    pub markdown_content: String,
}
/// Errors that can show up while generating the html
#[derive(Debug)]
pub enum Error<E> {
    /// A custom error the template can throw
    Custom(E),
    /// Something went wrong while writing the file
    File(std::io::Error),
}
impl<E: std::fmt::Display> std::fmt::Display for Error<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Custom(x) => write!(f, "{}", x),
            Error::File(x) => write!(f, "{}", x),
        }
    }
}

impl<E: std::fmt::Display + std::fmt::Debug> std::error::Error for Error<E> {}

/// Generates all the html files
pub fn generate_html<E>(
    paths: &Paths,
    types: &TypeWalker,
    mut function: impl FnMut(PassToHtmlGenerator) -> Result<String, E>,
) -> Result<HashSet<WarningType>, Error<E>> {
    let link_path = Path::new("/").join(&paths.root);
    let mut index_page = RecordGenerator {
        should_be_inlined: true,
        is_user_data: false,
        ty: tealr::Type::Single(tealr::SingleType {
            name: Default::default(),
            kind: tealr::KindOfType::External,
            generics: Default::default(),
        }),
        fields: Default::default(),
        static_fields: Default::default(),
        methods: Default::default(),
        documentation: Default::default(),
        type_doc: Default::default(),
        next_docs: Default::default(),
        should_generate_help_method: Default::default(),
        tag: Default::default(),
        macro_expressions: Default::default(),
        implements: Default::default(),
        next_overloads: Default::default(),
        next_is_hidden: Default::default(),
    };
    let sidebar = generate_sidebar_data(types, paths, &link_path);
    let write_path = Path::new(&paths.build_dir).join(&paths.root);
    for type_def in types.iter() {
        if let TypeGenerator::Record(x) = type_def
            && x.should_be_inlined
        {
            let x = x.to_owned();
            index_page.documentation.extend(x.documentation);
            index_page.fields.extend(x.fields);
            index_page.methods.extend(x.methods);
            index_page.static_fields.extend(x.static_fields);
            index_page.type_doc += &x.type_doc;
            continue;
        }
        write_type(
            &write_path,
            type_def,
            sidebar.clone(),
            types,
            link_path.clone(),
            paths,
            false,
            &mut function,
        )?;
    }
    write_type(
        &write_path,
        &TypeGenerator::Record(Box::new(index_page)),
        sidebar.clone(),
        types,
        link_path.clone(),
        paths,
        true,
        &mut function,
    )?;
    for v in types.extra_page.iter() {
        write_page(
            &write_path,
            PassToHtmlGenerator {
                sidebar: sidebar.clone(),
                page: TypeOrPage::CustomPage(CustomPage {
                    name: v.name.clone(),
                    markdown_content: v.content.clone(),
                }),
                link_path: link_path.clone(),
                all_types: types.given_types.clone(),
                globals: types.global_instances_off.clone(),
                def_files: paths.definition_files.clone(),
                library_name: paths.name.clone(),
            },
            &mut function,
        )?;
    }
    Ok(reset_emitted_warnings())
}

#[allow(clippy::too_many_arguments)]
fn write_type<E>(
    write_path: &Path,
    type_def: &TypeGenerator,
    sidebar: Vec<SideBar>,
    types: &TypeWalker,
    link_path: PathBuf,
    paths: &Paths,
    is_index: bool,
    func: &mut impl FnMut(PassToHtmlGenerator) -> Result<String, E>,
) -> Result<(), Error<E>> {
    let type_name = if type_should_be_inlined(type_def) {
        "index".into()
    } else {
        tealr::type_to_string(type_def.type_name(), false)
    };
    let users = find_users(type_def, types);
    let page = if is_index {
        TypeOrPage::IndexPage(IndexPage {
            all_types: types.given_types.to_owned(),
            type_name,
            type_members: type_def.to_owned(),
        })
    } else {
        TypeOrPage::Type(TypeDesc {
            type_members: type_def.to_owned(),
            type_name,
            used_by: users,
        })
    };
    let page_data = PassToHtmlGenerator {
        sidebar,
        page,
        link_path,
        all_types: types.given_types.to_owned(),
        globals: types.global_instances_off.to_owned(),
        def_files: paths.definition_files.to_owned(),
        library_name: paths.name.clone(),
    };
    write_page(write_path, page_data, func)?;
    Ok(())
}

fn write_page<E>(
    write_path: &Path,
    pass_through: PassToHtmlGenerator,
    func: &mut impl FnMut(PassToHtmlGenerator) -> Result<String, E>,
) -> Result<(), Error<E>> {
    let page_path = match &pass_through.page {
        TypeOrPage::Type(type_desc) => match type_desc.type_members.type_name() {
            tealr::Type::Single(single_type) => type_to_link_url(
                single_type.to_owned(),
                &pass_through.all_types,
                write_path.to_owned(),
            )
            .unwrap_or_else(|| write_path.join(sanitize_filename::sanitize(&type_desc.type_name))),
            tealr::Type::Function(_)
            | tealr::Type::Map(_)
            | tealr::Type::Or(_)
            | tealr::Type::Array(_)
            | tealr::Type::Tuple(_)
            | tealr::Type::Variadic(_) => {
                write_path.join(sanitize_filename::sanitize(&type_desc.type_name))
            }
        },
        TypeOrPage::IndexPage(index_page) => {
            write_path.join(sanitize_filename::sanitize(&index_page.type_name))
        }
        TypeOrPage::CustomPage(custom_page) => {
            write_path.join(sanitize_filename::sanitize(&custom_page.name))
        }
    }
    .with_extension("html");
    let page = match func(pass_through) {
        Ok(x) => x,
        Err(x) => {
            reset_emitted_warnings();
            return Err(Error::Custom(x));
        }
    };
    let as_bytes = page.as_bytes();
    let mut minify_cfg = minify_html::Cfg::new();
    minify_cfg.minify_css = true;
    let as_bytes = minify_html::minify(as_bytes, &minify_cfg);
    match std::fs::write(&page_path, as_bytes) {
        Ok(_) => Ok(()),
        Err(x) => {
            reset_emitted_warnings();
            Err(Error::File(x))
        }
    }
}
