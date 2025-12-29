//! Rust FFI bindings for rtad C library.
//!
//! rtad is a cross-platform library for appending data to executable files
//! and reading it back at runtime.
//!
//! All bindings are automatically generated from rtad.h using bindgen.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// Include the auto-generated bindings from build.rs
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
