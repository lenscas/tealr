use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{ImplItemMethod, ItemImpl};

pub(crate) fn document_teal(implement: ItemImpl) -> TokenStream {
    let (impl_generics, type_generics, where_clause) = implement.generics.split_for_impl();
    let mut add_function = None;
    let mut add_documentation = None;
    implement.items.into_iter().for_each(|v| match v {
        syn::ImplItem::Method(x) => match x.sig.ident.to_string().as_str() {
            "add_methods" => add_function = Some(x),
            "add_documentation" => add_documentation = Some(x),
            "add_docs_and_functions" => {
                panic!("add_docs_and_functions should not be implemented when using this macro.")
            }
            x => panic!("Function {} is not supported", x),
        },
        _ => panic!("the trait `TealData` should only contain methods."),
    });
    let trait_ = implement.trait_;
    let trait_ = match trait_ {
        Some((_, path, _)) => path,
        None => panic!("Trait implementation not complete"),
    };
    let type_ = implement.self_ty;
    let x = quote! {
        impl<#impl_generics>  #trait_ for #type_ #type_generics #where_clause {
            #add_function
            fn add_documentation<T: ::tealr::DocumentationCollector>(collector: &mut T) {
                collector.document_function(
                    "test",
                    "just a simple test",
                );
            }
        }
    };
    x.into()
}
fn get_documentation_from_func(
    function: Option<ImplItemMethod>,
) -> Option<(HashMap<String, String>, ImplItemMethod)> {
    let function = function?;
    function.block.stmts;
    None
}
