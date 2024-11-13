use proc_macro2::{Literal, Span, TokenStream};
use quote::ToTokens;
use venial::{parse_item, Struct};

pub(crate) fn get_tealr_name(attributes: &[venial::Attribute]) -> TokenStream {
    find_tag_with_value("tealr_name", attributes)
        .map(Into::into)
        .unwrap_or_else(|| quote!(::tealr))
}

#[allow(dead_code)]
#[cfg(feature = "debug_macros")]
fn debug_macro(ts: TokenStream) -> TokenStream {
    let hopefully_unique = {
        use ::std::hash::*;
        let hasher = &mut RandomState::new().build_hasher();
        hasher.finish()
    };

    //FEEL FREE TO TWEAK THIS DEFAULT PATH (e.g., your target dir)
    let mut debug_macros_dir = ::std::path::PathBuf::from("/tmp");
    std::fs::create_dir_all(&debug_macros_dir).unwrap();
    let file_name = &{
        debug_macros_dir.push(format!("{:016x}.rs", hopefully_unique));
        debug_macros_dir.into_os_string().into_string().unwrap()
    };
    std::fs::write(file_name, ts.to_string()).unwrap();
    quote!(::core::include! { #file_name })
}

#[allow(dead_code)]
#[cfg(not(feature = "debug_macros"))]
fn debug_macro(ts: TokenStream) -> TokenStream {
    ts
}

fn find_tag_with_value(to_find: &str, tags: &[venial::Attribute]) -> Option<TokenStream> {
    tags.iter()
        .find(|v| v.path.iter().cloned().collect::<TokenStream>().to_string() == "tealr")
        .and_then(|v| match &v.value {
            venial::AttributeValue::Empty => None,
            venial::AttributeValue::Group(_, y) => {
                if y.first().map(|v| v.to_string() == to_find).unwrap_or(false) {
                    y.get(2).map(|v| v.clone().into_token_stream())
                } else {
                    None
                }
            }
            venial::AttributeValue::Equals(_, _) => None,
        })
}

fn find_doc_tags(tags: &[venial::Attribute]) -> impl Iterator<Item = String> + '_ {
    tags.iter()
        .filter(|v| {
            let name = v.path.iter().cloned().collect::<TokenStream>().to_string();
            name == "lua_doc" || name == "doc" || name == "tealr_doc"
        })
        .filter_map(|v| match &v.value {
            venial::AttributeValue::Group(_, _) => None,
            venial::AttributeValue::Equals(_, y) => Some(
                y.iter()
                    .flat_map(|v| {
                        let z = v.to_string();
                        z.get(1..(z.len() - 1)).map(|v| v.to_string())
                    })
                    .collect::<String>(),
            ),
            venial::AttributeValue::Empty => None,
        })
}

fn add_commas(mut v: Vec<TokenStream>) -> TokenStream {
    let mut push_into = Vec::new();
    for value in v.drain(0..(v.len() - 1)) {
        push_into.push(value);
        push_into.push(proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone).to_token_stream());
    }
    push_into.push(v.remove(0));
    push_into.into_iter().collect()
}
struct BasicConfig {
    to_location: TokenStream,
    from_location: TokenStream,
    create_table: TokenStream,
    result_location_to: TokenStream,
    result_location_from: TokenStream,
    lua_type: TokenStream,
    lua_value: TokenStream,
    to_lua_name: TokenStream,
    error_message: TokenStream,
    type_name_path: TokenStream,
    type_body_loc: TokenStream,
    type_generator_loc: TokenStream,
    record_generator_loc: TokenStream,
    enum_generator_loc: TokenStream,
    user_data_location: TokenStream,
    teal_data_location: TokenStream,
    has_userdata_fields: bool,
    user_data_fields_location: TokenStream,
    teal_data_fields_location: TokenStream,
    user_data_wrapper_location: TokenStream,
    user_data_methods_location: TokenStream,
    teal_data_methods_location: TokenStream,
    invalid_enum_variant_error: TokenStream,
    typename_macro: TokenStream,
}

