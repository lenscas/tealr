use proc_macro2::{Ident, TokenStream};
use venial::{Declaration, Error};

use crate::from_to_lua::get_tealr_name;

pub(crate) fn impl_type_representation_derive(ast: &Declaration) -> proc_macro2::TokenStream {
    let name = ast.name();
    let tealr_name = get_tealr_name(ast.attributes());
    let gen = quote! {
        impl #tealr_name::TypeName for #name {
            fn get_type_parts() -> ::std::borrow::Cow<'static, [#tealr_name::NamePart]> {
                ::std::borrow::Cow::Borrowed(&[#tealr_name::NamePart::Type(#tealr_name::TealType{
                    name: ::std::borrow::Cow::Borrowed(stringify!(#name)),
                    generics: ::std::option::Option::None,
                    type_kind: #tealr_name::KindOfType::External
                })])
            }
        }
    };
    gen
}

fn generate_type_body(
    name: &Ident,
    traits: TokenStream,
    extra_method: Option<TokenStream>,
    tealr_name: &TokenStream,
) -> TokenStream {
    let extra_method = match extra_method {
        Some(x) => quote! {<Self as #traits>::#x(&mut gen);},
        None => quote!(),
    };
    quote! {
        impl #tealr_name::TypeBody for #name {
            fn get_type_body() -> #tealr_name::TypeGenerator {
                let mut gen = #tealr_name::RecordGenerator::new::<Self>(false);
                gen.is_user_data = true;
                #extra_method
                <Self as #traits>::add_methods(&mut gen);
                <_ as ::std::convert::From<_>>::from(gen)

            }
        }
    }
}

pub(crate) fn impl_rlua_user_data_derive(ast: &Declaration) -> proc_macro2::TokenStream {
    let tealr_name = get_tealr_name(ast.attributes());
    let name = match ast {
        Declaration::Struct(x) => &x.name,
        Declaration::Enum(x) => &x.name,
        _ => {
            return Error::new("As of right now, only structs and enums are supported.")
                .to_compile_error()
        }
    };
    let type_body =
        generate_type_body(name, quote! {#tealr_name::rlu::TealData}, None, &tealr_name);
    quote! {
        impl #tealr_name::rlu::rlua::UserData for #name {
            fn add_methods<'lua, T: #tealr_name::rlu::rlua::UserDataMethods<'lua, Self>>(methods: &mut T) {
                let mut x = #tealr_name::rlu::UserDataWrapper::from_user_data_methods(methods);
                <Self as #tealr_name::rlu::TealData>::add_methods(&mut x);
            }
        }
        #type_body
    }
}

pub(crate) fn impl_mlua_user_data_derive(ast: &Declaration) -> proc_macro2::TokenStream {
    let tealr_name = get_tealr_name(ast.attributes());
    let name = match ast {
        Declaration::Struct(x) => &x.name,
        Declaration::Enum(x) => &x.name,
        _ => {
            return Error::new("As of right now, only structs and enums are supported.")
                .to_compile_error()
        }
    };
    let type_body = generate_type_body(
        name,
        quote! {#tealr_name::mlu::TealData},
        Some(quote!(add_fields)),
        &tealr_name,
    );
    quote! {
        impl #tealr_name::mlu::mlua::UserData for #name {
            fn add_methods<'lua, T: #tealr_name::mlu::mlua::UserDataMethods<'lua, Self>>(methods: &mut T) {
                let mut x = #tealr_name::mlu::UserDataWrapper::from_user_data_methods(methods);
                <Self as #tealr_name::mlu::TealData>::add_methods(&mut x);
            }
            fn add_fields<'lua, F: #tealr_name::mlu::mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
                let mut wrapper = #tealr_name::mlu::UserDataWrapper::from_user_data_fields(fields);
                <Self as #tealr_name::mlu::TealData>::add_fields(&mut wrapper)
            }
        }
        #type_body
    }
}
