//! Module providing a trait and implementation to dereference [`AsRef<T>`]
//! paths in the state.
//!
//! This allows paths that resolve to types implementing `AsRef<T>` to be
//! treated as if they point directly to `T`.
//!
//! Example:
//! ```
//! use rust_state::{AsRefExt, State, Path, RustState};
//!
//! #[derive(Debug, PartialEq, Eq)]
//! struct TestItem {
//!     value: u32,
//! }
//!
//! #[derive(RustState)]
//! #[state_root]
//! struct MyState {
//!     inner: Box<TestItem>,
//! }
//!
//! let state = State::new(MyState {
//!     inner: Box::new(TestItem { value: 42 }),
//! });
//!
//! let path = MyState::path().inner().path_as_ref();
//!
//! assert_eq!(state.get(&path), TestItem { value : 42 });
//! ```

use std::marker::PhantomData;

use crate::{Path, Selector};

/// A path that dereferences a type implementing [`AsRef<T>`] in the state into
/// a path of `T`.
///
/// This type is not accessible outside this module. Instead,
/// [`AsRefExt`] is used to construct it and receive an `impl Path<State, T>`.
struct AsRefPath<State, RefPath, Inner, Target, const SAFE: bool> {
    ref_path: RefPath,
    _marker: PhantomData<(State, Inner, Target)>,
}

impl<State, RefPath, Inner, Target, const SAFE: bool> Clone for AsRefPath<State, RefPath, Inner, Target, SAFE>
where
    RefPath: Path<State, Inner, SAFE>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<State, RefPath, Inner, Target, const SAFE: bool> Copy for AsRefPath<State, RefPath, Inner, Target, SAFE> where
    RefPath: Path<State, Inner, SAFE>
{
}

impl<State, RefPath, Inner, Target, const SAFE: bool> Selector<State, Target, SAFE> for AsRefPath<State, RefPath, Inner, Target, SAFE>
where
    State: 'static,
    RefPath: Path<State, Inner, SAFE>,
    Inner: AsRef<Target> + 'static,
    Target: 'static,
{
    fn select<'a>(&'a self, state: &'a State) -> Option<&'a Target> {
        self.ref_path.follow(state).map(|inner| inner.as_ref())
    }
}

impl<State, RefPath, Inner, Target, const SAFE: bool> Path<State, Target, SAFE> for AsRefPath<State, RefPath, Inner, Target, SAFE>
where
    State: 'static,
    RefPath: Path<State, Inner, SAFE>,
    Inner: AsRef<Target> + AsMut<Target> + 'static,
    Target: 'static,
{
    fn follow<'a>(&self, state: &'a State) -> Option<&'a Target> {
        self.ref_path.follow(state).map(|inner| inner.as_ref())
    }

    fn follow_mut<'a>(&self, state: &'a mut State) -> Option<&'a mut Target> {
        self.ref_path.follow_mut(state).map(|inner| inner.as_mut())
    }
}

pub trait AsRefExt<State, Inner, Target, const SAFE: bool>
where
    State: 'static,
    Self: Path<State, Inner, SAFE>,
    Inner: AsRef<Target> + AsMut<Target> + 'static,
    Target: 'static,
{
    fn path_as_ref(self) -> impl Path<State, Target, SAFE> {
        AsRefPath {
            ref_path: self,
            _marker: PhantomData,
        }
    }
}

impl<State, T, Inner, Target, const SAFE: bool> AsRefExt<State, Inner, Target, SAFE> for T
where
    State: 'static,
    T: Path<State, Inner, SAFE>,
    Inner: AsRef<Target> + AsMut<Target> + 'static,
    Target: 'static,
{
}
