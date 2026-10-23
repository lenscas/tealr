//! This module contains everything you need if you want to generate the lua addon.
pub mod configuration;

use std::path::PathBuf;

use std::io::Write;
use tealr::{
    ExportedFunction, FunctionParam, FunctionRepresentation, MapRepresentation, RecordGenerator,
    Type, TypeWalker,
};
use zip::write::SimpleFileOptions;

use crate::lua_addon::configuration::LuaAddonConfig;
///Possible errors it can throw
#[derive(Debug)]
pub struct Error {
    /// Extra context for the error
    pub context: String,
    /// The actual error that it ran into
    pub kind: ErrorKind,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\nOriginal error:\n {}", self.context, self.kind)
    }
}
impl std::error::Error for Error {}
pub(crate) trait Context<E> {
    type Result;
    fn context(self, message: impl Into<String>) -> Result<Self::Result, E>;
}
impl<T, E: Into<ErrorKind>> Context<Error> for Result<T, E> {
    type Result = T;

    fn context(self, message: impl Into<String>) -> Result<Self::Result, Error> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => Err(Error {
                context: message.into(),
                kind: e.into(),
            }),
        }
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    /// IO error, likely not being able to write the file
    Io(std::io::Error),
    /// A failure while generating the `plugin.json` file
    Serde(serde_json::Error),
    /// A failure while zipping the addon up.
    Zip(zip::result::ZipError),
}
impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::Io(error) => error.fmt(f),
            ErrorKind::Serde(error) => error.fmt(f),
            ErrorKind::Zip(zip_error) => zip_error.fmt(f),
        }
    }
}

impl From<std::io::Error> for ErrorKind {
    fn from(value: std::io::Error) -> Self {
        ErrorKind::Io(value)
    }
}

impl From<serde_json::Error> for ErrorKind {
    fn from(value: serde_json::Error) -> Self {
        ErrorKind::Serde(value)
    }
}
impl From<zip::result::ZipError> for ErrorKind {
    fn from(value: zip::result::ZipError) -> Self {
        ErrorKind::Zip(value)
    }
}

/// Creates the lua addon, nicely zipped up and ready to be served through the site.
pub fn create_lua_addon(
    config: LuaAddonConfig,
    type_defs: &TypeWalker,
    mut path: PathBuf,
    is_global: bool,
) -> Result<bool, Error> {
    let zip_name = {
        let mut x = String::new();
        x += &config.name;
        x += ".zip";
        x
    };
    path.push(&zip_name);
    let x = std::fs::File::create(&path).context("Could not create zipfile for lua addon")?;
    let mut x = zip::write::ZipWriter::new(x);
    x.start_file("plugin.json", SimpleFileOptions::default())
        .context("Could not add plugin.json file to zip")?;
    x.write_all(
        serde_json::to_string_pretty(&config)
            .context("Could not create plugin.json")?
            .as_bytes(),
    )
    .context("Error while writing plugin.json")?;
    x.add_directory("library", SimpleFileOptions::default())
        .context("Could not create library folder")?;
    x.start_file(
        format!("library/{}.lua", &config.name),
        SimpleFileOptions::default(),
    )
    .context("Could not create library.lua file")?;
    let res = to_lua(&config.name, type_defs, is_global);
    x.write(res.as_bytes())
        .context("Failed writing library definition to library.lua")?;
    x.flush()
        .context("Could not write lua language server addon to disk")?;
    drop(x);
    Ok(true)
}

