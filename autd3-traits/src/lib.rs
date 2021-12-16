/*
 * File: lib.rs
 * Project: src
 * Created Date: 24/05/2021
 * Author: Shun Suzuki
 * -----
 * Last Modified: 16/12/2021
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
        fn build(&mut self) -> Result<()> {
            if self.built { return Ok(()); }
            if self.sampling_freq_div > hardware_defined::MOD_SAMPLING_FREQ_DIV_MAX { return Err(autd3_core::error::AutdError::FrequencyDivisionRatioOutOfRange(hardware_defined::MOD_SAMPLING_FREQ_DIV_MAX).into()); }
            let r = self.calc();
            if self.buffer().len() > hardware_defined::MOD_BUF_SIZE_MAX{ return Err(autd3_core::error::AutdError::ModulationOutOfBuffer(hardware_defined::MOD_BUF_SIZE_MAX).into()); }
            self.built = true;
            r
        }
        fn rebuild(&mut self) -> Result<()>{
            self.built = false;
            self.build()
        }
        fn buffer(&self) -> &[u8] {
            &self.buffer
        }
        fn sampling_frequency_division(&mut self) -> &mut usize {
            &mut self.sampling_freq_div
        }
        fn sampling_freq(&self) -> f64 {
            hardware_defined::MOD_SAMPLING_FREQ_BASE as f64 / self.sampling_freq_div as f64
        }
    }
    impl #impl_generics autd3_core::interface::IDatagramHeader for #name #ty_generics #where_clause {
        fn init(&mut self) -> Result<()> {
            self.build()?;
            self.sent = 0;
            Ok(())
        }
        fn pack(
            &mut self,
            msg_id: u8,
            tx: &mut autd3_core::hardware_defined::TxDatagram,
            fpga_flag: autd3_core::hardware_defined::FPGAControlFlags,
            cpu_flag: autd3_core::hardware_defined::CPUControlFlags,
        ){
            let mut header = tx.header_mut();
            header.msg_id = msg_id;
            let fpga_mask = autd3_core::hardware_defined::FPGAControlFlags::OUTPUT_BALANCE | autd3_core::hardware_defined::FPGAControlFlags::SILENT |  autd3_core::hardware_defined::FPGAControlFlags::FORCE_FAN;
            header.fpga_flag = (header.fpga_flag & !fpga_mask) | (fpga_flag & fpga_mask);
            header.cpu_flag = cpu_flag;
            header.mod_size = 0;

            tx.set_num_bodies(0);

            if self.is_finished() { return; }

            let mut offset = 0;
            let mut header = tx.header_mut();
            if self.sent == 0 {
              header.cpu_flag |= autd3_core::hardware_defined::CPUControlFlags::MOD_BEGIN;
              let div = (self.sampling_freq_div - 1) as u16;
              header.mod_data[0] = (div & 0xFF) as u8;
              header.mod_data[1] = (div >> 8 & 0xFF) as u8;
              offset += 2;
            }
            let mod_size = (self.buffer.len() - self.sent).clamp(0, autd3_core::hardware_defined::MOD_FRAME_SIZE - offset);
            if self.sent + mod_size >= self.buffer.len() { header.cpu_flag |= autd3_core::hardware_defined::CPUControlFlags::MOD_END; }
            header.mod_size = mod_size as _;

            unsafe{
                std::ptr::copy_nonoverlapping(self.buffer[self.sent..].as_ptr(), header.mod_data[offset..].as_mut_ptr(), mod_size);
            }
            self.sent += mod_size;
        }
        fn is_finished(&self) -> bool {
            self.sent == self.buffer.len()
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
            fn build(&mut self, geometry: &Geometry) -> Result<()> {
                if self.built {return Ok(())}
                self.data.resize(geometry.num_devices(), Drive{duty: 0x00, phase: 0x00});
                self.calc(geometry)?;
                self.built = true;
                Ok(())
            }

            fn rebuild(&mut self, geometry: &Geometry) -> Result<()>{
                self.built = false;
                self.build(geometry)
            }

            fn data(&self) -> &[Drive]{
                &self.data
            }

            fn take(self) -> Vec<Drive>{
                self.data
            }

            fn built(&self) -> bool {
                self.built
            }
        }

        impl #impl_generics autd3_core::interface::IDatagramBody for #name #ty_generics #where_clause {
            fn init(&mut self) {}
            fn pack(
                &mut self,
                geometry: &Geometry,
                tx: &mut autd3_core::hardware_defined::TxDatagram,
            )-> Result<()> {
                self.build(geometry)?;

                let header = tx.header_mut();
                header.fpga_flag |= autd3_core::hardware_defined::FPGAControlFlags::OUTPUT_ENABLE;
                header.fpga_flag &= !autd3_core::hardware_defined::FPGAControlFlags::SEQ_MODE;
                header.cpu_flag |= autd3_core::hardware_defined::CPUControlFlags::WRITE_BODY;

                tx.body_data_mut::<Drive>().copy_from_slice(self.data());

                tx.set_num_bodies(geometry.num_devices());

                Ok(())
            }
            fn is_finished(&self) -> bool {
                true
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
                let div = ((SEQ_BASE_FREQ as f64 / sample_freq) as usize).clamp(1, hardware_defined::SEQ_SAMPLING_FREQ_DIV_MAX);
                self.sample_freq_div = div;
                self.freq()
            }

            fn freq(&self) -> f64 {
                self.sampling_freq() / self.size() as f64
            }

            fn sampling_freq(&self) -> f64 {
                SEQ_BASE_FREQ as f64 / self.sample_freq_div as f64
            }

            fn sampling_freq_div(&mut self) -> &mut usize {
                &mut self.sample_freq_div
            }

            fn wait_on_sync(&mut self) -> &mut bool {
                &mut self.wait_on_sync
            }
        }
    };
    gen.into()
}
