//! Module providing a trait and an extension trait to index a array in the
//! state.
//!
//! Example:
//! ```
//! #![feature(auto_traits, negative_impls)]
//! use rust_state::{State, ManuallyAssertExt, RustState, ArrayLookupExt};
//!
//! #[derive(RustState)]
//! #[state_root]
//! struct MyState {
//!     items: [u32; 3],
//! }
//!
//! let state = State::new(MyState {
//!     items: [7, 8, 9],
//! });
//!
//! let item_path = MyState::path().items().array_index(1);
//!
//! assert_eq!(state.try_get(&item_path), Some(&8));
//! ```

use std::marker::PhantomData;

use crate::{Path, Selector};

/// A path for doing a dynamic lookup into an array.
///
/// This type is not accessible outside this module, instead
/// [`ArrayLookupExt`] can be used to construct it and receive a `impl
/// Path<State, Item>`.
struct ArrayLookup<State, ArrayPath, Item, const N: usize, const SAFE: bool> {
    array_path: ArrayPath,
    index: usize,
    _marker: PhantomData<(State, Item)>,
}

impl<State, ArrayPath, Item, const N: usize, const SAFE: bool> Clone for ArrayLookup<State, ArrayPath, Item, N, SAFE>
where
    ArrayPath: Path<State, [Item; N], SAFE>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<State, ArrayPath, Item, const N: usize, const SAFE: bool> Copy for ArrayLookup<State, ArrayPath, Item, N, SAFE> where
    ArrayPath: Path<State, [Item; N], SAFE>
{
}

impl<State, ArrayPath, Item, const N: usize, const SAFE: bool> Selector<State, Item, false> for ArrayLookup<State, ArrayPath, Item, N, SAFE>
where
    State: 'static,
    ArrayPath: Path<State, [Item; N], SAFE>,
    Item: 'static,
{
    fn select<'a>(&'a self, state: &'a State) -> Option<&'a Item> {
        self.follow(state)
    }
}

impl<State, ArrayPath, Item, const N: usize, const SAFE: bool> Path<State, Item, false> for ArrayLookup<State, ArrayPath, Item, N, SAFE>
where
    State: 'static,
    ArrayPath: Path<State, [Item; N], SAFE>,
    Item: 'static,
{
    fn follow<'a>(&self, state: &'a State) -> Option<&'a Item> {
        self.array_path.follow(state)?.get(self.index)
    }

    fn follow_mut<'a>(&self, state: &'a mut State) -> Option<&'a mut Item> {
        self.array_path.follow_mut(state)?.get_mut(self.index)
    }
}

pub trait ArrayLookupExt<State, T, Item, const N: usize, const SAFE: bool>
where
    State: 'static,
    Self: Path<State, [Item; N], SAFE>,
    Item: 'static,
{
    fn array_index(self, index: usize) -> impl Path<State, Item, false> {
        ArrayLookup {
            array_path: self,
            index,
            _marker: PhantomData,
        }
    }
}

impl<State, T, Item, const N: usize, const SAFE: bool> ArrayLookupExt<State, T, Item, N, SAFE> for T
where
    State: 'static,
    T: Path<State, [Item; N], SAFE>,
    Item: 'static,
{
}
