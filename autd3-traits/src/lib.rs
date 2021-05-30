/*
 * File: lib.rs
 * Project: src
 * Created Date: 24/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 30/05/2021
 * Modified By: Shun Suzuki (suzuki@hapis.k.u-tokyo.ac.jp)
 * -----
 * Copyright (c) 2021 Hapis Lab. All rights reserved.
 *
 */

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(Modulation)]
pub fn modulation_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_modulation_macro(&ast)
}

fn impl_modulation_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let gen = quote! {
        impl #impl_generics Modulation for #name #ty_generics #where_clause {
            fn build(&mut self, config: Configuration) -> Result<()>{
                self.calc(config)
            }
            fn buffer(&self) -> &[u8] {
                &self.buffer
            }
            fn remaining(&self) -> usize {
                self.buffer().len() - self.sent()
            }
            fn head(&self) -> *const u8 {
                unsafe { self.buffer().as_ptr().add(self.sent()) }
            }
            fn sent(&self) -> usize {
                self.sent
            }
            fn send(&mut self, sent: usize){
                self.sent += sent;
            }
        }
    };
    gen.into()
}

#[proc_macro_derive(Gain)]
pub fn gain_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_gain_macro(&ast)
}

fn impl_gain_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let gen = quote! {
        impl #impl_generics Gain for #name #ty_generics #where_clause {
            fn build(&mut self, geometry: &Geometry) -> Result<()>{
                if self.built() {return Ok(())}
                let buf: DataArray = unsafe { std::mem::zeroed() };
                self.data = vec![buf; geometry.num_devices()];
                self.calc(geometry)
            }

            fn data(&self) -> &[DataArray]{
                &self.data
            }

            fn built(&self) -> bool {
                self.built
            }
        }
    };
    gen.into()
}
