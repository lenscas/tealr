use crate::{documentation_collector::HelpMethodGenerator, DocumentationCollector};

use super::TealDataMethods;

///This is the teal version of [UserData](mlua::UserData).
pub trait TealData: Sized {
    ///same as [UserData::add_methods](mlua::UserData::add_methods).
    ///Refer to its documentation on how to use it.
    ///
    ///only difference is that it takes a [TealDataMethods](crate::mlu::TealDataMethods),
    ///which is the teal version of [UserDataMethods](mlua::UserDataMethods)
    fn add_methods<'lua, T: TealDataMethods<'lua, Self>>(_methods: &mut T) {}
    ///Implement this to output documentation for this type.
    fn add_documentation<T: DocumentationCollector>(_collector: &mut T) {}
    ///the method used to generate both the documentation and register the functions
    ///generally spoken you don't need/want to overwrite it as the default behavior should suffice.
    fn add_methods_and_documentation<
        'lua,
        T: TealDataMethods<'lua, Self> + DocumentationCollector + HelpMethodGenerator,
    >(
        methods: &mut T,
    ) {
        Self::add_methods(methods);
        Self::add_documentation(methods);
        methods.generate_help();
    }
}
