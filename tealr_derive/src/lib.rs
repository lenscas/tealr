#![warn(missing_docs)]
//!# Tealr_derive
//!The derive macro used by [tealr](https://github.com/lenscas/tealr/tree/master/tealr).
//!
//!Tealr is a crate that can generate `.d.tl` files for types that are exposed to `lua`/`teal` through [rlua](https://crates.io/crates/rlua)
//!
//!Read the [README.md](https://github.com/lenscas/tealr/tree/master/tealr/README.md) in [tealr](https://github.com/lenscas/tealr/tree/master/tealr) for more information.

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

///Implements UserData.
///
///It wraps the UserDataMethods into tealr::UserDataWrapper
///and then passes it to tealr::TealData::add_methods.
#[proc_macro_derive(UserData)]
pub fn user_data_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_user_data_derive(&ast).into()
}

fn impl_user_data_derive(ast: &syn::DeriveInput) -> syn::export::TokenStream2 {
    let name = &ast.ident;
    let gen = quote! {
        impl UserData for #name {
            fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(methods: &mut T) {
                let mut x = tealr::UserDataWrapper::from_user_data_methods(methods);
                <Self as TealData>::add_methods(&mut x);
            }
        }
    };
    gen
}

///Implements TypeRepresentation.
///
///TypeRepresentation::get_type_name will return the name of the type.
#[proc_macro_derive(TypeRepresentation)]
pub fn type_representation_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_type_representation_derive(&ast).into()
}
fn impl_type_representation_derive(ast: &syn::DeriveInput) -> syn::export::TokenStream2 {
    let name = &ast.ident;
    let gen = quote! {
        impl TypeRepresentation for #name {
            fn get_type_name() -> std::borrow::Cow<'static, str> {
                stringify!(#name).into()
            }
        }
    };
    gen
}

///Implement both UserData and TypeRepresentation.
///
///Look at tealr_derive::UserData and tealr_derive::TypeRepresentation
///for more information on how the implemented traits will behave.
#[proc_macro_derive(TealDerive)]
pub fn teal_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let mut stream = impl_type_representation_derive(&ast);
    stream.extend(impl_user_data_derive(&ast));
    stream.into()
}
