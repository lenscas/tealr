#![cfg_attr(docsrs, feature(doc_cfg))]

//! A higher level version of [tealr_doc_gen_core]. It provides the means to create documentation and definition files using templates written in lua.
//! In addition, it also contains both a template that can be used to create the documentation site and one to create a definition file for teal.
//! tealr_doc_gen_lib also contains a basic "runner" in lua that kickstarts the template.

use tealr::{
    FunctionRepresentation, RecordGenerator, ToTypename, TypeGenerator,
    mlu::{
        self, ExportInstances, TypedFunction, UserDataProxy,
        mlua::{self, FromLua, IntoLua, UserDataRef},
    },
};
use tealr_doc_gen_core::find_all_generics;
/// This string contains the whole of the [etlua](https://github.com/leafo/etlua) library. It allows you to write templates in lua.
///
/// It will be passed to lua as the function "etlua", allowing a single function call to load this library
pub static ETLUA: &str = include_str!("../etlua.lua");
/// The basic runner. It loads etlua, defines a couple of helpers used by the default templates and runs the given template
pub static RUNNER: &str = include_str!("../base_run_template.lua");

#[cfg(feature = "definitions")]
pub mod definition_files;

#[cfg(feature = "html")]
pub mod html;
/// Re-exported from [tealr_doc_gen_core]
pub use tealr_doc_gen_core as core;

/// Used by [render_type] to render the given type. See [render_type] for more information
pub struct RenderOptions<X: ToTypename> {
    pub function: TypedFunction<(FunctionRepresentation, X), String>,
    pub single: TypedFunction<(tealr::SingleType, X), String>,
    pub map: TypedFunction<(tealr::MapRepresentation, X), String>,
    pub or: TypedFunction<(Vec<tealr::Type>, X), String>,
    pub array: TypedFunction<(tealr::Type, X), String>,
    pub tuple: TypedFunction<(Vec<tealr::Type>, X), String>,
    pub variadic: TypedFunction<(tealr::Type, X), String>,
}

impl<X: ToTypename> tealr::TypeBody for RenderOptions<X> {
    fn get_type_body() -> TypeGenerator {
        let mut record_generator = RecordGenerator::new::<RenderOptions<X>>(false);
        record_generator.document_type(
            "Used to tell [type_to_string](index#type_to_string) how to render each variant of a type",
        );
        record_generator
            .add_field::<_, TypedFunction<(FunctionRepresentation, X), String>>("[\"function\"]");
        record_generator.add_field::<_, TypedFunction<(tealr::SingleType, X), String>>("single");
        record_generator
            .add_field::<_, TypedFunction<(tealr::MapRepresentation, X), String>>("map");
        record_generator.add_field::<_, TypedFunction<(Vec<tealr::Type>, X), String>>("[\"or\"]");
        record_generator.add_field::<_, TypedFunction<(tealr::Type, X), String>>("array");
        record_generator.add_field::<_, TypedFunction<(Vec<tealr::Type>, X), String>>("tuple");
        record_generator.add_field::<_, TypedFunction<(tealr::Type, X), String>>("variadic");
        TypeGenerator::Record(Box::new(record_generator))
    }
}

impl<X: ToTypename + IntoLua> IntoLua for RenderOptions<X> {
    fn into_lua(
        self,
        lua: &tealr::mlu::mlua::Lua,
    ) -> tealr::mlu::mlua::Result<tealr::mlu::mlua::Value> {
        let table = lua.create_table()?;
        table.set("function", self.function)?;
        table.set("single", self.single)?;
        table.set("map", self.map)?;
        table.set("or", self.or)?;
        table.set("array", self.array)?;
        table.set("tuple", self.tuple)?;
        table.set("variadic", self.variadic)?;
        Ok(tealr::mlu::mlua::Value::Table(table))
    }
}
impl<X: ToTypename> ToTypename for RenderOptions<X> {
    fn to_typename() -> tealr::Type {
        tealr::Type::new_single_with_generics(
            "RenderOptions",
            tealr::KindOfType::External,
            vec![X::to_typename()],
        )
    }
}

