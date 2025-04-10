//! Derive macros for rioc
//!
//! This crate provides procedural macros for the rioc framework.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, GenericParam, TypeParam};


#[proc_macro_attribute]
pub fn injected(_attr: TokenStream, annotated: TokenStream) -> TokenStream {
    annotated
}

/// Generates Provider trait implementation for a type
#[proc_macro_derive(IProvider,attributes(inject))]
pub fn derive_provider(input: TokenStream) -> TokenStream {
    // Parse input TokenStream
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    // Process generic parameters
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    
    // Build where clause
    let mut where_predicates = where_clause.map(|w| w.predicates.clone().into_iter().collect())
        .unwrap_or_else(Vec::new); 

    // Add Clone + 'static bounds for each type parameter
    for param in generics.params.iter() {
        if let GenericParam::Type(TypeParam { ident, .. }) = param {
            where_predicates.push(syn::parse_quote!(#ident: Clone + 'static));
        }
    }

    // Process fields with #[inject] attribute
    // let mut field_inits: Vec<TokenStream> = Vec::new();
    // if let syn::Data::Struct(data_struct) = input.data {
    //     for field in data_struct.fields {
    //         if let Some(attr) = field.attrs.iter().find(|a| a.path().is_ident("inject")) {
    //             let field_name = field.ident.unwrap();
    //             let field_ty = field.ty;
    //             // Extract name attribute if present

    //         }
    //     }
    // }


    let where_clause = if !where_predicates.is_empty() {
        quote! { where #(#where_predicates),* }
    } else {
        quote! {}
    };

    // Generate implementation code
    let expanded = quote! {
        impl #impl_generics rioc::Provider for #name #ty_generics #where_clause {           
            fn instantiate<C: rioc::Container>(&self, c: &C) -> ::std::boxed::Box<Self> {
                Box::new(self.clone())
            }
            
            fn as_any(&self) -> &dyn ::std::any::Any {
                self
            }
        }
    };

    // Convert generated code back to TokenStream
    TokenStream::from(expanded)
}