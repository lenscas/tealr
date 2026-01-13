use std::borrow::Cow;

use crate::{Field, FunctionParam, FunctionRepresentation, NameContainer, Type};

/// Teal supports [`macro expressions`](https://teal-language.org/book/macroexp.html). Basically functions that get inlined by the compiler
///
/// This type represents them, allowing you to add them to the type.
/// It offers an easy way to give type methods without having to actually attach them at runtime,
/// the downside being that they only work with teal.
///
/// the emitted syntax will look like `[metamethod] {name}: function[<{generics}>]({args}): {return} = macroexp({args}):{returns} {expr} end`
///
/// thus, a MacoExpr like
/// ```
/// use tealr::ToTypename;
/// let x = tealr::MacroExpr {
///     name: "example".to_string().into(),
///     expr: "return 1".into(),
///     signature: tealr::FunctionRepresentation {
///         params: vec![],
///         returns: vec![u8::to_typename()],
///     },
///     is_meta_method: false
/// };
/// ```
/// will emit
/// ` example: function(): integer = macroexp():integer return 1 end`
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[cfg_attr(
    feature = "self_to_lua",
    derive(crate::mlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    feature = "self_to_lua",
    tealr(tealr_name = crate)
)]
pub struct MacroExpr {
    ///the name of the macro expression
    pub name: NameContainer,
    ///this is used to create the signature
    pub signature: FunctionRepresentation,
    ///the expression. You have to add the `return` keyword yourself. The `end` is added automatically
    pub expr: String,
    ///wether the macro expression is used to define a metamethod or not
    pub is_meta_method: bool,
}

impl MacroExpr {
    ///Creates a new macro expression
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        args: Vec<Field>,
        returns: Type,
        expr: String,
        is_meta_method: bool,
    ) -> Self {
        let res = args
            .into_iter()
            .map(|v| FunctionParam {
                param_name: Some(v.name.to_string().into()),
                ty: v.ty,
            })
            .collect();
        Self {
            name: NameContainer::from(name.into()),
            signature: FunctionRepresentation {
                params: res,
                returns: vec![returns],
            },
            expr,
            is_meta_method,
        }
    }
    ///creates a new macroexpr that acts as a metamethod
    pub fn new_meta(
        name: impl Into<Cow<'static, str>>,
        args: Vec<Field>,
        returns: Type,
        expr: String,
    ) -> Self {
        Self::new(name, args, returns, expr, true)
    }
    ///creates a new macroexpr that acts as a normal method
    pub fn new_method(
        name: impl Into<Cow<'static, str>>,
        args: Vec<Field>,
        returns: Type,
        expr: String,
    ) -> Self {
        Self::new(name, args, returns, expr, false)
    }
}
