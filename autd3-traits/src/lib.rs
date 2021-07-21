/*
 * File: lib.rs
 * Project: src
 * Created Date: 24/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 21/07/2021
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
            fn build(&mut self) -> Result<()>{
                self.calc()
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
            fn sampling_frequency_division(&self) -> u16 {
                self.sampling_freq_div
            }
            fn sampling_freq(&self) -> f64 {
                autd3_core::hardware_defined::MOD_SAMPLING_FREQ_BASE as f64 / self.sampling_freq_div as f64
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

            fn take(self) -> Vec<DataArray>{
                self.data
            }

            fn built(&self) -> bool {
                self.built
            }
        }
    };
    gen.into()
}

#[proc_macro_derive(Sequence)]
pub fn sequence_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_sequence_macro(&ast)
}

fn impl_sequence_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let gen = quote! {
        impl #impl_generics Sequence for #name #ty_generics #where_clause {
            fn set_freq(&mut self, freq: f64) -> f64 {
                let sample_freq = self.size() as f64 * freq;
                let div = (SEQ_BASE_FREQ as f64 / sample_freq) as u16;
                self.sample_freq_div = div;
                self.freq()
            }

            fn freq(&self) -> f64 {
                self.sampling_freq() / self.size() as f64
            }

            fn sampling_freq(&self) -> f64 {
                SEQ_BASE_FREQ as f64 / self.sample_freq_div as f64
            }

            fn sampling_freq_div(&self) -> u16 {
                self.sample_freq_div
            }

            fn sent(&self) -> usize {
                self.sent
            }

            fn send(&mut self, sent: usize) {
                self.sent += sent
            }

            fn finished(&self) -> bool {
                self.remaining() == 0
            }
        }
    };
    gen.into()
}