/// Generates _just_ the definitions file, returning it as a string.
pub fn to_lua(library_name: &str, type_defs: &TypeWalker, is_global: bool) -> String {
    let mut file = String::with_capacity(type_defs.given_types.len());
    file.push_str("---@meta\n\n");

    let mut base_fields = String::with_capacity(type_defs.given_types.len());
    let mut base_methods = String::with_capacity(type_defs.given_types.len());

    let mut classes = String::with_capacity(type_defs.given_types.len());
    let mut class_fields = String::new();
    let mut class_methods = String::new();

    let mut class_name = String::new();
    for ty in &type_defs.given_types {
        match ty {
            tealr::TypeGenerator::Record(x) => {
                if x.should_be_inlined {
                    write_record_as_class(
                        &mut base_methods,
                        &mut base_fields,
                        "",
                        library_name,
                        x,
                        false,
                    )
                } else {
                    class_fields.clear();
                    class_methods.clear();
                    class_name.clear();
                    //class_name.push_str(name);
                    //class_name.push('.');
                    class_name.push_str(&type_to_name(&x.ty, "", TypeIsPartOf::None));
                    write_record_as_class(
                        &mut class_methods,
                        &mut class_fields,
                        "",
                        &class_name,
                        x,
                        true,
                    );
                    classes += &class_fields;
                    classes += &class_methods;
                }
            }
            tealr::TypeGenerator::Enum(x) => {
                classes.push_str("---@alias ");
                //classes.push_str(name);
                //classes.push('.');
                classes.push_str(&type_to_name(&x.ty, "", TypeIsPartOf::None));
                classes.push('\n');
                for variant in &x.variants {
                    classes.push_str("---|'\"");
                    classes.push_str(&String::from_utf8_lossy(variant));
                    classes.push_str("\"'\n");
                }
            }
        }
    }
    let mut write_function_to = String::new();
    for global in &type_defs.global_instances_off {
        if let Type::Function(x) = &global.ty {
            write_function_to.clear();
            write_function(
                #[allow(deprecated)]
                &ExportedFunction {
                    name: global.name.clone().into(),
                    params: x.params.to_owned(),
                    returns: x.returns.to_owned(),
                    is_meta_method: false,
                    is_hidden: false,
                    overloads: Vec::new(),
                },
                "",
                "",
                &mut write_function_to,
                &mut classes,
            )
        } else {
            classes.push_str("---@type ");
            classes.push_str(&type_to_name(&global.ty, "", TypeIsPartOf::None));
            classes.push('\n');
            classes.push_str(&global.name);
            classes.push_str(" = nil");
            classes.push('\n');
        }
    }
    file.push_str(&base_fields);
    if !is_global {
        file.push_str("local ");
    }
    file.push_str(library_name);
    file.push_str(" = {}\n");
    file.push_str(&base_methods);
    file.push_str(&classes);
    file.push_str("\n return ");
    file.push_str(library_name);
    file
}

fn write_record_as_class(
    methods: &mut String,
    fields: &mut String,
    base: &str,
    class_name: &str,
    generator: &RecordGenerator,
    write_class: bool,
) {
    fields.reserve(generator.fields.len());
    methods.reserve(generator.methods.len());

    if write_class {
        if let Type::Single(ty) = &generator.ty {
            ty.generics.iter().for_each(|generic| {
                fields.push_str("---@diagnostic disable-next-line: duplicate-doc-alias\n");
                fields.push_str("---@alias ");
                fields.push_str(&type_to_name(generic, "", TypeIsPartOf::None));
                fields.push_str(
                    " any Temporary workaround for lls not having support for generic classes\n",
                );
            })
        }
        fields.push_str("---@class ");
        fields.push_str(class_name);
        if generator.is_user_data || !generator.implements.is_empty() {
            fields.push(':');
        }
        if generator.is_user_data {
            fields.push_str("userdata");
            if !generator.implements.is_empty() {
                fields.push_str(" , ");
            }
        }
        fields.push_str(
            &generator
                .implements
                .iter()
                .map(|v| type_to_name(v, "", TypeIsPartOf::None))
                .collect::<Vec<_>>()
                .join(" , "),
        );
        fields.push('\n');
    }
    if write_class {
        for field in &generator.fields {
            fields.push_str("---@field ");
            fields.push_str(&String::from_utf8_lossy(&field.name));
            fields.push(' ');
            fields.push_str(&type_to_name(&field.ty, base, TypeIsPartOf::None));
            if let Some(x) = generator.documentation.get(&field.name) {
                fields.push(' ');
                fields.push_str(&x.replace('\n', "<br>"));
            }
            fields.push('\n');
        }
    } else {
        for field in &generator.fields {
            methods.push_str("---@type ");
            methods.push_str(&type_to_name(&field.ty, base, TypeIsPartOf::None));
            if let Some(x) = generator.documentation.get(&field.name) {
                methods.push(' ');
                methods.push_str(&x.replace('\n', "<br>"));
            }
            methods.push('\n');
            methods.push_str(class_name);
            methods.push('.');
            methods.push_str(&String::from_utf8_lossy(&field.name));
            methods.push_str(" = nil\n");
        }
    }
    if write_class {
        methods.push_str("local ");
        methods.push_str(&type_to_name(&generator.ty, "", TypeIsPartOf::None));
        methods.push_str(" = {}\n");
    }
    let mut function_written_out = String::new();
    generator.all_functions().for_each(|function| {
        write_function(
            function,
            class_name,
            base,
            &mut function_written_out,
            methods,
        )
    });
}

