/***************************************************************************
 *
 * osal-rs-serde-derive
 * Copyright (C) 2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, see <https://www.gnu.org/licenses/>.
 *
 ***************************************************************************/

//! Derive macros for osal-rs-serde
//!
//! This crate provides `#[derive(Serialize, Deserialize)]` macros for automatic
//! implementation of serialization traits. Enable them through the `derive`
//! feature of `osal-rs-serde` rather than depending on this crate directly.
//!
//! Named-field structs, tuple structs and unit structs are supported. Fields are
//! serialized in declaration order; tuple struct fields are named `"0"`, `"1"`, ...
//!
//! # Examples
//!
//! ```
//! use osal_rs_serde::{Serialize, Deserialize, to_bytes, from_bytes};
//!
//! #[derive(Serialize, Deserialize, Debug, PartialEq)]
//! struct SensorData {
//!     temperature: i16,
//!     humidity: u8,
//!     pressure: u32,
//! }
//!
//! #[derive(Serialize, Deserialize, Debug, PartialEq)]
//! struct Color(u8, u8, u8);
//!
//! #[derive(Serialize, Deserialize, Debug, PartialEq)]
//! struct Marker;
//!
//! let mut buffer = [0u8; 16];
//!
//! let data = SensorData { temperature: -5, humidity: 60, pressure: 1013 };
//! let len = to_bytes(&data, &mut buffer).unwrap();
//! assert_eq!(len, 7);
//! assert_eq!(from_bytes::<SensorData>(&buffer[..len]).unwrap(), data);
//!
//! let color = Color(255, 128, 0);
//! let len = to_bytes(&color, &mut buffer).unwrap();
//! assert_eq!(&buffer[..len], &[255, 128, 0]);
//! assert_eq!(from_bytes::<Color>(&buffer[..len]).unwrap(), color);
//!
//! // Unit structs carry no data
//! assert_eq!(to_bytes(&Marker, &mut buffer).unwrap(), 0);
//! assert_eq!(from_bytes::<Marker>(&[]).unwrap(), Marker);
//! ```
//!
//! # Limitations
//!
//! Enums are not supported yet:
//!
//! ```compile_fail
//! use osal_rs_serde::Serialize;
//!
//! #[derive(Serialize)]
//! enum Status {
//!     Active,
//!     Inactive,
//! }
//! ```
//!
//! Unions are not supported:
//!
//! ```compile_fail
//! use osal_rs_serde::Serialize;
//!
//! #[derive(Serialize)]
//! union Data {
//!     integer: i32,
//!     float: f32,
//! }
//! ```
//!
//! Generic structs are not supported yet:
//!
//! ```compile_fail
//! use osal_rs_serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct Container<T> {
//!     value: T,
//! }
//! ```


// Compile the README examples as doctests so they cannot go stale
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

use proc_macro::TokenStream;
use quote::quote;
use proc_macro2::Literal;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// Derive macro for the `Serialize` trait.
///
/// Automatically implements serialization for structs with named or unnamed fields.
/// The generated code calls `serialize_struct_start`, then `serialize_field` for each
/// field in declaration order, then `serialize_struct_end`.
///
/// # Examples
///
/// ```
/// use osal_rs_serde::{Serialize, to_bytes};
///
/// #[derive(Serialize)]
/// struct Point {
///     x: i32,
///     y: i32,
/// }
///
/// let mut buffer = [0u8; 8];
/// let len = to_bytes(&Point { x: 1, y: -1 }, &mut buffer).unwrap();
///
/// // Little-endian, fields in declaration order
/// assert_eq!(len, 8);
/// assert_eq!(&buffer[..4], &1i32.to_le_bytes());
/// assert_eq!(&buffer[4..8], &(-1i32).to_le_bytes());
/// ```
#[proc_macro_derive(Serialize)]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;


    let Data::Struct(data_struct) = &input.data else {
        let message = match &input.data {
            Data::Enum(_) => "Serialize derive macro does not support enums yet",
            Data::Union(_) => "Serialize derive macro does not support unions",
            Data::Struct(_) => unreachable!(),
        };
        return syn::Error::new_spanned(name, message).to_compile_error().into();
    };

    let serialize_impl = match &data_struct.fields {
        Fields::Named(fields) => {
            let field_count = fields.named.len();
            let field_serializations = fields.named.iter().map(|f| {
                let field_name = &f.ident;
                let field_name_str = field_name.as_ref().unwrap().to_string();
                quote! {
                    serializer.serialize_field(#field_name_str, &self.#field_name)?;
                }
            });

            quote! {
                impl osal_rs_serde::Serialize for #name {
                    fn serialize<S: osal_rs_serde::Serializer>(&self, name: &str, serializer: &mut S) -> Result<(), S::Error> {
                        serializer.serialize_struct_start(name, #field_count)?;
                        #(#field_serializations)*
                        serializer.serialize_struct_end()?;
                        Ok(())
                    }
                }
            }
        }
        Fields::Unnamed(fields) => {
            let field_count = fields.unnamed.len();
            let field_serializations = (0..fields.unnamed.len()).map(|i| {
                let index = syn::Index::from(i);
                let name_lit = Literal::string(&i.to_string());
                quote! {
                    serializer.serialize_field(#name_lit, &self.#index)?;
                }
            });

            quote! {
                impl osal_rs_serde::Serialize for #name {
                    fn serialize<S: osal_rs_serde::Serializer>(&self, name: &str, serializer: &mut S) -> Result<(), S::Error> {
                        serializer.serialize_struct_start(name, #field_count)?;
                        #(#field_serializations)*
                        serializer.serialize_struct_end()?;
                        Ok(())
                    }
                }
            }
        }
        Fields::Unit => {
            quote! {
                impl osal_rs_serde::Serialize for #name {
                    fn serialize<S: osal_rs_serde::Serializer>(&self, _name: &str, _serializer: &mut S) -> Result<(), S::Error> {
                        Ok(())
                    }
                }
            }
        }
    };

    TokenStream::from(serialize_impl)
}

