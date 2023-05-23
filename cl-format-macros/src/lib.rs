#![doc = r#"The macros here should auto generate several traits and the major TildeAble trait.

For example:

```rust
#[derive(Debug, PartialEq, TildeAble)]
pub enum TildeKind {
    /// ~C ~:C
    #[implTo(char)]
    Char,

    /// ~$ ~5$ ~f
    #[implTo(float)]
    Float(Option<String>),

    /// ~d ~:d ~:@d
    Digit(Option<String>),

    /// ~a
    #[implTo(float, char, String)]
    Va,

    /// loop
    Loop(Vec<Tilde>),

    /// text inside the tilde
    Text(String),

    /// vec
    VecTilde(Vec<Tilde>),
}
```

Will generate:

```rust
/// all default method is return none.
trait TildeAble {
    fn len(&self) -> usize;
    fn into_tildekind_char(&self) -> Option<&dyn TildeKindChar>{None}
    fn into_tildekind_va(&self) -> Option<&dyn TildeKindVa>{None}
    // and all other fields...
}

impl TildeAble for char {
    fn into_tildekind_char(&self) -> Option<&dyn TildeKindChar> {
        Some(self)
    }

    fn into_tildekind_va(&self) -> Option<&dyn TildeKindVa> {
        Some(self)
    }
}

impl TildeAble for float {
    fn into_tildekind_va(&self) -> Option<&dyn TildeKindVa> {
        Some(self)
    }
}

impl TildeAble for String {
    fn into_tildekind_va(&self) -> Option<&dyn TildeKindVa> {
        Some(self)
    }
}

trait TildeKindChar {
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, Box<dyn std::error::Error>> {
        Err("un-implenmented yet".into())
    }
}

trait TildeKindVa {
    fn format(&self, tkind: &TildeKind) -> Result<Option<String>, Box<dyn std::error::Error>> {
        Err("un-implenmented yet".into())
    }
}

```
"#]

use std::{collections::HashMap, error::Error};

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::{
    buffer::{Cursor, TokenBuffer},
    parse::{self, Parser},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    Attribute, Data, DataEnum, DeriveInput, Expr, Token, Variant,
};

#[proc_macro_derive(TildeAble, attributes(implTo))]
pub fn derive_tilde_able(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    //let mut all_methods_headers = vec![];
    let mut return_types_traits = vec![];
    let mut all_default_methods = vec![];
    let mut types_impl_methods = HashMap::new();

    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let all_vars = variants.iter().map(|var| parse_variant_attrs(var));

            all_vars.for_each(|(field, tys)| {
                let fname = Ident::new(
                    &(String::from("into_tildekind_") + &field.to_lowercase()),
                    Span::call_site(),
                );

                let return_type =
                    Ident::new(&(String::from("TildeKind") + &field), Span::call_site());

                // add default methods to TildeAble
                all_default_methods
                    .push(quote! {
						fn #fname(&self) -> Option<&dyn #return_type> {
							None
						}});

                // impl for types
                tys.for_each(|ty| {
                    let en = types_impl_methods.entry(ty).or_insert(vec![]);
                    en.push(quote! {fn #fname(&self) -> Option<&dyn #return_type> {
						Some(self)
					}})
                });

                //				
                return_types_traits.push(quote! {
                    pub trait #return_type: Debug {
                        fn format(&self, tkind: &TildeKind) -> Result<Option<String>, TildeError> {
                            Err(TildeError::new(ErrorKind::EmptyImplenmentError, "haven't implenmented yet").into(),)
                        }
                }})
            });
        }
        _ => panic!("only support the enum"),
    };

    let mut result = vec![];

    // trait TildeAble defination
    let tilde_able_trait = quote! {
        pub trait TildeAble:Debug {
            fn len(&self) -> usize;
            #(#all_default_methods)*
        }
    };

    let mut auto_impl_for_types = types_impl_methods
        .iter()
        .map(|(ty, methods)| {
            quote! {
                impl TildeAble for #ty {
                    fn len(&self) -> usize {
                        1
                    }
                    #(#methods)*
                }
            }
        })
        .collect();

    // merge together
    result.push(tilde_able_trait);
    result.append(&mut auto_impl_for_types);
    result.append(&mut return_types_traits);

    proc_macro2::TokenStream::from_iter(result.into_iter()).into()
}

