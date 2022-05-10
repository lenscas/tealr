use proc_macro2::{Ident, TokenStream};
use venial::{Declaration, Error};

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

fn generate_type_body(
    name: &Ident,
    traits: TokenStream,
    extra_method: Option<TokenStream>,
) -> TokenStream {
    let extra_method = match extra_method {
        Some(x) => quote! {<Self as #traits>::#x(gen);},
        None => quote!(),
    };
    quote! {
        impl ::tealr::TypeBody for #name {
            fn get_type_body() -> ::tealr::TypeGenerator {
                let mut gen = ::tealr::RecordGenerator::new::<Self>(false);
                gen.is_user_data = true;
                #extra_method
                <Self as #traits>::add_methods(&mut gen);
                <_ as ::std::convert::From<_>>::from(gen)

            }
        }
    }
}

pub(crate) fn impl_rlua_user_data_derive(ast: &Declaration) -> proc_macro2::TokenStream {
    let name = match ast {
        Declaration::Struct(x) => &x.name,
        Declaration::Enum(x) => &x.name,
        _ => {
            return Error::new("As of right now, only structs and enums are supported.")
                .to_compile_error()
        }
    };
    let type_body = generate_type_body(name, quote! {::tealr::rlu::TealData}, None);
    quote! {
        impl rlua::UserData for #name {
            fn add_methods<'lua, T: ::tealr::rlu::rlua::UserDataMethods<'lua, Self>>(methods: &mut T) {
                let mut x = ::tealr::rlu::UserDataWrapper::from_user_data_methods(methods);
                <Self as ::tealr::rlu::TealData>::add_methods(&mut x);
            }
        }
        #type_body
    }
}

pub(crate) fn impl_mlua_user_data_derive(ast: &Declaration) -> proc_macro2::TokenStream {
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
        quote! {::tealr::mlu::TealData},
        Some(quote!(add_fields)),
    );
    quote! {
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
        #type_body
    }
}
