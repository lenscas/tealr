use std::{borrow::Cow, string::FromUtf8Error};

use crate::{type_parts_to_str, NamePart, TypeBody, TypeGenerator, TypeName};

type V = Vec<NamePart>;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
///Used to document what global instances get made by the module
#[cfg_attr(
    all(feature = "mlua", feature = "derive", not(feature = "rlua")),
    derive(crate::mlu::FromToLua, crate::TypeName)
)]
#[cfg_attr(
    all(feature = "rlua", feature = "derive", not(feature = "mlua")),
    derive(crate::rlu::FromToLua, crate::TypeName)
)]
#[cfg_attr(
    all(any(feature = "rlua", feature = "mlua"), feature = "derive", not(all(feature = "rlua", feature = "mlua"))),
    tealr(tealr_name = crate)
)]
pub struct GlobalInstance {
    ///name of the global
    #[cfg_attr(
        all(any(feature = "rlua", feature = "mlua"), feature = "derive", not(all(feature = "rlua", feature = "mlua"))),
        tealr(remote =  String))]
    pub name: Cow<'static, str>,
    ///the type
    #[cfg_attr(
        all(any(feature = "rlua", feature = "mlua"), feature = "derive", not(all(feature = "rlua", feature = "mlua"))),
        tealr(remote =  V))]
    pub teal_type: Cow<'static, [NamePart]>,
    ///if the type is external
    pub is_external: bool,
    ///documentation for this global
    pub doc: String,
}

///This generates the .d.tl files
#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
#[cfg_attr(
    all(feature = "mlua", feature = "derive", not(feature = "rlua")),
    derive(crate::mlu::FromToLua, crate::TypeName)
)]
#[cfg_attr(
    all(feature = "rlua", feature = "derive", not(feature = "mlua")),
    derive(crate::rlu::FromToLua, crate::TypeName)
)]
#[cfg_attr(
    all(any(feature = "rlua", feature = "mlua"), feature = "derive", not(all(feature = "rlua", feature = "mlua"))),
    tealr(tealr_name = crate)
)]
pub struct TypeWalker {
    ///All the types that are currently registered by the TypeWalker
    pub given_types: Vec<TypeGenerator>,
    ///list of items that
    pub global_instances_off: Vec<GlobalInstance>,
}

