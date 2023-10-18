use proc_macro::TokenStream;
use proc_macro2::{Ident, Punct, Span};
use quote::quote;
use syn::{
    parse,
    punctuated::Punctuated,
    token::{Gt, Lt},
    Data, DeriveInput, Fields, FieldsNamed, FieldsUnnamed, GenericParam, TypeParam, Variant,
    WhereClause,
};

#[proc_macro_derive(Parse, attributes(hi))]
pub fn derive_parse(input: TokenStream) -> TokenStream {
    // Get base information of struct
    let ast: DeriveInput = parse(input).unwrap();
    let name = ast.ident;

    // Get the generics we need to pass to the item
    let mut generics = ast.generics.clone();
    let binding = generics.clone();
    let (_, type_generics, _) = binding.split_for_impl();
    generics.lt_token.get_or_insert(Lt::default());
    generics.gt_token.get_or_insert(Gt::default());

    // Add a generic for the input type
    let input_type = Ident::new("__InputType", Span::call_site());
    generics.params.push(GenericParam::Type(TypeParam {
        attrs: vec![],
        ident: input_type.clone(),
        colon_token: None,
        bounds: Default::default(),
        eq_token: None,
        default: None,
    }));

    // Add `where` predicates to make sure each type can be parsed from `__InputType`
    let where_clause = generics.make_where_clause();
    let types_that_must_implement_parse: Vec<_> = match &ast.data {
        Data::Struct(data) => match &data.fields {
            Fields::Unit => vec![],
            Fields::Unnamed(fields) => fields.unnamed.iter().map(|x| x.ty.clone()).collect(),
            Fields::Named(fields) => fields.named.iter().map(|x| x.ty.clone()).collect(),
        },
        Data::Enum(data) => data
            .variants
            .iter()
            .flat_map(|variant| match &variant.fields {
                Fields::Unit => vec![],
                Fields::Unnamed(fields) => fields.unnamed.iter().map(|x| x.ty.clone()).collect(),
                Fields::Named(fields) => fields.named.iter().map(|x| x.ty.clone()).collect(),
            })
            .collect(),
        Data::Union(data) => data.fields.named.iter().map(|x| x.ty.clone()).collect(),
    };
    for ty in types_that_must_implement_parse {
        where_clause
            .predicates
            .push(parse(quote! { #ty: Parse<#input_type> }.into()).unwrap());
    }

    // Separate the `impl` generics and `where` clause
    let (impl_generics, _, where_clause) = generics.split_for_impl();

    match ast.data {
        Data::Struct(data) => match data.fields {
            Fields::Unit => {
                let gen = quote! {
                    impl #impl_generics Parse<#input_type> for #name #type_generics #where_clause {
                        type Error = ::std::convert::Infallible;

                        fn parse(input: #input_type) -> Result<(#input_type, Self), Self::Error> {
                            Ok((input, Self))
                        }
                    }
                };

                gen.into()
            }

            Fields::Unnamed(fields) => {
                let Some(first_ty) = fields.unnamed.first().map(|field| &field.ty) else {
                    let gen = quote! {
                        impl #impl_generics Parse<#input_type> for #name #type_generics #where_clause {
                            type Error = ::std::convert::Infallible;

                            fn parse(input: #input_type) -> Result<(#input_type, Self), Self::Error> {
                                Ok((input, Self()))
                            }
                        }
                    };

                    return gen.into();
                };

                let bindings = fields.unnamed.iter().enumerate().map(|(index, _)| {
                    let mut ident = "field".to_string();
                    ident += &index.to_string();
                    let ident = Ident::new(&ident, Span::call_site());
                    ident
                });

                let fields = bindings.clone();

                let gen = quote! {
                    impl #impl_generics Parse<#input_type> for #name #type_generics #where_clause {
                        type Error = <#first_ty as Parse<#input_type>>::Error;

                        fn parse(input: #input_type) -> Result<(#input_type, Self), Self::Error> {
                            #(let (input, #bindings) = Parse::parse(input)?;)*
                            Ok((input, Self(#(#fields,)*)))
                        }
                    }
                };

                return gen.into();
            }

            Fields::Named(fields) => {
                let Some(first_ty) = fields.named.first().map(|field| &field.ty) else {
                    let gen = quote! {
                        impl #impl_generics Parse<#input_type> for #name #type_generics #where_clause {
                            type Error = ::std::convert::Infallible;

                            fn parse(input: #input_type) -> Result<(#input_type, Self), Self::Error> {
                                Ok((input, Self()))
                            }
                        }
                    };

                    return gen.into();
                };

                let bindings = fields.named.iter().enumerate().map(|(index, _)| {
                    let mut ident = "field".to_string();
                    ident += &index.to_string();
                    let ident = Ident::new(&ident, Span::call_site());
                    ident
                });

                let fields = fields.named.iter().enumerate().map(|(index, field)| {
                    let mut ident = "field".to_string();
                    ident += &index.to_string();
                    let value_ident = Ident::new(&ident, Span::call_site());
                    let key_ident = &field.ident;
                    quote! { #key_ident: #value_ident }
                });

                let gen = quote! {
                    impl #impl_generics Parse<#input_type> for #name #type_generics #where_clause {
                        type Error = <#first_ty as Parse<#input_type>>::Error;

                        fn parse(input: #input_type) -> Result<(#input_type, Self), Self::Error> {
                            #(let (input, #bindings) = Parse::parse(input)?;)*
                            Ok((input, Self { #(#fields,)* }))
                        }
                    }
                };

                return gen.into();
            }
        },

        Data::Enum(data) => {
            let is_infallible = data.variants.iter().any(|variant| match &variant.fields {
                Fields::Unit => true,
                Fields::Unnamed(fields) => fields.unnamed.is_empty(),
                Fields::Named(fields) => fields.named.is_empty(),
            });

            // If this is `None`, it means the parser is infallible.
            let error_type = if is_infallible {
                None
            } else {
                let last_variant = data
                    .variants
                    .first()
                    .expect("this trait cannot be derived on an empty enum");

                match &last_variant.fields {
                    Fields::Unit => None,
                    Fields::Unnamed(FieldsUnnamed {
                        unnamed: fields, ..
                    })
                    | Fields::Named(FieldsNamed { named: fields, .. }) => {
                        fields.first().map(|field| field.ty.clone())
                    }
                }
            };

            let last_index = data.variants.len() - 1;

            let do_we_need_to_clone_input_type =
                data.variants
                    .iter()
                    .enumerate()
                    .any(|(index, variant)| match &variant.fields {
                        Fields::Unit => false,
                        _ => index != last_index,
                    });

            let attempts = data.variants.iter().enumerate().map(|(index, variant)| {
                let is_last = index == last_index;

                let name = &variant.ident;

                match &variant.fields {
                    Fields::Unit => quote! { return Ok((input, Self::#name)); },

                    Fields::Unnamed(fields) => {
                        let bindings = fields.unnamed.iter().enumerate().map(|(index, _)| {
                            let mut ident = "field".to_string();
                            ident += &index.to_string();
                            let ident = Ident::new(&ident, Span::call_site());
                            ident
                        });

                        let fields = bindings.clone();

                        if is_last {
                            let gen = quote! {
                                {
                                    #(let (input, #bindings) = Parse::parse(input)?;)*
                                    Ok((input, Self::#name(#(#fields,)*)))
                                }
                            };

                            return gen.into();
                        }

                        let gen = quote! {
                            'attempt: {
                                let input = input.clone();
                                #(let Ok((input, #bindings)) = Parse::parse(input) else {
                                    break 'attempt;
                                };)*
                                return Ok((input, Self::#name(#(#fields,)*)));
                            }
                        };

                        return gen.into();
                    }

                    Fields::Named(fields) => {
                        let bindings = fields.named.iter().enumerate().map(|(index, _)| {
                            let mut ident = "field".to_string();
                            ident += &index.to_string();
                            let ident = Ident::new(&ident, Span::call_site());
                            ident
                        });

                        let fields = fields.named.iter().enumerate().map(|(index, field)| {
                            let mut ident = "field".to_string();
                            ident += &index.to_string();
                            let value_ident = Ident::new(&ident, Span::call_site());
                            let key_ident = &field.ident;
                            quote! { #key_ident: #value_ident }
                        });

                        if is_last {
                            let gen = quote! {
                                {
                                    #(let (input, #bindings) = Parse::parse(input)?;)*
                                    Ok((input, Self::#name { #(#fields,)* }))
                                }
                            };

                            return gen.into();
                        }

                        let gen = quote! {
                            'attempt: {
                                let input = input.clone();
                                #(let Ok((input, #bindings)) = Parse::parse(input) else {
                                    break 'attempt;
                                };)*
                                return Ok((input, Self::#name { #(#fields,)* }));
                            }
                        };

                        return gen.into();
                    }
                }
            });

            let error_type = match error_type {
                None => quote! { ::std::convert::Infallible },
                Some(error_type) => quote! { <#error_type as Parse<#input_type>>::Error },
            };

            let where_clause = if do_we_need_to_clone_input_type {
                let mut clause = where_clause
                    .map(|x| x.clone())
                    .unwrap_or_else(|| WhereClause {
                        where_token: Default::default(),
                        predicates: Punctuated::new(),
                    });

                clause
                    .predicates
                    .push(parse(quote! { #input_type: ::core::clone::Clone }.into()).unwrap());

                Some(clause)
            } else {
                where_clause.map(|x| x.clone())
            };

            let gen = quote! {
                impl #impl_generics Parse<#input_type> for #name #type_generics #where_clause {
                    type Error = #error_type;

                    fn parse(input: #input_type) -> Result<(#input_type, Self), Self::Error> {
                        #(#attempts)*
                    }
                }
            };

            gen.into()
        }

        Data::Union(data) => unimplemented!(),
    }
}

// #[proc_macro_derive(Parse)]
// pub fn parse_derive(input: TokenStream) -> TokenStream {
//     let ast: DeriveInput = syn::parse(input).unwrap();
//     let name = ast.ident;
//     let (impl_generics, type_generics, where_clause) = ast.generics.split_for_impl();

//     match ast.data {
//         Data::Struct(data) => match data.fields {
//             Fields::Unit => {
//                 let gen = quote! {
//                     impl #impl_generics Parse for #name #type_generics #where_clause {
//                         type Error = ::std::convert::Infallible;

//                         fn parse(chars: ::std::str::Chars) -> Result<(::std::str::Chars<'_>, Self), Self::Error> {
//                             Ok((chars, #name))
//                         }
//                     }
//                 };

//                 gen.into()
//             }

//             Fields::Unnamed(fields) => {
//                 let Some(syn::Field { ty: first_ty, .. }) = fields.unnamed.first() else {
//                     return quote! {
//                         impl #impl_generics Parse for #name #type_generics #where_clause {
//                             type Error = ::std::convert::Infallible;

//                             fn parse(chars: ::std::str::Chars) -> Result<(::std::str::Chars<'_>, Self), Self::Error> {
//                                 Ok((chars, #name ()))
//                             }
//                         }
//                     }.into();
//                 };

//                 let parse_statements = fields.unnamed.iter().enumerate().map(|(index, _)| {
//                     let mut ident = "field".to_string();
//                     ident += &index.to_string();
//                     let ident = Ident::new(&ident, Span::call_site());

//                     quote! { let (chars, #ident) = Parse::parse(chars)?; }
//                 });

//                 let names = fields.unnamed.iter().enumerate().map(|(index, _)| {
//                     let mut ident = "field".to_string();
//                     ident += &index.to_string();
//                     Ident::new(&ident, Span::call_site())
//                 });

//                 let gen = quote! {
//                     impl #impl_generics Parse for #name #type_generics #where_clause {
//                         type Error = <#first_ty as Parse>::Error;

//                         fn parse(chars: ::std::str::Chars) -> Result<(::std::str::Chars<'_>, Self), Self::Error> {
//                             #(#parse_statements)*
//                             Ok((chars, #name ( #(#names),* )))
//                         }
//                     }
//                 };

//                 gen.into()
//             }

//             Fields::Named(fields) => {
//                 let Some(syn::Field { ty: first_ty, .. }) = fields.named.first() else {
//                     return quote! {
//                         impl #impl_generics Parse for #name #type_generics #where_clause {
//                             type Error = ::std::convert::Infallible;

//                             fn parse(chars: ::std::str::Chars) -> Result<(::std::str::Chars<'_>, Self), Self::Error> {
//                                 Ok((chars, #name {}))
//                             }
//                         }
//                     }.into();
//                 };

//                 let parse_statements = fields.named.iter().enumerate().map(|(index, _)| {
//                     let mut ident = "field".to_string();
//                     ident += &index.to_string();
//                     let ident = Ident::new(&ident, Span::call_site());

//                     quote! { let (chars, #ident) = Parse::parse(chars)?; }
//                 });

//                 let names = fields.named.iter().enumerate().map(|(index, field)| {
//                     let mut ident = "field".to_string();
//                     ident += &index.to_string();

//                     let value_ident = Ident::new(&ident, Span::call_site());
//                     let key_ident = field
//                         .ident
//                         .clone()
//                         .expect("named structs should have named keys");

//                     quote! { #key_ident: #value_ident }
//                 });

//                 let gen = quote! {
//                     impl #impl_generics Parse for #name #type_generics #where_clause {
//                         type Error = <#first_ty as Parse>::Error;

//                         fn parse(chars: ::std::str::Chars) -> Result<(::std::str::Chars<'_>, Self), Self::Error> {
//                             #(#parse_statements)*
//                             Ok((chars, #name { #(#names),* }))
//                         }
//                     }
//                 };

//                 gen.into()
//             }
//         },

//         Data::Enum(data) => {
//             let variants = data.variants.iter().map(|variant| match &variant.fields {
//                 Fields::Unit => {
//                     let variant_name = &variant.ident;

//                     quote! {
//                         return Ok((chars, #name :: #variant_name));
//                     }
//                 }

//                 Fields::Unnamed(fields) => {
//                     let variant_name = &variant.ident;

//                     if fields.unnamed.len() == 0 {
//                         return quote! {
//                             return Ok((chars, #name :: #variant_name ()))
//                         };
//                     }

//                     let lifetime = Lifetime::new("'attempt", Span::call_site());

//                     let parse_statements = fields.unnamed.iter().enumerate().map(|(index, _)| {
//                         let mut ident = "field".to_string();
//                         ident += &index.to_string();
//                         let ident = Ident::new(&ident, Span::call_site());

//                         quote! {
//                             let (chars, #ident) = match Parse::parse(chars) {
//                                 Ok((chars, #ident)) => (chars, #ident),
//                                 _ => break #lifetime,
//                             };
//                         }
//                     });

//                     let names = fields.unnamed.iter().enumerate().map(|(index, _)| {
//                         let mut ident = "field".to_string();
//                         ident += &index.to_string();
//                         Ident::new(&ident, Span::call_site())
//                     });

//                     quote! {
//                         'attempt: {
//                             let chars = chars.clone();
//                             #(#parse_statements)*
//                             return Ok((chars, #name :: #variant_name ( #(#names),* )));
//                         }
//                     }
//                 }

//                 Fields::Named(fields) => {
//                     let variant_name = &variant.ident;

//                     if fields.named.len() == 0 {
//                         return quote! {
//                             return Ok((chars, #name :: #variant_name {}))
//                         };
//                     }

//                     let lifetime = Lifetime::new("'attempt", Span::call_site());

//                     let parse_statements = fields.named.iter().enumerate().map(|(index, _)| {
//                         let mut ident = "field".to_string();
//                         ident += &index.to_string();
//                         let ident = Ident::new(&ident, Span::call_site());

//                         quote! {
//                             let (chars, #ident) = match Parse::parse(chars) {
//                                 Ok((chars, #ident)) => (chars, #ident),
//                                 _ => break #lifetime,
//                             };
//                         }
//                     });

//                     let names = fields.named.iter().enumerate().map(|(index, field)| {
//                         let mut ident = "field".to_string();
//                         ident += &index.to_string();

//                         let value_ident = Ident::new(&ident, Span::call_site());

//                         let key_ident = field
//                             .ident
//                             .as_ref()
//                             .expect("struct variants must have named fields");

//                         quote! { #key_ident: #value_ident }
//                     });

//                     quote! {
//                         'attempt: {
//                             let chars = chars.clone();
//                             #(#parse_statements)*
//                             return Ok((chars, #name :: #variant_name { #(#names),* }));
//                         }
//                     }
//                 }
//             });

//             let gen = if data.variants.iter().any(|x| match &x.fields {
//                 Fields::Unit => true,
//                 Fields::Unnamed(fields) => fields.unnamed.len() == 0,
//                 Fields::Named(fields) => fields.named.len() == 0,
//             }) {
//                 quote! {
//                     impl #impl_generics Parse for #name #type_generics #where_clause {
//                         type Error = ::std::convert::Infallible;

//                         fn parse(chars: ::std::str::Chars) -> Result<(::std::str::Chars<'_>, Self), Self::Error> {
//                             #(#variants)*
//                         }
//                     }
//                 }
//             } else {
//                 quote! {
//                     impl #impl_generics Parse for #name #type_generics #where_clause {
//                         type Error = &'static str;

//                         fn parse(chars: ::std::str::Chars) -> Result<(::std::str::Chars<'_>, Self), Self::Error> {
//                             #(#variants)*
//                             Err("didn't match any alternative")
//                         }
//                     }
//                 }
//             };

//             gen.into()
//         }

//         Data::Union(data) => {
//             let keys = data.fields.named.iter().map(|x| &x.ident);

//             let gen = quote! {
//                 impl #impl_generics Parse for #name #type_generics #where_clause {
//                     type Error = &'static str;

//                     fn parse(chars: ::std::str::Chars) -> Result<(::std::str::Chars<'_>, Self), Self::Error> {
//                         #(
//                             if let Ok((chars, value)) = Parse::parse(chars.clone()) {
//                                 return Ok((chars, #name { #keys: value }));
//                             };
//                         )*

//                         Err("didn't match any alternative")
//                     }
//                 }
//             };

//             gen.into()
//         }
//     }
// }
