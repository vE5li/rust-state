//! Module providing a trait and implementation to unwrap an [`Option`] path in
//! the state.
//!
//! This allows treating `Option<T>` fields as if they are `T` in a path, at the
//! cost of runtime checks. If the value is `None`, the path returns `None`.
//!
//! Example:
//! ```
//! #![feature(auto_traits, negative_impls)]
//! use rust_state::{State, ManuallyAssertExt, OptionExt, Path, RustState};
//!
//! #[derive(Debug, PartialEq, Eq)]
//! struct TestItem {
//!     value: usize,
//! }
//!
//! #[derive(RustState)]
//! #[state_root]
//! struct MyState {
//!     option: Option<TestItem>,
//! }
//!
//! let state = State::new(MyState {
//!     option: Some(TestItem { value: 20 }),
//! });
//!
//! let path = MyState::path().option().unwrapped();
//!
//! assert_eq!(state.try_get(&path), Some(&TestItem { value: 20 }));
//! ```

use std::marker::PhantomData;

use crate::{Path, Selector};

/// A path that unwraps an [`Option<T>`] in the state tree into a path of `T`.
///
/// This type is not accessible outside this module. Instead,
/// [`OptionExt`] is used to construct it and receive an `impl Path<State, T>`.
struct OptionUnwrapped<State, OptionPath, Unwrapped, const SAFE: bool> {
    option_path: OptionPath,
    _marker: PhantomData<(State, Unwrapped)>,
}

impl<State, OptionPath, Unwrapped, const SAFE: bool> Clone for OptionUnwrapped<State, OptionPath, Unwrapped, SAFE>
where
    OptionPath: Path<State, Option<Unwrapped>, SAFE>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<State, OptionPath, Unwrapped, const SAFE: bool> Copy for OptionUnwrapped<State, OptionPath, Unwrapped, SAFE> where
    OptionPath: Path<State, Option<Unwrapped>, SAFE>
{
}

impl<State, OptionPath, Unwrapped, const SAFE: bool> Selector<State, Unwrapped, false>
    for OptionUnwrapped<State, OptionPath, Unwrapped, SAFE>
where
    State: 'static,
    OptionPath: Path<State, Option<Unwrapped>, SAFE>,
    Unwrapped: 'static,
{
    fn select<'a>(&'a self, state: &'a State) -> Option<&'a Unwrapped> {
        self.option_path.follow(state).and_then(Option::as_ref)
    }
}

impl<State, OptionPath, Unwrapped, const SAFE: bool> Path<State, Unwrapped, false> for OptionUnwrapped<State, OptionPath, Unwrapped, SAFE>
where
    State: 'static,
    OptionPath: Path<State, Option<Unwrapped>, SAFE>,
    Unwrapped: 'static,
{
    fn follow<'a>(&self, state: &'a State) -> Option<&'a Unwrapped> {
        self.option_path.follow(state).and_then(Option::as_ref)
    }

    fn follow_mut<'a>(&self, state: &'a mut State) -> Option<&'a mut Unwrapped> {
        self.option_path.follow_mut(state).and_then(Option::as_mut)
    }
}

/// Extension trait providing `.unwrapped()` for [`Option<T>`] paths.
///
/// Converts a `Path<State, Option<T>>` into a `Path<State, T>`, returning
/// `None` if the inner value is `None`.
pub trait OptionExt<State, T, Unwrapped, const SAFE: bool>
where
    State: 'static,
    Self: Path<State, Option<Unwrapped>, SAFE>,
    Unwrapped: 'static,
{
    /// Unwrap the [`Option`] in this path, converting a `Path<State,
    /// Option<T>>` into a `Path<State, T>`.
    ///
    /// This path is *not* safe. It may return `None` at runtime if the
    /// underlying [`Option`] is `None`.
    fn unwrapped(self) -> impl Path<State, Unwrapped, false> {
        OptionUnwrapped {
            option_path: self,
            _marker: PhantomData,
        }
    }
}

impl<State, T, Unwrapped, const SAFE: bool> OptionExt<State, T, Unwrapped, SAFE> for T
where
    State: 'static,
    T: Path<State, Option<Unwrapped>, SAFE>,
    Unwrapped: 'static,
{
}