fn implement_for_struct(structure: Struct, config: BasicConfig) -> TokenStream {
    let to_loc = config.to_location;
    let from_loc = config.from_location;
    let create_table = config.create_table;
    let result_location_to = config.result_location_to;
    let result_location_from = config.result_location_from;
    let lua_location = config.lua_type;
    let lua_value = config.lua_value;
    let error_message = config.error_message;
    let type_name_path = config.type_name_path;
    let type_body_loc = config.type_body_loc;
    let type_generator_loc = config.type_generator_loc;
    let record_generator_loc = config.record_generator_loc;
    let name = &structure.name;
    let to_lua_name = config.to_lua_name;

    let (to_add, (to_remove, type_body)): (TokenStream, (TokenStream, TokenStream)) =
        match structure.fields {
            venial::Fields::Unit => {
                return venial::Error::new("Unit structs are not supported.").to_compile_error()
            }
            venial::Fields::Tuple(x) => x
                .fields
                .iter()
                .enumerate()
                .map(|(key, x)| {
                    let ty = &x.0.ty;
                    let name = format!("param{key}");
                    let key_as_str = Literal::usize_unsuffixed(key);
                    let (set_value, get_value, type_name) =
                        find_tag_with_value("remote", &x.0.attributes)
                            .map(|v| {
                                (
                                    quote! {<#v as ::std::convert::From<#ty>>::from(self.#key_as_str)},
                                    quote! {get::<#v>(#key)?.into()},
                                    v.to_token_stream(),
                                )
                            })
                            .unwrap_or_else(|| {
                                (quote! {self.#key_as_str}, quote! {get(#key)?}, quote! {#ty})
                            });
                    let docs = find_doc_tags(&x.0.attributes).map(|v| quote! {
                        gen.document(#v);
                    }).collect::<TokenStream>();
                    (
                        quote! {table.set(#key,#set_value)?;},
                        (
                            quote! {#key_as_str: as_table.#get_value,},
                            quote! {
                                #docs
                                gen
                                    .fields
                                    .push(
                                        ::std::convert::From::from((::std::borrow::Cow::Borrowed(#name).into(),
                                        <(#type_name) as #type_name_path>::to_typename()))
                                    );
                            },
                        ),
                    )
                })
                .unzip(),
            venial::Fields::Named(x) => x
                .fields
                .iter()
                .map(|(field, _)| {
                    let name = &field.name;
                    let ty = &field.ty;
                    let (set_value, get_value, type_name) =
                        find_tag_with_value("remote", &field.attributes)
                            .map(|v| {
                                (
                                    quote! {<#v as ::std::convert::From<#ty>>::from(self.#name)},
                                    quote! {get::<#v>(stringify!(#name))?.into()},
                                    v.to_token_stream(),
                                )
                            })
                            .unwrap_or_else(|| {
                                (
                                    quote! {self.#name},
                                    quote! {get(stringify!(#name))?},
                                    quote! {#ty},
                                )
                            });
                    let docs = find_doc_tags(&field.attributes).map(|v| quote! {
                        gen.document(#v);
                    }).collect::<TokenStream>();
                    (
                        quote! { table.set(stringify!(#name),#set_value)?;},
                        (
                            quote! {#name: as_table.#get_value,},
                            quote! {
                                #docs
                                gen
                                    .fields
                                    .push(
                                        ::std::convert::From::from((::std::borrow::Cow::Borrowed(stringify!(#name)).into(),
                                        <(#type_name) as #type_name_path>::to_typename()))
                                    );
                                gen.copy_docs(stringify!(#name).as_bytes());
                            },
                        ),
                    )
                })
                .unzip(),
        };
    let document_type = find_doc_tags(&structure.attributes)
        .map(|v| quote! {gen.document_type(#v);})
        .collect::<TokenStream>();
    quote! {
        impl #to_loc for #name {
            fn #to_lua_name(self, #lua_location) -> #result_location_to {
                let mut table = #create_table()?;
                #to_add
                lua.pack(table)
            }
        }
        impl #from_loc for #name {
            fn from_lua(lua_value:#lua_value, #lua_location) -> #result_location_from {
                let as_table = match lua_value {
                    #lua_value::Table(x) => x,
                    x => Err(#error_message)?
                };
                Ok(Self {
                    #to_remove
                })
            }
        }
        impl #type_body_loc for #name {
            fn get_type_body()-> #type_generator_loc {
                let mut gen = #record_generator_loc::new::<Self>(false);
                #document_type
                #type_body
                <#type_generator_loc as ::std::convert::From<_>>::from(gen)
            }
        }
    }
}

fn implement_for_enum(enumeration: venial::Enum, config: BasicConfig) -> TokenStream {
    if enumeration.is_c_enum() {
        return implement_for_c_enum(enumeration, config);
    }
    let call_fields = find_tag_with_value("extend_fields", &enumeration.attributes)
        .map(|v| quote! {#v(fields)})
        .unwrap_or_else(|| quote! {});
    let call_methods = find_tag_with_value("extend_methods", &enumeration.attributes)
        .map(|v| quote! {#v(methods)});
    let name = enumeration.name;
    let user_data_location = config.user_data_location;
    let user_data_fields_location = config.user_data_fields_location;
    let teal_data_location = config.teal_data_location;
    let user_data_wrapper_location = config.user_data_wrapper_location;
    let teal_data_fields_location = config.teal_data_fields_location;
    let type_body_loc = config.type_body_loc;
    let type_generator_loc = config.type_generator_loc;
    let user_data_methods_location = config.user_data_methods_location;
    let teal_data_methods_location = config.teal_data_methods_location;
    let record_generator_loc = config.record_generator_loc;
    let type_name_macro = config.typename_macro;

    let (add_fields_user_data, add_fields_teal_data, add_fields_type_body) = config
        .has_userdata_fields
        .then(|| {
            (
                quote! {
                    fn add_fields<F: #user_data_fields_location<Self>>(fields: &mut F) {
                        let mut wrapper = #user_data_wrapper_location::from_user_data_fields(fields);
                        <Self as #teal_data_location>::add_fields(&mut wrapper)
                    }
                },
                quote! {
                    fn add_fields<F: #teal_data_fields_location<Self>>(fields: &mut F) {
                        #call_fields
                    }
                },
                quote! {
                    <Self as #teal_data_location>::add_fields(&mut gen);
                },
            )
        })
        .unwrap_or_else(|| (quote! {}, quote! {}, quote! {}));
    let (variant_functions, (creator_functions, is_of_branches)): (Vec<_>, (Vec<_>, TokenStream)) = enumeration
        .variants
        .iter()
        .map(|(variant, _)| {
            let variant_name = variant.name.clone();
            let variant_as_text = variant_name.to_string();
            let is_method_name = format!("Is{}", variant_name);
            let new_variant = format!("New{}", variant_name);
            if variant.is_empty_variant() || matches!(variant.fields, venial::Fields::Unit)
            {
                (vec![quote! {
                    methods.add_method(
                        #is_method_name,
                        |_,this,()| match this {
                            #name::#variant_name => Ok(true),
                            _ => Ok(false)
                        }
                    );
                }], (
                     vec![quote! {
                        methods.add_function(
                            #new_variant,
                            |_,()| Ok(#name::#variant_name)
                        );
                    }],
                     quote! {
                        #name::#variant_name => #variant_as_text,
                    }
                 ))
            } else {
                match &variant.fields {
                    venial::Fields::Unit => unreachable!(),
                    venial::Fields::Tuple(x) => {
                        let field_names = x
                            .fields
                            .iter()
                            .cloned()
                            .enumerate()
                            .map(|(key, (x, _))| {
                                let z = format!("param{key}");
                                (
                                    proc_macro2::Ident::new(
                                        &z,
                                        x.ty.tokens.first().map(|v| v.span())
                                            .unwrap_or_else(Span::call_site)),
                                    x,
                                )
                            })
                            .map(|(v, field)| (quote! {#v}, field))
                            .map(|(v, field)| {
                                let (to_with_from_conversion, to_as_type) = find_tag_with_value("remote", &field.attributes)
                                    .or_else(|| {
                                        let name = field.ty;
                                        Some(quote! {#name})
                                    })
                                    .map(|to|
                                        (quote! {<(#to) as From<_>>::from(<_ as ::std::borrow::ToOwned>::to_owned(#v))}, to)
                                    )
                                    .unwrap();
                                (v, to_with_from_conversion, to_as_type)
                            })
                            .collect::<Vec<_>>();
                        let (combined_with, combined_none): (Vec<TokenStream>, Vec<TokenStream>) =
                            field_names
                                .iter()
                                .map(|(_, to, _)| (quote!(Some(#to)), quote!(None)))
                                .unzip();
                        let to_as_types = add_commas(field_names.iter().map(|(_, _, t)| t.to_owned()).collect::<Vec<_>>());
                        let combined_none = add_commas(combined_none);
                        let combined_with = add_commas(combined_with);
                        let fields = add_commas(field_names.iter().map(|v| v.0.clone()).collect());
                        let is_method_name = format!("Is{variant_name}");
                        let get_method_name = format!("Get{variant_name}");
                        let get_or_nill_method_name = format!("Get{variant_name}OrNil");
                        let new_variant_from = format!("New{variant_name}From");
                        let convert_back = add_commas(field_names.into_iter().map(|(name, _, _)| quote! {<_ as ::std::convert::From<_>>::from(#name)}).collect::<Vec<TokenStream>>());

                        (vec![quote! {
                            methods.add_method(
                                #is_method_name,
                                |_,this,()| match this {
                                    #name::#variant_name(..) => Ok(true),
                                    _ => Ok(false)
                                }
                            );
                            methods.add_method(
                                #get_method_name,
                                |_,this,()| match this {
                                    #name::#variant_name(#fields) => Ok((true,#combined_with)),
                                    _ => Ok((false,#combined_none))
                                }
                            );
                            methods.add_method(
                                #get_or_nill_method_name,
                                |_,this,()| match this {
                                    #name::#variant_name(#fields) => Ok((#combined_with)),
                                    _ => Ok((#combined_none))
                                }
                            );
                        }], (
                             vec![quote! {
                                methods.add_function(
                                    #new_variant_from,
                                    |_, (#fields):(#to_as_types)|
                                        Ok(#name::#variant_name(#convert_back))
                                );
                        }], quote! {
                            #name::#variant_name(..) => #variant_as_text,
                        }))
                    }
                    venial::Fields::Named(_) => todo!(),
                }
            }
        })
        .unzip();
    let variant_functions: TokenStream = variant_functions.into_iter().flatten().collect();
    let creator_functions: TokenStream = creator_functions.into_iter().flatten().collect();
    let document_type = find_doc_tags(&enumeration.attributes)
        .map(|v| quote! {gen.document_type(#v);})
        .collect::<TokenStream>();
    let mut trait_impls = quote! {
        impl #user_data_location for #name {
            #add_fields_user_data
            fn add_methods<M: #user_data_methods_location<Self>>(methods: &mut M) {
                let mut wrapper = #user_data_wrapper_location::from_user_data_methods(methods);
                <Self as #teal_data_location>::add_methods(&mut wrapper)
            }
        }
        impl #teal_data_location for #name {
            #add_fields_teal_data
            fn add_methods<T: #teal_data_methods_location<Self>>(methods: &mut T) {
                #variant_functions
                #call_methods
                methods.add_method("GetTypeName",|_,this,()|{
                    Ok(match this {
                        #is_of_branches
                    }.to_string())
                });
                #creator_functions
            }
        }
        impl #type_body_loc for #name {
            fn get_type_body() -> #type_generator_loc {
                let mut gen = #record_generator_loc::new::<Self>(false);
                gen.is_user_data = true;
                #document_type
                #add_fields_type_body;
                <Self as #teal_data_location>::add_methods(&mut gen);
                <#type_generator_loc as ::std::convert::From<_>>::from(gen)
            }
        }
    };
    let creator_struct_name = find_tag_with_value("creator_name", &enumeration.attributes)
        .map(|v| v.to_token_stream())
        .unwrap_or_else(|| {
            proc_macro2::Ident::new(&format!("{name}Creator"), name.span()).into_token_stream()
        });
    let visibility = enumeration
        .vis_marker
        .map(|v| v.to_token_stream())
        .unwrap_or_else(|| quote! {});
    let attributes = enumeration
        .attributes
        .iter()
        .filter(|x| {
            x.path
                .iter()
                .map(|x| x.to_string())
                .any(|x| x.contains("tealr"))
        })
        .map(|v| v.to_token_stream())
        .collect::<TokenStream>();
    let creator_struct_stream = quote! {
        #[derive(#type_name_macro)]
        #attributes
        ///Automatically generated for exporting to lua
        #visibility struct #creator_struct_name {}
    };
    let parsed = parse_item(creator_struct_stream.clone()).unwrap();
    let with_userdata = crate::user_data::impl_mlua_user_data_derive(&parsed);
    //let with_type_name = crate::user_data::impl_type_representation_derive(&parsed);
    let with_clone = quote! {
        impl ::std::clone::Clone for #creator_struct_name {
            fn clone(&self) -> Self {
                Self {}
            }
        }
    };
    let with_teal_data = quote! {
        impl #teal_data_location for #creator_struct_name {
            fn add_methods<T: #teal_data_methods_location<Self>>(methods: &mut T) {
                #creator_functions
            }
        }
    };
    let with_new_method = quote! {
        impl #creator_struct_name {
            ///creates a new instance of this object
            pub fn new() -> Self {
                Self{}
            }
        }
        impl ::std::default::Default for #creator_struct_name {
            fn default() -> Self {
                Self::new()
            }
        }
    };

    trait_impls.extend(Some(creator_struct_stream));
    trait_impls.extend(Some(with_userdata));
    trait_impls.extend(Some(with_clone));
    trait_impls.extend(Some(with_teal_data));
    trait_impls.extend(Some(with_new_method));
    let from_loc = config.from_location;
    let lua_value = config.lua_value;
    let lua_location = config.lua_type;
    let error_message = config.error_message;
    let result_location_from = config.result_location_from;
    let with_from_lua = quote! {
        impl #from_loc for #name {
            fn from_lua(lua_value:#lua_value, #lua_location) -> #result_location_from {
                match lua_value.as_userdata() {
                    Some(x) => x.take(),
                    None => {
                        let x = lua_value;
                        Err(#error_message)
                    }
                }
            }
        }
    };
    trait_impls.extend(with_from_lua);
    trait_impls
}

fn implement_for_c_enum(enumeration: venial::Enum, config: BasicConfig) -> TokenStream {
    let name = enumeration.name;
    let type_body_loc = config.type_body_loc;
    let type_generator_loc = config.type_generator_loc;
    let to_loc = config.to_location;
    let from_loc = config.from_location;
    let result_location_to = config.result_location_to;
    let result_location_from = config.result_location_from;
    let lua_location = config.lua_type;
    let lua_value = config.lua_value;
    let enum_generator_loc = config.enum_generator_loc;
    let invalid_enum_variant_error = config.invalid_enum_variant_error;
    let to_lua_name = config.to_lua_name;
    let document_type = find_doc_tags(&enumeration.attributes)
        .map(|v| quote! {gen.document_type(#v);})
        .collect::<TokenStream>();

    let (to_branches, (from_branches, variants)): (TokenStream, (TokenStream, TokenStream)) =
        enumeration
            .variants
            .iter()
            .map(|(v, _)| {
                let variant_name = &v.name;
                (
                    quote! {#name::#variant_name => stringify!(#variant_name),},
                    (
                        quote! {stringify!(#variant_name) => #name::#variant_name,},
                        quote! {
                            gen
                                .variants
                                .push(
                                    ::std::borrow::Cow::Borrowed(stringify!(#variant_name)).into(),
                                );
                        },
                    ),
                )
            })
            .unzip();

    quote! {
        impl #to_loc for #name {
            fn #to_lua_name(self, #lua_location) -> #result_location_to {
                let res = match self {
                    #to_branches
                };
                lua.pack(res.to_string())
            }
        }
        impl #from_loc for #name {
            fn from_lua(lua_value:#lua_value, #lua_location) -> #result_location_from {
                let x = <std::string::String as #from_loc>::from_lua(lua_value,lua)?;
                Ok(match x.as_str() {
                    #from_branches
                    x => return Err(#invalid_enum_variant_error)
                })
            }
        }
        impl #type_body_loc for #name {
            fn get_type_body()-> #type_generator_loc {
                let mut gen = #enum_generator_loc::new::<Self>();
                #document_type;
                #variants;
                <#type_generator_loc as ::std::convert::From<_>>::from(gen)
            }
        }
    }
}

pub(crate) fn mlua_from_to_lua(input: TokenStream) -> TokenStream {
    let parsed = parse_item(input).unwrap();
    let tealr_name = get_tealr_name(parsed.attributes());
    let config = BasicConfig {
        to_location: quote! {#tealr_name::mlu::mlua::IntoLua},
        to_lua_name: quote!(into_lua),
        from_location: quote! {#tealr_name::mlu::mlua::FromLua},
        create_table: quote! {lua.create_table},
        result_location_to: quote! {#tealr_name::mlu::mlua::Result<#tealr_name::mlu::mlua::Value>},
        result_location_from: quote! {#tealr_name::mlu::mlua::Result<Self>},
        lua_type: quote! {lua: &#tealr_name::mlu::mlua::Lua},
        lua_value: quote! {#tealr_name::mlu::mlua::Value},
        error_message: quote! {
            #tealr_name::mlu::mlua::Error::FromLuaConversionError{
                from: x.type_name(),
                to: "unknown".to_string(),
                message:None
            }
        },
        type_name_path: quote! {#tealr_name::ToTypename},
        type_body_loc: quote! {#tealr_name::TypeBody},
        type_generator_loc: quote! {#tealr_name::TypeGenerator},
        record_generator_loc: quote! {#tealr_name::RecordGenerator},
        enum_generator_loc: quote! {#tealr_name::EnumGenerator},
        user_data_location: quote! {#tealr_name::mlu::mlua::UserData},
        teal_data_location: quote! {#tealr_name::mlu::TealData},
        has_userdata_fields: true,
        user_data_fields_location: quote! {#tealr_name::mlu::mlua::UserDataFields},
        teal_data_fields_location: quote! {#tealr_name::mlu::TealDataFields},
        user_data_wrapper_location: quote! {#tealr_name::mlu::UserDataWrapper},
        user_data_methods_location: quote! {#tealr_name::mlu::mlua::UserDataMethods},
        teal_data_methods_location: quote! {#tealr_name::mlu::TealDataMethods},
        invalid_enum_variant_error: quote! {#tealr_name::mlu::mlua::Error::FromLuaConversionError {
            from: "String",
            to:"unknown".to_string(),
            message:None
        } },
        typename_macro: quote! {#tealr_name::ToTypename},
    };

    match parsed {
        venial::Item::Struct(x) => implement_for_struct(x, config),
        venial::Item::Enum(x) => implement_for_enum(x, config),
        _ => venial::Error::new("Only structs and enums are supported").to_compile_error(),
    }
}
