//! Module providing a trait and an extension trait to index a vector in the
//! state.
//!
//! There are two main ways to get the items of a [`Vec`]:
//!
//! `Indexing the vector`: This work for every vector of T but the item the path
//! resolves to might change if the state changes.
//!
//!
//! `A lookup`: This is a bit slower and only for `Vec<T> where T: VecItem` but
//! resolves to the same item every time.
//!
//! Example:
//! ```
//! #![feature(auto_traits, negative_impls)]
//! use rust_state::{State, ManuallyAssertExt, RustState, VecIndexExt, VecItem, VecLookupExt};
//!
//! #[derive(Debug, PartialEq, Eq)]
//! struct TestItem {
//!     id: u32,
//! }
//!
//! impl VecItem for TestItem {
//!     type Id = u32;
//!
//!     fn get_id(&self) -> Self::Id {
//!         self.id
//!     }
//! }
//!
//! #[derive(RustState)]
//! #[state_root]
//! struct MyState {
//!     items: Vec<TestItem>,
//! }
//!
//! let state = State::new(MyState {
//!     items: vec![TestItem { id: 10 }],
//! });
//!
//! let lookup_path = MyState::path().items().lookup(10);
//!
//! assert_eq!(state.try_get(&lookup_path), Some(&TestItem { id: 10 }));
//!
//! let index_path = MyState::path().items().index(0);
//!
//! assert_eq!(state.try_get(&index_path), Some(&TestItem { id: 10 }));
//! ```

use std::hash::Hash;
use std::marker::PhantomData;

use crate::{Path, Selector};

/// An item inside a vector accessible through a [`Path`].
pub trait VecItem {
    /// The unique Id of the item. To make sure that [`Path`]s stay valid
    /// between updates of the state, this Id should be unique for each
    /// entry and not be re-used.
    type Id: Copy + PartialEq + Eq + Hash;

    /// Get the unique id of this entry.
    fn get_id(&self) -> Self::Id;
}

/// A path for doing a dynamic lookup into a [`Vec`] of [`VecItem`]s.
///
/// This type is not accessible outside this module, instead
/// [`VecLookupExt`] can be used to construct it and receive a `impl
/// Path<State, Item>`.
struct VecLookup<State, VectorPath, Item, const SAFE: bool>
where
    Item: VecItem,
{
    vector_path: VectorPath,
    id: Item::Id,
    _marker: PhantomData<State>,
}

impl<State, VectorPath, Item, const SAFE: bool> Clone for VecLookup<State, VectorPath, Item, SAFE>
where
    VectorPath: Path<State, Vec<Item>, SAFE>,
    Item: VecItem,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<State, VectorPath, Item, const SAFE: bool> Copy for VecLookup<State, VectorPath, Item, SAFE>
where
    VectorPath: Path<State, Vec<Item>, SAFE>,
    Item: VecItem,
{
}

impl<State, VectorPath, Item, const SAFE: bool> Selector<State, Item, false> for VecLookup<State, VectorPath, Item, SAFE>
where
    State: 'static,
    VectorPath: Path<State, Vec<Item>, SAFE>,
    Item: VecItem + 'static,
{
    fn select<'a>(&'a self, state: &'a State) -> Option<&'a Item> {
        self.vector_path.follow(state)?.iter().find(|e| e.get_id() == self.id)
    }
}

impl<State, VectorPath, Item, const SAFE: bool> Path<State, Item, false> for VecLookup<State, VectorPath, Item, SAFE>
where
    State: 'static,
    VectorPath: Path<State, Vec<Item>, SAFE>,
    Item: VecItem + 'static,
{
    fn follow<'a>(&self, state: &'a State) -> Option<&'a Item> {
        self.vector_path.follow(state)?.iter().find(|e| e.get_id() == self.id)
    }

    fn follow_mut<'a>(&self, state: &'a mut State) -> Option<&'a mut Item> {
        self.vector_path.follow_mut(state)?.iter_mut().find(|e| e.get_id() == self.id)
    }
}

pub trait VecLookupExt<State, T, Item, const SAFE: bool>
where
    State: 'static,
    Self: Path<State, Vec<Item>, SAFE>,
    Item: VecItem + 'static,
{
    fn lookup(self, id: Item::Id) -> impl Path<State, Item, false> {
        VecLookup {
            vector_path: self,
            id,
            _marker: PhantomData,
        }
    }
}

impl<State, T, Item, const SAFE: bool> VecLookupExt<State, T, Item, SAFE> for T
where
    State: 'static,
    T: Path<State, Vec<Item>, SAFE>,
    Item: VecItem + 'static,
{
}

/// A path for doing a dynamic index into a [`Vec`].
///
/// This type is not accessible outside this module, instead
/// [`VecIndexExt`] can be used to construct it and receive a `impl
/// Path<State, Item>`.
struct VecIndex<State, VectorPath, Item, const SAFE: bool> {
    vector_path: VectorPath,
    index: usize,
    _marker: PhantomData<(State, Item)>,
}

impl<State, VectorPath, Item, const SAFE: bool> Clone for VecIndex<State, VectorPath, Item, SAFE>
where
    VectorPath: Path<State, Vec<Item>, SAFE>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<State, VectorPath, Item, const SAFE: bool> Copy for VecIndex<State, VectorPath, Item, SAFE> where
    VectorPath: Path<State, Vec<Item>, SAFE>
{
}

impl<State, VectorPath, Item, const SAFE: bool> Selector<State, Item, false> for VecIndex<State, VectorPath, Item, SAFE>
where
    State: 'static,
    VectorPath: Path<State, Vec<Item>, SAFE>,
    Item: 'static,
{
    fn select<'a>(&'a self, state: &'a State) -> Option<&'a Item> {
        self.follow(state)
    }
}

impl<State, VectorPath, Item, const SAFE: bool> Path<State, Item, false> for VecIndex<State, VectorPath, Item, SAFE>
where
    State: 'static,
    VectorPath: Path<State, Vec<Item>, SAFE>,
    Item: 'static,
{
    fn follow<'a>(&self, state: &'a State) -> Option<&'a Item> {
        self.vector_path.follow(state)?.get(self.index)
    }

    fn follow_mut<'a>(&self, state: &'a mut State) -> Option<&'a mut Item> {
        self.vector_path.follow_mut(state)?.get_mut(self.index)
    }
}

pub trait VecIndexExt<State, T, Item, const SAFE: bool>
where
    State: 'static,
    Self: Path<State, Vec<Item>, SAFE>,
    Item: 'static,
{
    fn index(self, index: usize) -> impl Path<State, Item, false> {
        VecIndex {
            vector_path: self,
            index,
            _marker: PhantomData,
        }
    }
}

impl<State, T, Item, const SAFE: bool> VecIndexExt<State, T, Item, SAFE> for T
where
    State: 'static,
    T: Path<State, Vec<Item>, SAFE>,
    Item: 'static,
{
}
