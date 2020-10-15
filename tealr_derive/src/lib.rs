extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

#[proc_macro_derive(UserData)]
pub fn user_data_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_user_data_derive(&ast)
}
fn impl_user_data_derive(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl UserData for #name {
            fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(methods: &mut T) {
                let mut x = tealr::UserDataWrapper::from_user_data_methods(methods);
                <Self as TealData>::add_methods(&mut x);
            }
        }
    };
    gen.into()
}