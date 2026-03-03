use std::borrow::Cow;

use crate::{Field, ToTypename};

pub trait TealDataMacros<T: ToTypename> {
    fn add_macro<R: ToTypename>(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        args: Vec<Field>,
        expr: impl Into<String>,
    ) -> &mut Self;
    fn add_meta_method_macro<R: ToTypename>(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        args: Vec<Field>,
        expr: impl Into<String>,
    ) -> &mut Self;
    fn document(&mut self, documentation: &str) -> &mut Self;
}
