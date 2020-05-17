#![allow(
    clippy::match_bool, // Code reads cleaner this way sometimes
    clippy::fn_to_numeric_cast, // Necessary for dynasm
    clippy::transmute_ptr_to_ptr // Makes code cleaner when the destination type is clear
)]
#![feature(proc_macro_hygiene)]

pub mod compiler;
pub mod cpu_state;
pub mod executor;
pub mod gb;