impl TypeWalker {
    ///creates the TypeWalker
    pub fn new() -> Self {
        Default::default()
    }
    ///gives an iterator back over every type
    pub fn iter(&self) -> std::slice::Iter<'_, TypeGenerator> {
        self.given_types.iter()
    }
    ///Process a type such that the body will be added directly into the module instead of becoming a child record.
    ///
    ///When embedding teal/lua there is probably not really a reason to do so.
    ///However, it ***IS*** needed for the struct that gets exposed directly to teal when using mlua to make a lua/teal library.
    pub fn process_type_inline<A: TypeName + TypeBody>(mut self) -> Self {
        let mut x = <A as TypeBody>::get_type_body();
        match &mut x {
            TypeGenerator::Record(x) => {
                x.should_be_inlined = true;
            }
            TypeGenerator::Enum(_) => (),
        }
        self.given_types.push(x);
        self
    }
    ///prepares a type to have a `.d.tl` file generated, and adds it to the list of types to generate.
    pub fn process_type<A: TypeName + TypeBody>(mut self) -> Self {
        let x = <A as TypeBody>::get_type_body();
        self.given_types.push(x);
        self
    }
    ///generates the `.d.tl` file. It outputs the string, its up to you to store it.
    #[cfg_attr(feature = "rlua", doc = " ```")]
    #[cfg_attr(not(feature = "rlua"), doc = " ```ignore")]
    ///# use rlua::{Lua, Result, UserDataMethods};
    ///# use tealr::{rlu::{TealData,UserData, TealDataMethods,UserDataWrapper}, TypeWalker, TypeName};
    ///#[derive(UserData,TypeName)]
    ///struct Example {}
    ///impl TealData for Example {}
    ///let generated_string = TypeWalker::new().process_type::<Example>().generate("Examples",true);
    ///assert_eq!(generated_string,Ok(String::from("global record Examples
    ///\trecord Example
    ///\t\tuserdata
    ///
    ///
    ///\tend
    ///end
    ///return Examples"
    ///)));
    ///```
    pub fn generate(
        self,
        outer_name: &str,
        is_global: bool,
    ) -> std::result::Result<String, FromUtf8Error> {
        let v: Vec<_> = self
            .given_types
            .into_iter()
            .map(|v| v.generate())
            .collect::<std::result::Result<_, _>>()?;
        let v = v.join("\n");
        let scope = if is_global { "global" } else { "local" };
        let global_instances = self
            .global_instances_off
            .into_iter()
            .map(|global| {
                let GlobalInstance {
                    name,
                    teal_type,
                    is_external,
                    doc,
                } = global;
                let doc = doc
                    .lines()
                    .map(|v| {
                        let mut add = String::from("--");
                        add.push_str(v);
                        add.push_str("\n\n");
                        add
                    })
                    .collect::<String>();
                let teal_type = type_parts_to_str(teal_type);
                let teal_type = if is_external {
                    format!("{outer_name}.{teal_type}")
                } else {
                    teal_type.to_string()
                };
                format!("{doc}global {name}: {teal_type}")
            })
            .collect::<Vec<_>>()
            .join("\n");
        let global_instances = if !global_instances.is_empty() {
            let mut x = String::from("\n");
            x.push_str(&global_instances);
            x
        } else {
            global_instances
        };
        Ok(format!(
            "{} record {name}\n{record}\nend{global_instances}\nreturn {name}",
            scope,
            name = outer_name,
            record = v
        ))
    }
    ///Same as calling [Typewalker::generate(outer_name,true)](crate::TypeWalker::generate).
    #[cfg_attr(feature = "rlua", doc = " ```")]
    #[cfg_attr(not(feature = "rlua"), doc = " ```ignore")]
    ///# use rlua::{Lua, Result, UserDataMethods};
    ///# use tealr::{rlu::{TealData, TealDataMethods,UserDataWrapper,UserData}, TypeWalker, TypeName};
    ///#[derive(UserData,TypeName)]
    ///struct Example {}
    ///impl TealData for Example {}
    ///let generated_string = TypeWalker::new().process_type::<Example>().generate_global("Examples");
    ///assert_eq!(generated_string,Ok(String::from("global record Examples
    ///\trecord Example
    ///\t\tuserdata
    ///
    ///
    ///\tend
    ///end
    ///return Examples"
    ///)));
    ///```
    pub fn generate_global(self, outer_name: &str) -> std::result::Result<String, FromUtf8Error> {
        self.generate(outer_name, true)
    }
    ///Same as calling [Typewalker::generate(outer_name,false)](crate::TypeWalker::generate).
    #[cfg_attr(feature = "rlua", doc = " ```")]
    #[cfg_attr(not(feature = "rlua"), doc = " ```ignore")]
    ///# use rlua::{Lua, Result, UserDataMethods};
    ///# use tealr::{rlu::{TealData, TealDataMethods,UserDataWrapper,UserData,}, TypeWalker, TypeName};
    ///#[derive(UserData,TypeName)]
    ///struct Example {}
    ///impl TealData for Example {}
    ///let generated_string = TypeWalker::new().process_type::<Example>().generate_local("Examples");
    ///assert_eq!(generated_string,Ok(String::from("local record Examples
    ///\trecord Example
    ///\t\tuserdata
    ///
    ///
    ///\tend
    ///end
    ///return Examples"
    ///)));
    ///```
    pub fn generate_local(self, outer_name: &str) -> std::result::Result<String, FromUtf8Error> {
        self.generate(outer_name, false)
    }
}

#[cfg(feature = "rlua")]
impl TypeWalker {
    ///collect every instance that is getting shared with lua
    pub fn document_global_instance<T: crate::rlu::ExportInstances>(
        mut self,
    ) -> rlua::Result<Self> {
        let mut collector = crate::export_instance::InstanceWalker::new();
        T::default().add_instances(&mut collector)?;
        self.global_instances_off.append(&mut collector.instances);
        Ok(self)
    }
}

#[cfg(feature = "mlua")]
impl TypeWalker {
    ///collect every instance that is getting shared with lua
    pub fn document_global_instance<T: crate::mlu::ExportInstances>(
        mut self,
    ) -> mlua::Result<Self> {
        let mut collector = crate::export_instance::InstanceWalker::new();
        T::default().add_instances(&mut collector)?;
        self.global_instances_off.append(&mut collector.instances);
        Ok(self)
    }
}
