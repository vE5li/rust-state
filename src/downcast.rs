//! Module providing a trait and implementation to downcast [`Box<dyn Any>`]
//! paths in the state.
//!
//! This allows dynamically typed paths (`Box<dyn Any>`) to be downcast into a
//! concrete type.
//!
//! Example:
//! ```
//! #![feature(auto_traits, negative_impls)]
//! use rust_state::{State, DowncastExt, Path, RustState};
//! use std::any::Any;
//!
//! #[derive(Debug, PartialEq, Eq)]
//! struct Inner {
//!     value: u32,
//! }
//!
//! #[derive(RustState)]
//! #[state_root]
//! struct MyState {
//!     dynamic: Box<dyn Any>,
//! }
//!
//! let state = State::new(MyState {
//!     dynamic: Box::new(Inner { value: 99 }),
//! });
//!
//! let path = MyState::path().dynamic().downcast::<Inner>();
//!
//! assert_eq!(state.try_get(&path), Some(&Inner { value: 99 }));
//! ```

use std::any::Any;
use std::marker::PhantomData;

use crate::{Path, Selector};

/// A path that downcasts a [`Box<dyn Any>`] in the state to a specific concrete
/// type.
///
/// This type is not accessible outside this module. Instead,
/// [`DowncastExt`] is used to construct it and receive an `impl Path<State,
/// T>`.
struct DowncastPath<State, AnyPath, To, const SAFE: bool> {
    path: AnyPath,
    _marker: PhantomData<(State, To)>,
}

impl<State, AnyPath, To, const SAFE: bool> Clone for DowncastPath<State, AnyPath, To, SAFE>
where
    AnyPath: Path<State, Box<dyn Any>, SAFE>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<State, AnyPath, To, const SAFE: bool> Copy for DowncastPath<State, AnyPath, To, SAFE> where AnyPath: Path<State, Box<dyn Any>, SAFE> {}

impl<State, AnyPath, To, const SAFE: bool> Selector<State, To, false> for DowncastPath<State, AnyPath, To, SAFE>
where
    State: 'static,
    AnyPath: Path<State, Box<dyn Any>, SAFE>,
    To: 'static,
{
    fn select<'a>(&'a self, state: &'a State) -> Option<&'a To> {
        self.follow(state)
    }
}

impl<State, AnyPath, To, const SAFE: bool> Path<State, To, false> for DowncastPath<State, AnyPath, To, SAFE>
where
    State: 'static,
    AnyPath: Path<State, Box<dyn Any>, SAFE>,
    To: 'static,
{
    fn follow<'a>(&self, state: &'a State) -> Option<&'a To> {
        self.path.follow(state).and_then(|any| any.downcast_ref::<To>())
    }

    fn follow_mut<'a>(&self, state: &'a mut State) -> Option<&'a mut To> {
        self.path.follow_mut(state).and_then(|any| any.downcast_mut::<To>())
    }
}

pub trait DowncastExt<State, AnyPath, const SAFE: bool>
where
    State: 'static,
    Self: Path<State, Box<dyn Any>, SAFE>,
{
    fn downcast<To: 'static>(self) -> impl Path<State, To, false> {
        DowncastPath {
            path: self,
            _marker: PhantomData,
        }
    }
}

impl<State, T, const SAFE: bool> DowncastExt<State, T, SAFE> for T
where
    State: 'static,
    T: Path<State, Box<dyn Any>, SAFE>,
{
}
