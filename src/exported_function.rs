use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    string::FromUtf8Error,
};

use crate::{get_generics, type_generator::NameContainer, FunctionParam, Name, NamePart, Type};
#[allow(dead_code)]
type X = Vec<NamePart>;

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
    ///The full layout of the function based on teal's syntax
    #[deprecated]
    #[cfg_attr(
        all(feature = "derive", feature = "mlua"),
        tealr(remote = X)
    )]
    pub signature: Cow<'static, [crate::NamePart]>,
    ///The parameters that this function requires
    pub params: Vec<FunctionParam>,
    ///The return type of the function
    pub returns: Vec<Type>,
    ///If this function is a meta_method
    pub is_meta_method: bool,
}
impl ExportedFunction {
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
        use crate::FunctionRepresentation;
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
        let type_to_generate = Type::Function(FunctionRepresentation {
            params: params.clone(),
            returns: R::get_types(),
        });
        #[allow(deprecated)]
        let signature = crate::new_type_to_old(type_to_generate, false);
        #[allow(deprecated)]
        Self {
            name: name.as_ref().as_bytes().to_vec().into(),
            signature,
            is_meta_method,
            params,
            returns: R::get_types(),
        }
    }

    /// Give the [`params`] a name in case this is not automatically derived.
    ///
    /// # Panics
    ///
    /// Will panic if the given argument does not have the same number of fields as [`params`].
    pub fn name_parameters(
        &mut self,
        names: impl IntoIterator<IntoIter = impl ExactSizeIterator<Item = impl Into<Name>>>,
    ) -> &mut Self {
        let names = names.into_iter();
        assert_eq!(names.len(), self.params.len());
        for (name, p) in names.zip(self.params.iter_mut()) {
            p.param_name = Some(name.into());
        }
        self
    }

    pub(crate) fn generate(
        self,
        documentation: &HashMap<NameContainer, String>,
    ) -> std::result::Result<String, FromUtf8Error> {
        let documentation = match documentation.get(&self.name) {
            None => "".to_string(),
            Some(x) => x
                .lines()
                .map(|v| {
                    let mut str = "--".to_string();
                    str.push_str(v);
                    str.push('\n');
                    str
                })
                .collect(),
        };
        let metamethod = if self.is_meta_method {
            "metamethod "
        } else {
            ""
        };
        let name = String::from_utf8(self.name.0)?;
        let signature = crate::type_parts_to_str(
            #[allow(deprecated)]
            self.signature,
        );
        Ok(format!("{documentation}{metamethod}{name}: {signature}",))
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
}
