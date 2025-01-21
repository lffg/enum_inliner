use std::{fmt, mem};

use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    visit_mut::VisitMut,
    GenericParam, Ident, ImplItemFn, ItemEnum, ItemImpl, Macro, Result, Type,
};

#[derive(Debug)]
struct EnumInlinerInput {
    enum_item: ItemEnum,
    item_impl: ItemImpl,
}

impl Parse for EnumInlinerInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let enum_item = input.parse()?;
        let item_impl = input.parse()?;

        if !input.is_empty() {
            let span = input.cursor().span();
            return Err(syn::Error::new(
                span,
                "can only have two items: enum and impl block",
            ));
        }

        Ok(EnumInlinerInput {
            enum_item,
            item_impl,
        })
    }
}

/// Check that the identifier of the impl block is the same as the enum item.
fn match_identifiers(enum_item: &ItemEnum, item_impl: &ItemImpl) -> Result<()> {
    let enum_ident = enum_item.ident.clone();

    let Type::Path(self_ty) = &*item_impl.self_ty else {
        return Err(error_at(
            &item_impl.self_ty,
            "must be the same ident as the enum",
        ));
    };
    let Some(impl_ident) = self_ty.path.get_ident() else {
        return Err(error_at(
            &item_impl.self_ty,
            "must be the same ident as the enum",
        ));
    };

    if enum_ident != *impl_ident {
        return Err(error_at(impl_ident, "must be the same ident as the enum"));
    }

    Ok(())
}

/// Extracts the "placeholder parameter", which is identified by having the
/// `ident` type. As in the following example:
///
/// ```ignore
/// impl Foo<const __VARIANT__: ident> { .. }
/// ```
///
/// Returns the identifier in case of a match.
fn placeholder_param(param: &GenericParam) -> Option<&Ident> {
    let GenericParam::Const(p) = param else {
        return None;
    };
    let Type::Path(ty) = &p.ty else {
        return None;
    };
    let ty_ident = ty.path.get_ident()?;
    if ty_ident != "ident" {
        return None;
    }
    Some(&p.ident)
}

fn get_placeholder_param(item_impl: &ItemImpl) -> Result<&Ident> {
    let mut ident_params = item_impl
        .generics
        .params
        .iter()
        .filter_map(placeholder_param);
    let Some(ident) = ident_params.next() else {
        return Err(error_at(
            item_impl.impl_token,
            "missing a placeholder identifier definition",
        ));
    };
    if let Some(next) = ident_params.next() {
        return Err(error_at(next, "can only have one placeholder identifier"));
    }
    Ok(ident)
}

fn remove_placeholder_ident_generic_param(item_impl: &mut ItemImpl) {
    item_impl.generics.params = mem::take(&mut item_impl.generics.params)
        .into_iter()
        .filter(|p| {
            let is_placeholder = placeholder_param(p).is_some();
            !is_placeholder
        })
        .collect();
}

/// The inliner takes every `fn` item's block. For each one, the function's
/// block is copied for every variant of the target enum, with the placeholder
/// `target` ident being replaced by the corresponding variant name.
struct Inliner {
    variants: Vec<Ident>,
    target_placeholder: Ident,
}

impl VisitMut for Inliner {
    fn visit_impl_item_fn_mut(&mut self, f: &mut ImplItemFn) {
        let arms = self.variants.iter().map(|variant| {
            let mut body = f.block.clone();

            let mut replacer = PlaceholderReplacer {
                target_placeholder: self.target_placeholder.clone(),
                substitute: variant.clone(),
            };
            replacer.visit_block_mut(&mut body);

            quote! {
                Self::#variant => { #body }
            }
        });

        f.block = parse_quote! {
            {
                match self {
                    #(#arms)*
                }
            }
        };
    }
}

struct PlaceholderReplacer {
    target_placeholder: Ident,
    substitute: Ident,
}

impl VisitMut for PlaceholderReplacer {
    fn visit_ident_mut(&mut self, i: &mut Ident) {
        if *i == self.target_placeholder {
            *i = replaced_ident(i.span(), self.substitute.clone());
        }
    }

    fn visit_macro_mut(&mut self, i: &mut Macro) {
        i.tokens = mem::take(&mut i.tokens)
            .into_iter()
            .map(|tt| match tt {
                TokenTree::Ident(ident) if ident == self.target_placeholder => {
                    TokenTree::from(replaced_ident(ident.span(), self.substitute.clone()))
                }
                tt => tt,
            })
            .collect();
    }
}

fn expand(input: EnumInlinerInput) -> Result<TokenStream> {
    let EnumInlinerInput {
        enum_item,
        mut item_impl,
    } = input;

    match_identifiers(&enum_item, &item_impl)?;

    let placeholder_ident = get_placeholder_param(&item_impl)?.clone();
    remove_placeholder_ident_generic_param(&mut item_impl);

    let variants = enum_item.variants.iter().map(|v| v.ident.clone()).collect();

    let mut inliner = Inliner {
        variants,
        target_placeholder: placeholder_ident,
    };
    inliner.visit_item_impl_mut(&mut item_impl);

    Ok(quote! {
        #enum_item

        #item_impl
    })
}

fn replaced_ident(old_span: Span, mut new: Ident) -> Ident {
    new.set_span(old_span);
    new
}

#[proc_macro]
pub fn enum_inline(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as EnumInlinerInput);

    expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn error_at<T: ToTokens, M: fmt::Display>(spanned_by: T, msg: M) -> syn::Error {
    syn::Error::new_spanned(spanned_by.into_token_stream(), msg)
}
