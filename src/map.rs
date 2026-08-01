//! Module providing a trait and an extension trait to index a map in the state.
//!
//! Example:
//! ```
//! #![feature(auto_traits, negative_impls)]
//! use std::collections::HashMap;
//! use rust_state::{State, ManuallyAssertExt, RustState, MapItem, MapLookupExt};
//!
//! #[derive(Debug, PartialEq, Eq)]
//! struct TestItem;
//!
//! impl MapItem for TestItem {
//!     type Id = u32;
//! }
//!
//! #[derive(RustState)]
//! #[state_root]
//! struct MyState {
//!     items: HashMap<u32, TestItem>,
//! }
//!
//! let state = State::new(MyState {
//!     items: HashMap::from([(10, TestItem)]),
//! });
//!
//! let item_path = MyState::path().items().lookup(10);
//!
//! assert_eq!(state.try_get(&item_path), Some(&TestItem));
//! ```

use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;

use crate::{Path, Selector};

pub trait MapItem {
    type Id: Eq + PartialEq + Hash + Copy;
}

struct MapLookup<State, Path, Item, const SAFE: bool>
where
    Item: MapItem,
{
    path: Path,
    id: Item::Id,
    _marker: PhantomData<State>,
}

impl<State, Path, Item, const SAFE: bool> Clone for MapLookup<State, Path, Item, SAFE>
where
    Path: crate::Path<State, HashMap<Item::Id, Item>, SAFE>,
    Item: MapItem,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<State, Path, Item, const SAFE: bool> Copy for MapLookup<State, Path, Item, SAFE>
where
    Path: crate::Path<State, HashMap<Item::Id, Item>, SAFE>,
    Item: MapItem,
{
}

impl<State, Path, Item, const SAFE: bool> Selector<State, Item, false> for MapLookup<State, Path, Item, SAFE>
where
    State: 'static,
    Path: crate::Path<State, HashMap<Item::Id, Item>, SAFE>,
    Item: MapItem + 'static,
{
    fn select<'a>(&'a self, state: &'a State) -> Option<&'a Item> {
        self.path.follow(state)?.get(&self.id)
    }
}

impl<State, Path, Item, const SAFE: bool> crate::Path<State, Item, false> for MapLookup<State, Path, Item, SAFE>
where
    State: 'static,
    Path: crate::Path<State, HashMap<Item::Id, Item>, SAFE>,
    Item: MapItem + 'static,
{
    fn follow<'a>(&self, state: &'a State) -> Option<&'a Item> {
        self.path.follow(state)?.get(&self.id)
    }

    fn follow_mut<'a>(&self, state: &'a mut State) -> Option<&'a mut Item> {
        self.path.follow_mut(state)?.get_mut(&self.id)
    }
}

pub trait MapLookupExt<State, T, Item, const SAFE: bool>
where
    State: 'static,
    Self: Path<State, HashMap<Item::Id, Item>, SAFE>,
    Item: MapItem + 'static,
{
    fn lookup(self, id: Item::Id) -> impl Path<State, Item, false> {
        MapLookup {
            path: self,
            id,
            _marker: PhantomData,
        }
    }
}

impl<State, T, Item, const SAFE: bool> MapLookupExt<State, T, Item, SAFE> for T
where
    State: 'static,
    T: Path<State, HashMap<Item::Id, Item>, SAFE>,
    Item: MapItem + 'static,
{
}