/// new macro for optimizing
/// Give different methods to trait rahter than all same format
#[proc_macro_derive(TildeAble2, attributes(implTo))]
pub fn derive_tilde_able_2(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    //let mut all_methods_headers = vec![];
    let mut return_types_traits = vec![];
    //let mut all_default_methods = vec![];
    //let mut types_impl_methods = HashMap::new();

    match input.data {
        Data::Enum(DataEnum { ref variants, .. }) => {
            let all_vars = variants.iter().map(|var| parse_variant_attrs(var));

            all_vars.for_each(|(field, tys)| {
                // let fname = Ident::new(
                //     &(String::from("into_tildekind_") + &field.to_lowercase()),
                //     Span::call_site(),
                // );

                let return_type =
                    Ident::new(&(String::from("TildeKind") + &field), Span::call_site());

                // add default methods to TildeAble
                // all_default_methods
                //     .push(quote! {
				// 		fn #fname(&self) -> Option<&dyn #return_type> {
				// 			None
				// 		}});

                // impl for types
                // tys.for_each(|ty| {
                //     let en = types_impl_methods.entry(ty).or_insert(vec![]);
                //     en.push(quote! {fn #fname(&self) -> Option<&dyn #return_type> {
				// 		Some(self)
				// 	}})
                // });

                //
				let method_name = Ident::new(&(String::from("format_to_") + &field.to_lowercase()), Span::call_site());
                return_types_traits.push(quote! {
                    pub trait #return_type: Debug { //:= TODO: change this name
                        fn #method_name(&self, tkind: &TildeKind) -> Result<Option<String>, TildeError> { //:= TODO: also change this name
                            Err(TildeError::new(ErrorKind::EmptyImplenmentError, "haven't implenmented yet").into(),)
                        }
                }})
            });
        }
        _ => panic!("only support the enum"),
    };

    let mut result = vec![];

    // trait TildeAble defination
    // let tilde_able_trait = quote! {
    //     pub trait TildeAble:Debug {
    //         fn len(&self) -> usize;
    //         #(#all_default_methods)*
    //     }
    // };

    // let mut auto_impl_for_types = types_impl_methods
    //     .iter()
    //     .map(|(ty, methods)| {
    //         quote! {
    //             impl TildeAble for #ty {
    //                 fn len(&self) -> usize {
    //                     1
    //                 }
    //                 #(#methods)*
    //             }
    //         }
    //     })
    //     .collect();

    // merge together
    //result.push(tilde_able_trait);
    //result.append(&mut auto_impl_for_types);
    result.append(&mut return_types_traits);

    proc_macro2::TokenStream::from_iter(result.into_iter()).into()
}

/// return the field Ident and all types implTo. Empty if there is no implTo types
fn parse_variant_attrs(variant: &Variant) -> (String, impl Iterator<Item = Ident> + '_) {
    let all_impl_to_type = variant
        .attrs
        .iter()
        .filter(|attr| attr.path().get_ident().map(|d| d.to_string()) == Some("implTo".to_string()))
        .map(|attr| get_types_impl_to(attr).unwrap())
        .flatten();

    let field = variant.ident.to_string();

    (field.clone(), all_impl_to_type)
}

/// parse the `implTo` attribute
fn get_types_impl_to(attribute: &Attribute) -> Result<impl Iterator<Item = Ident>, Box<dyn Error>> {
    let mut result = vec![];
    attribute.parse_nested_meta(|meta| {
        result.push(
            meta.path
                .get_ident()
                .ok_or(syn::Error::new(meta.path.span(), "get_ident issue"))?
                .clone(),
        );
        Ok(())
    });

    Ok(result.into_iter())
}

///////////////////////
///////////////////////
///////////////////////

