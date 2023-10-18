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
                    .last()
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
