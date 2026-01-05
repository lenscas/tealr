use crate::{ToTypename, Type, TypeBody, TypeGenerator};

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
///Used to document what global instances get made by the module
#[cfg_attr(
    all(feature = "mlua", feature = "derive"),
    derive(crate::mlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    all(feature = "mlua", feature = "derive"),
    tealr(tealr_name = crate)
)]
pub struct GlobalInstance {
    ///the name of the instance
    pub name: String,
    ///the type
    pub ty: Type,
    ///documentation for this global
    pub doc: String,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
///Used to document what global instances get made by the module
#[cfg_attr(
    all(feature = "mlua", feature = "derive"),
    derive(crate::mlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    all(feature = "mlua", feature = "derive"),
    tealr(tealr_name = crate)
)]
pub struct ExtraPage {
    ///The name of the extra page
    pub name: String,
    ///The markdown content of the extra page.
    pub content: String,
}

///This generates the .d.tl files
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[cfg_attr(
    all(feature = "mlua", feature = "derive"),
    derive(crate::mlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    all(feature = "mlua", feature = "derive"),
    tealr(tealr_name = crate)
)]
pub struct TypeWalker {
    tealr_version_used: String,
    ///All the types that are currently registered by the TypeWalker
    pub given_types: Vec<TypeGenerator>,
    ///list of items that
    pub global_instances_off: Vec<GlobalInstance>,
    ///list of extra pages that need to be generated.
    pub extra_page: Vec<ExtraPage>,
}

impl Default for TypeWalker {
    fn default() -> Self {
        Self {
            tealr_version_used: crate::get_tealr_version().to_string(),
            given_types: Default::default(),
            global_instances_off: Default::default(),
            extra_page: Default::default(),
        }
    }
}

impl TypeWalker {
    ///creates the TypeWalker
    pub fn new() -> Self {
        Default::default()
    }
    ///Adds a new page that should be included in the documentation
    pub fn add_page(mut self, name: String, content: String) -> Self {
        self.extra_page.push(ExtraPage { name, content });
        self
    }
    ///reads a file and adds it as an extra page
    pub fn add_page_from(
        &mut self,
        name: String,
        location: impl AsRef<std::path::Path>,
    ) -> Result<&mut Self, std::io::Error> {
        let content = std::fs::read_to_string(location)?;
        self.extra_page.push(ExtraPage { name, content });
        Ok(self)
    }
    ///gives an iterator back over every type
    pub fn iter(&self) -> std::slice::Iter<'_, TypeGenerator> {
        self.given_types.iter()
    }
    ///Process a type such that the body will be added directly into the module instead of becoming a child record.
    ///
    ///When embedding teal/lua there is probably not really a reason to do so.
    ///However, it ***IS*** needed for the struct that gets exposed directly to teal when using mlua to make a lua/teal library.
    pub fn process_type_inline<A: ToTypename + TypeBody>(mut self) -> Self {
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
    pub fn process_type<A: ToTypename + TypeBody>(mut self) -> Self {
        let x = <A as TypeBody>::get_type_body();
        self.given_types.push(x);
        self
    }
    /// Generates the json needed by [tealr_doc_gen](https://crates.io/crates/tealr_doc_gen) to generate the documentation.
    ///
    /// It is up to you to store it properly
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
    /// Generates the json needed by [tealr_doc_gen](https://crates.io/crates/tealr_doc_gen) to generate the documentation in a pretty-printed way.
    ///
    /// It is up to you to store it properly.
    ///
    pub fn to_json_pretty(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
    /// Checks if the version of tealr to create this [TypeWalker] is the same version as the current [tealr](crate) version
    pub fn check_correct_version(&self) -> bool {
        self.tealr_version_used == crate::get_tealr_version()
    }
    /// Gets the version of [tealr](crate) that was used to create this [TypeWalker]
    pub fn get_tealr_version_used(&self) -> &str {
        &self.tealr_version_used
    }
}

impl TypeWalker {
    #[cfg(feature = "mlua")]
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
