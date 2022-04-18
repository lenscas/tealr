use proc_macro2::TokenStream;
use quote::ToTokens;
use venial::{Declaration, Error, Struct};

pub(crate) fn impl_type_representation_derive(ast: &Declaration) -> proc_macro2::TokenStream {
    let name = ast.name();
    let gen = quote! {
        impl ::tealr::TypeName for #name {
            fn get_type_parts() -> ::std::borrow::Cow<'static, [::tealr::NamePart]> {
                ::std::borrow::Cow::Borrowed(&[::tealr::NamePart::Type(::tealr::TealType{
                    name: ::std::borrow::Cow::Borrowed(stringify!(#name)),
                    generics: ::std::option::Option::None,
                    type_kind: ::tealr::KindOfType::External
                })])
            }
        }
    };
    gen
}

pub(crate) fn impl_rlua_user_data_derive(ast: Declaration) -> proc_macro2::TokenStream {
    let structure = match ast {
        Declaration::Struct(x) => x,
        _ => return Error::new("As of right now, only struts are supported.").to_compile_error(),
    };
    return quote! {};
    // let name = &structure.name;
    // let generics = structure
    //     .generic_params
    //     .map(|v| v.into_token_stream())
    //     .unwrap_or_else(|| quote! {});
    // let where_clause = structure
    //     .where_clause
    //     .map(|v| v.into_token_stream())
    //     .unwrap_or_else(|| quote! {});
    // let res = quote! {
    //     impl#generics rlua::UserData for #name#generics
    //     #where_clause
    //     {
    //         fn add_methods<'lua, T: ::tealr::rlu::rlua::UserDataMethods<'lua, Self>>(methods: &mut T) {
    //             let mut x = ::tealr::rlu::UserDataWrapper::from_user_data_methods(methods);
    //             <Self as ::tealr::rlu::TealData>::add_methods(&mut x);
    //         }
    //     }
    //     impl#generics ::tealr::TypeBody for #name#generics
    //     #where_clause
    //     {
    //         fn get_type_body(gen: &mut ::tealr::TypeGenerator) {
    //             gen.is_user_data = true;
    //             <Self as ::tealr::rlu::TealData>::add_methods(gen);
    //         }
    //     }
    // };
    // res
}

pub(crate) fn impl_mlua_user_data_derive(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl mlua::UserData for #name {
            fn add_methods<'lua, T: ::tealr::mlu::mlua::UserDataMethods<'lua, Self>>(methods: &mut T) {
                let mut x = ::tealr::mlu::UserDataWrapper::from_user_data_methods(methods);
                <Self as ::tealr::mlu::TealData>::add_methods(&mut x);
            }
            fn add_fields<'lua, F: ::tealr::mlu::mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
                let mut wrapper = ::tealr::mlu::UserDataWrapper::from_user_data_fields(fields);
                <Self as ::tealr::mlu::TealData>::add_fields(&mut wrapper)
            }
        }
        impl ::tealr::TypeBody for #name {
            fn get_type_body(gen: &mut ::tealr::TypeGenerator) {
                gen.is_user_data = true;
                <Self as ::tealr::mlu::TealData>::add_fields(gen);
                <Self as ::tealr::mlu::TealData>::add_methods(gen);

            }
        }
    };
    gen
}