impl<X: ToTypename + FromLua> FromLua for RenderOptions<X> {
    fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Self> {
        let table = mlua::Table::from_lua(value, lua)?;
        Ok(RenderOptions {
            function: table.get("function")?,
            single: table.get("single")?,
            map: table.get("map")?,
            or: table.get("or")?,
            array: table.get("array")?,
            tuple: table.get("tuple")?,
            variadic: table.get("variadic")?,
        })
    }
}
/// A function exposed to lua to render a type.
///
/// `options` contain the functions defined in lua that know how to render the various parts.
/// They will more than likely call `render_type` again recursively.
///
/// `extra_value` is custom data that can be passed between invocations, allowing you to keep track of what is currently
/// being worked on.
/// This is needed as at least in some languages, some types are written different when they are used as
/// parameters of a function or the return type.
///
/// An example of this in teal is [`mlua::Variadic<T>`](mlua::Variadic). When used as a parameter of a function it changes the name
/// of the parameter to `...`.
/// Meanwhile, when used as the return type it is `T...`. Written out you you get `function(... : T): T...`
pub fn render_type<X: ToTypename + IntoLua>(
    ty: &tealr::Type,
    options: RenderOptions<X>,
    extra_value: X,
) -> mlua::Result<String> {
    let res = match ty.clone() {
        tealr::Type::Function(function_representation) => options
            .function
            .call((function_representation, extra_value)),
        tealr::Type::Single(single_type) => options.single.call((single_type, extra_value)),
        tealr::Type::Map(map_representation) => options.map.call((map_representation, extra_value)),
        tealr::Type::Or(items) => options.or.call((items, extra_value)),
        tealr::Type::Array(items) => options.array.call((*items, extra_value)),
        tealr::Type::Tuple(items) => options.tuple.call((items, extra_value)),
        tealr::Type::Variadic(ty) => options.variadic.call((*ty, extra_value)),
    }?;
    Ok(res)
}

pub(crate) fn shared_globals<T: mlu::InstanceCollector>(
    instance_collector: &mut T,
    etlua: &str,
    template: &str,
) -> mlu::mlua::Result<()> {
    instance_collector.document_instance("function to load etlua");
    instance_collector.add_instance("etlua", |lua| {
        lua.load(etlua).set_name("etlua").into_function()
    })?;
    instance_collector.document_instance("template that gets loaded");
    instance_collector.add_instance("template", |_| Ok(template))?;

    instance_collector.add_instance("find_generics", |lua| {
        TypedFunction::from_rust(
            |_, ty: UserDataRef<tealr::Type>| Ok(find_all_generics(&ty)),
            lua,
        )
    })?;
    instance_collector.document_instance("Removes all duplicate instances of a table using a function to select what to look for. Returns a new table with the duplicates removed");
    instance_collector.document_instance("```teal_lua");
    instance_collector.document_instance("local with_dupes = {1,2,3,3,2,1}");
    instance_collector.document_instance(
        "local without_duplicates = dedupe_by(with_dupes,function(x:integer):integer return x end)",
    );
    instance_collector.document_instance("print(#without_duplicates, #with_dupes)");
    instance_collector.document_instance("```");
    instance_collector.add_instance("dedupe_by", |lua| {
        TypedFunction::from_rust(
            |lua,
             (a, b): (
                Vec<mlu::generics::X>,
                TypedFunction<mlu::generics::X, mlu::generics::Y>,
            )| {
                let table = lua.create_table()?;
                a.into_iter()
                    .filter_map(|x| match b.call(x.clone()) {
                        Ok(to_store) => match table.contains_key(to_store.clone()) {
                            Ok(true) => None,
                            Ok(false) => {
                                for v in table.pairs::<mlu::generics::Y, bool>() {
                                    let k = match v {
                                        Ok((v, _)) => v,
                                        Err(x) => return Some(Err(x)),
                                    };
                                    match k.equals(&to_store) {
                                        Ok(true) => {
                                            return None;
                                        }
                                        Err(x) => return Some(Err(x)),
                                        Ok(false) => {}
                                    };
                                }
                                match table.set(to_store, true) {
                                    Ok(()) => Some(Ok(x)),
                                    Err(x) => Some(Err(x)),
                                }
                            }
                            Err(x) => Some(Err(x)),
                        },
                        Err(x) => Some(Err(x)),
                    })
                    .collect::<Result<Vec<_>, _>>()
            },
            lua,
        )
    })?;
    instance_collector.add_instance("type_to_string", |lua| {
        TypedFunction::from_rust(
            |_,
             (ty, options, extra_value): (
                UserDataRef<tealr::Type>,
                RenderOptions<mlu::generics::X>,
                mlu::generics::X,
            )| render_type(&ty, options, extra_value),
            lua,
        )
    })?;
    instance_collector.add_instance("ty", UserDataProxy::<tealr::Type>::new)?;
    Ok(())
}

pub(crate) fn setup_lua<Config: ExportInstances>(
    a: Config,
    runner: &str,
    name: &str,
) -> mlua::Result<String> {
    let lua = unsafe { mlua::Lua::unsafe_new() };
    mlu::set_global_env(a, &lua)?;
    lua.load(runner).set_name(name).call(())
}