fn get_param_name(param: &FunctionParam, key: usize) -> String {
    if let Type::Variadic(_) = param.ty {
        String::from("...")
    } else {
        param
            .param_name
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| format!("Param{}", key + 1))
    }
}

fn write_function(
    function: &ExportedFunction,
    class_name: &str,
    base: &str,
    function_written_out: &mut String,
    methods: &mut String,
) {
    for signature in function.get_all_as_function_repr() {
        let generics = signature.get_generics();

        for generic in generics {
            methods.push_str("---@generic ");
            methods.push_str(&generic.to_string());
            methods.push('\n');
        }
        function_written_out.clear();
        function_written_out.push_str("function ");
        if !class_name.is_empty() {
            function_written_out.push_str(class_name);
            function_written_out.push('.');
        }
        function_written_out.push_str(&String::from_utf8_lossy(&function.name));
        function_written_out.push('(');
        if !signature.params.is_empty() {
            for (key, param) in signature.params.iter().enumerate() {
                methods.push_str("---@param ");

                let param_name = get_param_name(param, key);
                methods.push_str(&param_name);
                function_written_out.push_str(&param_name);
                function_written_out.push(',');
                methods.push(' ');
                methods.push_str(&type_to_name(
                    &param.ty,
                    base,
                    TypeIsPartOf::FunctionParameter,
                ));

                methods.push('\n')
            }
            function_written_out.pop();
        }
        function_written_out.push_str(") end\n");
        for returned in &signature.returns {
            methods.push_str("---@return ");
            methods.push_str(&type_to_name(returned, base, TypeIsPartOf::FunctionReturn));
            methods.push('\n');
        }
        methods.push_str(function_written_out);
    }
}

#[derive(PartialEq, Eq)]
enum TypeIsPartOf {
    None,
    FunctionParameter,
    FunctionReturn,
}

fn type_to_name(ty: &Type, base: &str, part_off: TypeIsPartOf) -> String {
    match ty {
        Type::Function(FunctionRepresentation { params, returns }) => {
            let mut name = "fun(".to_string();
            if !params.is_empty() {
                for (key, param) in params.iter().enumerate() {
                    let param_name = get_param_name(param, key);
                    name += &param_name;
                    name += ":";
                    name += &type_to_name(&param.ty, base, TypeIsPartOf::FunctionParameter);
                    name += ",";
                }
                name.pop();
            }
            name += "):";
            if !returns.is_empty() {
                for returned in returns {
                    name += &type_to_name(returned, base, TypeIsPartOf::FunctionReturn);
                    name += ",";
                }
                name.pop();
            } else {
                name += "nil"
            }
            name
        }
        Type::Single(x) => {
            let name = x.name.to_string();
            let full_name = if x.kind.is_external() {
                let mut string = String::new();
                if !base.is_empty() {
                    string.push_str(base);
                    string.push('.');
                }
                string.push_str(&name);
                string
            } else {
                name
            };
            //a multiple return type tends to be written as `T...`
            //however, this is not how lua-language-server expects it. Then it should be `T ...`
            //so, we check for it following this format and then fixing it to be compatible with lua-language-server
            full_name
                .strip_suffix("...")
                .map(|v| v.trim_end().to_string() + " ...")
                .unwrap_or(full_name)
        }
        Type::Array(x) => {
            let x = type_to_name(x, base, TypeIsPartOf::None);
            format! {"({})[]",x}
        }
        Type::Map(MapRepresentation { key, value }) => {
            let key = type_to_name(key, base, TypeIsPartOf::None);
            let value = type_to_name(value, base, TypeIsPartOf::None);
            format!("{{ [{}]: {}}}", key, value)
        }
        Type::Or(x) => {
            let mut name = String::with_capacity(x.len() * 2 + 4);
            name.push_str("( ");
            for part in x.iter().map(|v| type_to_name(v, base, TypeIsPartOf::None)) {
                name += &part;
                name.push('|');
            }
            name.pop();
            name.push_str(" )");
            name
        }
        Type::Tuple(x) => {
            let mut name = String::with_capacity(x.len() * 2);
            for part in x.iter().map(|v| type_to_name(v, base, TypeIsPartOf::None)) {
                name += &part;
                name.push(',');
            }
            name.pop();
            name
        }
        Type::Variadic(x) => {
            let mut name = type_to_name(x, base, TypeIsPartOf::None);
            if part_off == TypeIsPartOf::FunctionReturn {
                name += " ...";
            }
            name
        }
    }
}
