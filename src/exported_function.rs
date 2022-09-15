use std::{borrow::Cow, collections::HashMap, string::FromUtf8Error};

use crate::{type_generator::NameContainer, NamePart};

#[cfg(any(feature = "rlua", feature = "mlua"))]
fn add_generics(v: &[crate::TealType], generics: &mut std::collections::HashSet<crate::NamePart>) {
    use crate::KindOfType;

    v.iter().for_each(|v| {
        let should_recurse = if v.type_kind == KindOfType::Generic {
            !generics.insert(crate::NamePart::Type(v.clone()))
        } else {
            true
        };
        if should_recurse {
            if let Some(x) = &v.generics {
                add_generics(x, generics)
            }
        }
    })
}

type X = Vec<NamePart>;

///Contains the data needed to write down the type of a function
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(
    all(feature = "mlua", feature = "derive", not(feature = "rlua")),
    derive(crate::mlu::FromToLua, crate::TypeName)
)]
#[cfg_attr(
    all(feature = "rlua", feature = "derive", not(feature = "mlua")),
    derive(crate::rlu::FromToLua, crate::TypeName)
)]
#[cfg_attr(
    all(any(feature = "rlua", feature = "mlua"), feature = "derive",not(all(feature = "rlua", feature = "mlua"))),
    tealr(tealr_name = crate)
)]
pub struct ExportedFunction {
    ///Name of the function
    pub name: NameContainer,
    ///The full signature of the function
    #[cfg_attr(
        all(any(feature = "rlua", feature = "mlua"), feature = "derive",not(all(feature = "rlua", feature = "mlua"))),
        tealr(remote = X)
    )]
    pub signature: Cow<'static, [crate::NamePart]>,
    ///If this function is a meta_method
    pub is_meta_method: bool,
}
impl ExportedFunction {
    ///Creates an ExportedFunction with the given name, Parameters and return value
    ///```no_run
    ///# use tealr::ExportedFunction;
    ///ExportedFunction::new::<(String,String),String,_>(b"concat",false,None);
    ///```
    #[cfg(any(feature = "rlua", feature = "mlua"))]
    pub fn new<A: crate::TealMultiValue, R: crate::TealMultiValue, S: AsRef<[u8]>>(
        name: S,
        is_meta_method: bool,
        extra_self: Option<Cow<'static, [crate::NamePart]>>,
    ) -> Self {
        use crate::KindOfType;
        use std::collections::HashSet;
        let mut generics = HashSet::new();
        let params2 = A::get_types();
        let contains_extra_params = !params2.is_empty();
        let params2 = params2.into_iter().inspect(|v| match v {
            NamePart::Symbol(_) => (),
            NamePart::Type(v) => {
                if v.type_kind == KindOfType::Generic {
                    generics.insert(NamePart::Type(v.to_owned()));
                }
                if let Some(x) = &v.generics {
                    add_generics(x, &mut generics)
                }
            }
        });
        let mut params = if let Some(x) = extra_self {
            let mut z = x.to_vec();
            if contains_extra_params {
                z.push(NamePart::Symbol(Cow::Borrowed(",")));
            }
            z
        } else {
            Vec::new()
        };
        params.extend(params2);
        let mut returns = R::get_types()
            .into_iter()
            .inspect(|v| match v {
                NamePart::Symbol(_) => (),
                NamePart::Type(v) => {
                    if v.type_kind == KindOfType::Generic {
                        generics.insert(NamePart::Type(v.to_owned()));
                    }
                    if let Some(x) = &v.generics {
                        add_generics(x, &mut generics)
                    }
                }
            })
            .collect();
        //ExportedFunction::new::<A, R>(name.as_ref().to_vec(), is_meta_method)
        let mut type_def = vec![NamePart::Symbol(Cow::Borrowed("function"))];
        if !generics.is_empty() {
            type_def.push("<".into());
            let iter = generics.into_iter();
            let iter =
                itertools::Itertools::intersperse(iter, NamePart::Symbol(Cow::Borrowed(",")));
            type_def.extend(iter);
            type_def.push(">".into());
        }
        type_def.push("(".into());
        let iter = params.into_iter();
        type_def.extend(iter);
        type_def.push("):(".into());
        type_def.append(&mut returns);
        type_def.push(")".into());
        let signature = Cow::Owned(type_def);
        Self {
            name: name.as_ref().to_vec().into(),
            signature,
            is_meta_method,
        }
    }
    pub(crate) fn generate(
        self,
        documentation: &HashMap<NameContainer, String>,
    ) -> std::result::Result<String, FromUtf8Error> {
        let documentation = match documentation.get(&self.name) {
            None => "".to_string(),
            Some(x) => x.lines().map(|v| format!("--{}\n", v)).collect(),
        };
        let metamethod = if self.is_meta_method {
            "metamethod "
        } else {
            ""
        };
        let name = String::from_utf8(self.name.0)?;
        let signature = crate::type_parts_to_str(self.signature);
        Ok(format!("{documentation}{metamethod}{name}: {signature}",))
    }
}
