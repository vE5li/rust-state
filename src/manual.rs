//! Module providing an extension trait to manually assert that a path is safe.
//!
//! This can be useful for situations where you select an item from a vector or
//! map that you know exists.
//!
//! Example:
//! ```
//! use rust_state::{State, ManuallyAssertExt, RustState, VecItem, VecLookupExt};
//!
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
//! // We *know* that our item exists, so we can wrap the path with `manually_asserted`.
//! let item_path = MyState::path().items().lookup(10).manually_asserted();
//!
//! // We can use `get` since the path is safe.
//! assert_eq!(state.get(&item_path).id, 10);
//! ```

use std::marker::PhantomData;

use crate::{Path, Selector};

struct ManuallyAsserted<State, AssertedPath, To> {
    path: AssertedPath,
    _marker: PhantomData<(State, To)>,
}

impl<State, AssertedPath, To> Clone for ManuallyAsserted<State, AssertedPath, To>
where
    AssertedPath: Path<State, To, false>,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<State, AssertedPath, To> Copy for ManuallyAsserted<State, AssertedPath, To> where AssertedPath: Path<State, To, false> {}

impl<State, AssertedPath, To> Selector<State, To> for ManuallyAsserted<State, AssertedPath, To>
where
    State: 'static,
    AssertedPath: Path<State, To, false>,
    To: 'static,
{
    fn select<'a>(&'a self, state: &'a State) -> Option<&'a To> {
        self.path.select(state)
    }
}

impl<State, AssertedPath, To> Path<State, To> for ManuallyAsserted<State, AssertedPath, To>
where
    State: 'static,
    AssertedPath: Path<State, To, false>,
    To: 'static,
{
    fn follow<'a>(&self, state: &'a State) -> Option<&'a To> {
        self.path.follow(state)
    }

    fn follow_mut<'a>(&self, state: &'a mut State) -> Option<&'a mut To> {
        self.path.follow_mut(state)
    }
}

pub trait ManuallyAssertExt<State, To>
where
    State: 'static,
    Self: Path<State, To, false>,
    To: 'static,
{
    fn manually_asserted(self) -> impl Path<State, To> {
        ManuallyAsserted {
            path: self,
            _marker: PhantomData,
        }
    }
}

// Blanket implementation.
impl<State, To, T> ManuallyAssertExt<State, To> for T
where
    State: 'static,
    T: Path<State, To, false>,
    To: 'static,
{
}
