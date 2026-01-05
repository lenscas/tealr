use std::collections::HashSet;

use crate::{
    get_generic_types, get_generics, type_generator::NameContainer, FunctionParam, Name, Type,
};

///Contains the data needed to write down the type of a function
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(
    all(feature = "derive", feature = "mlua"),
    derive(crate::mlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    all(feature = "derive", feature = "mlua"),
    tealr(tealr_name = crate)
)]
pub struct ExportedFunction {
    ///Name of the function
    pub name: NameContainer,
    ///The parameters that this function requires
    pub params: Vec<FunctionParam>,
    ///The return type of the function
    pub returns: Vec<Type>,
    ///If this function is a meta_method
    pub is_meta_method: bool,
}
impl ExportedFunction {
    ///turns the exported function into just its type representation
    pub fn into_type(&self) -> Type {
        Type::Function(crate::FunctionRepresentation {
            params: self.params.clone(),
            returns: self.returns.clone(),
        })
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
