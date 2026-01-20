/***************************************************************************
 *
 * osal-rs-serde-derive
 * Copyright (C) 2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 ***************************************************************************/

//! Derive macros for osal-rs-serde
//!
//! This crate provides `#[derive(Serialize, Deserialize)]` macros for automatic
//! implementation of serialization traits.
//!
//! # Examples
//!
//! ```ignore
//! use osal_rs_serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct SensorData {
//!     temperature: i16,
//!     humidity: u8,
//!     pressure: u32,
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// Derive macro for the `Serialize` trait.
///
/// Automatically implements serialization for structs with named or unnamed fields.
///
/// # Examples
///
/// ```ignore
/// #[derive(Serialize)]
/// struct Point {
///     x: i32,
///     y: i32,
/// }
/// ```
#[proc_macro_derive(Serialize)]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let serialize_impl = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => {
                let field_serializations = fields.named.iter().map(|f| {
                    let field_name = &f.ident;
                    quote! {
                        self.#field_name.serialize(serializer)?;
                    }
                });

                quote! {
                    impl osal_rs_serde::Serialize for #name {
                        fn serialize<S: osal_rs_serde::Serializer>(&self, serializer: &mut S) -> core::result::Result<(), S::Error> {
                            #(#field_serializations)*
                            Ok(())
                        }
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let field_serializations = (0..fields.unnamed.len()).map(|i| {
                    let index = syn::Index::from(i);
                    quote! {
                        self.#index.serialize(serializer)?;
                    }
                });

                quote! {
                    impl osal_rs_serde::Serialize for #name {
                        fn serialize<S: osal_rs_serde::Serializer>(&self, serializer: &mut S) -> core::result::Result<(), S::Error> {
                            #(#field_serializations)*
                            Ok(())
                        }
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    impl osal_rs_serde::Serialize for #name {
                        fn serialize<S: osal_rs_serde::Serializer>(&self, _serializer: &mut S) -> core::result::Result<(), S::Error> {
                            Ok(())
                        }
                    }
                }
            }
        },
        Data::Enum(_) => {
            return syn::Error::new_spanned(
                name,
                "Serialize derive macro does not support enums yet"
            )
            .to_compile_error()
            .into();
        }
        Data::Union(_) => {
            return syn::Error::new_spanned(
                name,
                "Serialize derive macro does not support unions"
            )
            .to_compile_error()
            .into();
        }
    };

    TokenStream::from(serialize_impl)
}

/// Derive macro for the `Deserialize` trait.
///
/// Automatically implements deserialization for structs with named or unnamed fields.
///
/// # Examples
///
/// ```ignore
/// #[derive(Deserialize)]
/// struct Point {
///     x: i32,
///     y: i32,
/// }
/// ```
#[proc_macro_derive(Deserialize)]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let deserialize_impl = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => {
                let field_names = fields.named.iter().map(|f| &f.ident);
                let field_types = fields.named.iter().map(|f| &f.ty);

                quote! {
                    impl osal_rs_serde::Deserialize for #name {
                        fn deserialize<D: osal_rs_serde::Deserializer>(deserializer: &mut D) -> core::result::Result<Self, D::Error> {
                            Ok(Self {
                                #(#field_names: <#field_types as osal_rs_serde::Deserialize>::deserialize(deserializer)?,)*
                            })
                        }
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let field_types = fields.unnamed.iter().map(|f| &f.ty);

                quote! {
                    impl osal_rs_serde::Deserialize for #name {
                        fn deserialize<D: osal_rs_serde::Deserializer>(deserializer: &mut D) -> core::result::Result<Self, D::Error> {
                            Ok(Self(
                                #(<#field_types as osal_rs_serde::Deserialize>::deserialize(deserializer)?,)*
                            ))
                        }
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    impl osal_rs_serde::Deserialize for #name {
                        fn deserialize<D: osal_rs_serde::Deserializer>(_deserializer: &mut D) -> core::result::Result<Self, D::Error> {
                            Ok(Self)
                        }
                    }
                }
            }
        },
        Data::Enum(_) => {
            return syn::Error::new_spanned(
                name,
                "Deserialize derive macro does not support enums yet"
            )
            .to_compile_error()
            .into();
        }
        Data::Union(_) => {
            return syn::Error::new_spanned(
                name,
                "Deserialize derive macro does not support unions"
            )
            .to_compile_error()
            .into();
        }
    };

    TokenStream::from(deserialize_impl)
}
