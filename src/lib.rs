#![feature(auto_traits)]
#![feature(negative_impls)]

// Reexport self as `rust_state` so that the derive macro works in this crate.
extern crate self as rust_state;

mod array;
mod as_ref;
mod downcast;
mod manual;
mod map;
mod option;
mod path;
mod state;
mod vec;

pub use array::ArrayLookupExt;
pub use as_ref::AsRefExt;
pub use downcast::DowncastExt;
pub use macros::RustState;
pub use manual::ManuallyAssertExt;
pub use map::{MapItem, MapLookupExt};
pub use option::OptionExt;
pub use path::{AutoImplSelector, Path, PathExt, Selector, SelectorExt};
pub use state::{State, StateMarker, UpdateError};
pub use vec::{VecIndexExt, VecItem, VecLookupExt};
