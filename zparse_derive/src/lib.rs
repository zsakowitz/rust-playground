use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    parse,
    punctuated::Punctuated,
    token::{Gt, Lt},
    Data, DeriveInput, Fields, GenericParam, TypeParam, WhereClause,
};

#[proc_macro_derive(TryParse)]
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
    let input_type = Ident::new("Input", Span::call_site());
    generics.params.push(GenericParam::Type(TypeParam {
        attrs: vec![],
        ident: input_type.clone(),
        colon_token: None,
        bounds: Default::default(),
        eq_token: None,
        default: None,
    }));

    // Add `where` predicates to make sure each type can be parsed from `Input`
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
            .push(parse(quote! { #ty: TryParse<#input_type> }.into()).unwrap());
    }

    // Separate the `impl` generics and `where` clause
    let (impl_generics, _, where_clause) = generics.split_for_impl();

    match ast.data {
        Data::Struct(data) => match data.fields {
            Fields::Unit => {
                let gen = quote! {
                    impl #impl_generics TryParse<#input_type> for #name #type_generics #where_clause {
                        fn try_parse(input: #input_type) -> Option<(#input_type, Self)> {
                            Some((input, Self))
                        }
                    }
                };

                gen.into()
            }

            Fields::Unnamed(fields) => {
                if fields.unnamed.is_empty() {
                    let gen = quote! {
                        impl #impl_generics TryParse<#input_type> for #name #type_generics #where_clause {
                            fn try_parse(input: #input_type) -> Option<(#input_type, Self)> {
                                Some((input, Self()))
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
                    impl #impl_generics TryParse<#input_type> for #name #type_generics #where_clause {
                        fn try_parse(input: #input_type) -> Option<(#input_type, Self)> {
                            #(let (input, #bindings) = TryParse::try_parse(input)?;)*
                            Some((input, Self(#(#fields,)*)))
                        }
                    }
                };

                return gen.into();
            }

            Fields::Named(fields) => {
                if fields.named.is_empty() {
                    let gen = quote! {
                        impl #impl_generics TryParse<#input_type> for #name #type_generics #where_clause {
                            fn try_parse(input: #input_type) -> Option<(#input_type, Self)> {
                                Some((input, Self {}))
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
                    impl #impl_generics TryParse<#input_type> for #name #type_generics #where_clause {
                        fn try_parse(input: #input_type) -> Option<(#input_type, Self)> {
                            #(let (input, #bindings) = TryParse::try_parse(input)?;)*
                            Some((input, Self { #(#fields,)* }))
                        }
                    }
                };

                return gen.into();
            }
        },

        Data::Enum(data) => {
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
                    Fields::Unit => quote! { return Some((input, Self::#name)); },

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
                                    #(let (input, #bindings) = TryParse::try_parse(input)?;)*
                                    Some((input, Self::#name(#(#fields,)*)))
                                }
                            };

                            return gen.into();
                        }

                        let gen = quote! {
                            'attempt: {
                                let input = input.clone();
                                #(let Some((input, #bindings)) = TryParse::try_parse(input) else {
                                    break 'attempt;
                                };)*
                                return Some((input, Self::#name(#(#fields,)*)));
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
                                    #(let (input, #bindings) = TryParse::try_parse(input)?;)*
                                    Some((input, Self::#name { #(#fields,)* }))
                                }
                            };

                            return gen.into();
                        }

                        let gen = quote! {
                            'attempt: {
                                let input = input.clone();
                                #(let Some((input, #bindings)) = TryParse::try_parse(input) else {
                                    break 'attempt;
                                };)*
                                return Some((input, Self::#name { #(#fields,)* }));
                            }
                        };

                        return gen.into();
                    }
                }
            });

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
                impl #impl_generics TryParse<#input_type> for #name #type_generics #where_clause {
                    fn try_parse(input: #input_type) -> Option<(#input_type, Self)> {
                        #(#attempts)*
                    }
                }
            };

            gen.into()
        }

        Data::Union(data) => unimplemented!(),
    }
}