/// Derive macro for the `Deserialize` trait.
///
/// Automatically implements deserialization for structs with named or unnamed fields.
/// The generated code mirrors [`Serialize`](macro@Serialize): `deserialize_struct_start`,
/// then `deserialize_field` for each field in declaration order, then `deserialize_struct_end`.
///
/// # Examples
///
/// ```
/// use osal_rs_serde::{Deserialize, from_bytes};
///
/// #[derive(Deserialize, Debug, PartialEq)]
/// struct Point {
///     x: i32,
///     y: i32,
/// }
///
/// let mut bytes = [0u8; 8];
/// bytes[..4].copy_from_slice(&1i32.to_le_bytes());
/// bytes[4..].copy_from_slice(&(-1i32).to_le_bytes());
///
/// assert_eq!(from_bytes::<Point>(&bytes).unwrap(), Point { x: 1, y: -1 });
///
/// // Too few bytes is an error, not a partially filled struct
/// assert!(from_bytes::<Point>(&bytes[..6]).is_err());
/// ```
#[proc_macro_derive(Deserialize)]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let Data::Struct(data_struct) = &input.data else {
        let message = match &input.data {
            Data::Enum(_) => "Deserialize derive macro does not support enums yet",
            Data::Union(_) => "Deserialize derive macro does not support unions",
            Data::Struct(_) => unreachable!(),
        };
        return syn::Error::new_spanned(name, message).to_compile_error().into();
    };

    let deserialize_impl = match &data_struct.fields {
        Fields::Named(fields) => {
            let field_deserializations = fields.named.iter().map(|f| {
                let field_name = &f.ident;
                let field_name_str = field_name.as_ref().unwrap().to_string();
                let field_type = &f.ty;
                quote! {
                    #field_name: deserializer.deserialize_field::<#field_type>(#field_name_str)?
                }
            });

            quote! {
                impl osal_rs_serde::Deserialize for #name {
                    fn deserialize<D: osal_rs_serde::Deserializer>(deserializer: &mut D, name: &str) -> Result<Self, D::Error> {
                        deserializer.deserialize_struct_start(name)?;
                        let result = Self {
                            #(#field_deserializations,)*
                        };
                        deserializer.deserialize_struct_end()?;
                        Ok(result)
                    }
                }
            }
        }
        Fields::Unnamed(fields) => {
            let field_deserializations = fields.unnamed.iter().enumerate().map(|(i, f)| {
                let name_lit = Literal::string(&i.to_string());
                let field_type = &f.ty;
                quote! {
                    deserializer.deserialize_field::<#field_type>(#name_lit)?
                }
            });

            quote! {
                impl osal_rs_serde::Deserialize for #name {
                    fn deserialize<D: osal_rs_serde::Deserializer>(deserializer: &mut D, name: &str) -> Result<Self, D::Error> {
                        deserializer.deserialize_struct_start(name)?;
                        let result = Self(
                            #(#field_deserializations,)*
                        );
                        deserializer.deserialize_struct_end()?;
                        Ok(result)
                    }
                }
            }
        }
        Fields::Unit => {
            quote! {
                impl osal_rs_serde::Deserialize for #name {
                    fn deserialize<D: osal_rs_serde::Deserializer>(_deserializer: &mut D, _name: &str) -> Result<Self, D::Error> {
                        Ok(Self)
                    }
                }
            }
        }
    };

    TokenStream::from(deserialize_impl)
}
