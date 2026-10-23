use std::collections::HashSet;

use crate::{
    FunctionParam, FunctionRepresentation, Name, Type, get_generic_types, get_generics,
    type_generator::NameContainer,
};

///Contains the data needed to write down the type of a function
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(
    feature = "self_to_lua",
    derive(crate::mlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    feature = "self_to_lua",
    tealr(tealr_name = crate)
)]
pub struct ExportedFunction {
    /// Name of the function
    pub name: NameContainer,
    /// The parameters that this function requires
    pub params: Vec<FunctionParam>,
    /// The return type of the function
    pub returns: Vec<Type>,
    /// If this function is a meta_method
    pub is_meta_method: bool,
    /// If the main signature of the function should remain hidden from the documentation/definition files.
    ///
    /// If true, the last overload will be used instead
    ///
    /// When true without overloads, the function will not show up in either documentation or definition files.
    pub is_hidden: bool,
    /// Possible overloads for the given function
    pub overloads: Vec<FunctionRepresentation>,
}
impl ExportedFunction {
    /// Turns the exported function into just its type representation
    ///
    /// If the function is marked as hidden but has no overloads then the signature of the function will be used regardless
    pub fn into_type(&self) -> Type {
        self.get_all_as_types().next().unwrap_or_else(|| {
            Type::Function(crate::FunctionRepresentation {
                params: self.params.clone(),
                returns: self.returns.clone(),
            })
        })
    }
    /// Returns an iterator that has each signature (including overloads) of the function as a type
    pub fn get_all_as_types(&self) -> impl Iterator<Item = Type> {
        self.get_all_as_function_repr().map(Type::Function)
    }
    /// Returns an iterator that has each signature (including overloads) of the function as a [FunctionRepresentation]
    pub fn get_all_as_function_repr(&self) -> impl Iterator<Item = FunctionRepresentation> {
        (!self.is_hidden)
            .then(|| crate::FunctionRepresentation {
                params: self.params.clone(),
                returns: self.returns.clone(),
            })
            .into_iter()
            .chain(
                self.overloads
                    .iter()
                    .map(|v| crate::FunctionRepresentation {
                        params: v.params.clone(),
                        returns: v.returns.clone(),
                    }),
            )
    }
    ///Creates an ExportedFunction with the given name, Parameters and return value
    ///```no_run
    ///# use tealr::ExportedFunction;
    ///ExportedFunction::new::<(String,String),String,_>("concat",false,None);
    ///```
    pub fn new<A: crate::TealMultiValue, R: crate::TealMultiValue, S: AsRef<str>>(
        name: S,
        is_meta_method: bool,
        extra_self: Option<Type>,
    ) -> Self {
        let params = A::get_types_as_params();
        let params = if let Some(extra_self) = extra_self {
            let mut new_params = Vec::with_capacity(params.len() + 1);
            new_params.push(FunctionParam {
                param_name: Some("self".into()),
                ty: extra_self,
            });
            new_params.extend(params);
            new_params
        } else {
            params
        };

        Self {
            name: name.as_ref().as_bytes().to_vec().into(),
            is_meta_method,
            params,
            returns: R::get_types(),
            overloads: Vec::new(),
            is_hidden: false,
        }
    }

    ///Get all the generics that this function uses.
    pub fn get_generics(&self) -> HashSet<&Name> {
        self.params
            .iter()
            .map(|v| &v.ty)
            .chain(self.returns.iter())
            .flat_map(get_generics)
            .collect()
    }
    ///Get all the generics that this function uses.
    pub fn get_generic_types(&self) -> HashSet<Type> {
        self.params
            .iter()
            .map(|v| &v.ty)
            .chain(self.returns.iter())
            .flat_map(get_generic_types)
            .collect()
    }
}