// abandon, use the one in cl-format
#[proc_macro]
pub fn cl_format(tokens: TokenStream) -> TokenStream {
    let items = Punctuated::<Expr, Token![,]>::parse_terminated
        .parse(tokens)
        .unwrap();

    //dbg!(&items);

    let mut items = items.pairs();

    let cs = match items.next() {
        Some(cs) => match cs.value() {
            Expr::Lit(l) => match &l.lit {
                syn::Lit::Str(s) => {
                    //dbg!(s.value());
                    let ss = s.value();
                    quote! {let cs = control_str::ControlStr::from(#ss).unwrap();}
                }
                _ => panic!("the first arg have to be &str"),
            },
            Expr::Path(syn::ExprPath { attrs, qself, path }) => {
                let pp = path
                    .get_ident()
                    .unwrap_or_else(|| panic!("path get ident failed"));
                quote! {let cs = control_str::ControlStr::from(#pp).unwrap();}
            }
            Expr::Reference(er) => match er.expr.as_ref() {
                Expr::Path(syn::ExprPath { attrs, qself, path }) => {
                    let pp = path
                        .get_ident()
                        .unwrap_or_else(|| panic!("path get ident failed"));
                    quote! {let cs = control_str::ControlStr::from(&#pp).unwrap();}
                }
                _ => panic!("the first arg have to be &str"),
            },
            _ => panic!("the first arg have to be &str"),
        },
        None => return proc_macro2::TokenStream::new().into(),
    };

    //dbg!(cs.to_string());
    //dbg!(items.len());
    let args = args_picker(items);
    //dbg!(args.to_string());

    let q = quote! {{
        #cs
        let args = #args;
        cs.reveal(args)
    }};
    //println!("result: \n{}", q.to_string());
    q.into()
}

// abandon, use the one in cl-format
fn args_picker(mut pairs: syn::punctuated::Pairs<Expr, Token![,]>) -> proc_macro2::TokenStream {
    let mut result = vec![];
    loop {
        match pairs.next() {
            Some(a) => match a.value() {
                Expr::Path(syn::ExprPath { attrs, qself, path }) => {
                    let pp = path
                        .get_ident()
                        .unwrap_or_else(|| panic!("path get ident failed"));
                    result.push(quote! {#pp as &dyn tildes::TildeAble})
                }
                Expr::Reference(er) => match er.expr.as_ref() {
                    Expr::Path(syn::ExprPath { attrs, qself, path }) => {
                        let pp = path
                            .get_ident()
                            .unwrap_or_else(|| panic!("path get ident failed"));
                        result.push(quote! {&#pp as &dyn tildes::TildeAble})
                    }
                    Expr::Lit(l) => {
                        let x = match &l.lit {
                            syn::Lit::Str(x) => x.to_token_stream(),
                            syn::Lit::ByteStr(x) => x.to_token_stream(),
                            syn::Lit::Byte(x) => x.to_token_stream(),
                            syn::Lit::Char(x) => x.to_token_stream(),
                            syn::Lit::Int(x) => x.to_token_stream(),
                            syn::Lit::Float(x) => x.to_token_stream(),
                            syn::Lit::Bool(x) => x.to_token_stream(),
                            syn::Lit::Verbatim(x) => x.to_token_stream(),
                            _ => unreachable!(),
                        };

                        result.push(quote! {&#x as &dyn tildes::TildeAble})
                    }
                    _ => panic!("unsupport"),
                },
                // temporary value lifetime issue
                // Expr::Array(a) => {
                //     let a = args_picker(a.elems.pairs());
                //     result.push(quote! {&#a as &dyn tildes::TildeAble})
                // }
                _ => panic!("only accept Path, Referance, and Array"),
            },
            None => {
                return quote! {
                    Into::<tildes::Args<'_>>::into([
                        #(
                            #result,
                        )*
                    ])
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::{parse::Parser, parse2, parse_quote, Variant};

    #[test]
    fn test_get_types_impl_to() -> Result<(), Box<dyn Error>> {
        let test_case: Attribute = parse_quote! {
                #[implTo(a,b,c,d)]
        };

        //dbg!(test_case);
        assert_eq!(
            vec!["a", "b", "c", "d"]
                .into_iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
            get_types_impl_to(&test_case)
                .unwrap()
                .into_iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
        );

        let test_case: Attribute = parse_quote! {
                #[implTo(a)]
        };

        //dbg!(test_case);
        assert_eq!(
            vec!["a"]
                .into_iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
            get_types_impl_to(&test_case)
                .unwrap()
                .into_iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
        );

        Ok(())
    }

    #[test]
    fn test_parse_variant_attrs() -> Result<(), Box<dyn Error>> {
        let test_case: Variant = parse_quote! {
            #[implTo(a,b,c,d)]
            A
        };

        //dbg!(test_case);
        let result = parse_variant_attrs(&test_case);
        assert_eq!(result.0, "A");
        assert_eq!(
            result.1.map(|i| i.to_string()).collect::<Vec<_>>(),
            vec!["a", "b", "c", "d"]
                .into_iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
        );

        //
        let test_case: Variant = parse_quote! {
            B
        };

        //dbg!(&test_case);
        let mut result = parse_variant_attrs(&test_case);
        assert_eq!(result.0, "B");
        assert_eq!(result.1.next(), None);

        Ok(())
    }

    #[test]
    fn test_args_picker() -> Result<(), Box<dyn Error>> {
        //let s: syn::Expr = syn::parse_str("a!(a1, &a2, a3)")?;
        //let s: Punctuated<Expr, Token![,]> = syn::parse_str("a!(a1, &a2, a3)")?;
        // let s: TokenStream = "a1, &a2, a3, [[&3]]".parse().unwrap();
        // let items = Punctuated::<Expr, Token![,]>::parse_terminated
        //     .parse(s.into())
        //     .unwrap();
        // dbg!(items);

        Ok(())
    }
}
