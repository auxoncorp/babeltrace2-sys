#![deny(warnings, clippy::all)]

#[allow(clippy::missing_safety_doc)]
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(unaligned_references)] // bindgen generated tests, https://github.com/rust-lang/rust-bindgen/issues/1651
#[allow(deref_nullptr)]
mod bindings;

#[allow(non_camel_case_types)]
pub mod ffi;

pub(crate) mod util;

mod clock;
mod component;
mod component_class;
mod ctf_plugin;
mod env;
mod error;
mod event;
mod field;
mod graph;
mod logger;
mod message;
mod message_iterator;
mod plugin;
mod port;
mod proxy_plugin;
mod self_component;
mod stream;
mod trace;
mod trace_iterator;
mod utils_plugin;
mod value;

pub use clock::*;
pub use component::*;
pub use component_class::*;
pub use ctf_plugin::*;
pub use env::*;
pub use error::*;
pub use event::*;
pub use field::*;
pub use graph::*;
pub use logger::*;
pub use message::*;
pub use message_iterator::*;
pub use plugin::*;
pub use port::*;
pub use proxy_plugin::*;
pub use self_component::*;
pub use stream::*;
pub use trace::*;
pub use trace_iterator::*;
pub use utils_plugin::*;
pub use value::*;
