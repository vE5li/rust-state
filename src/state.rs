//! Module providing the [`State`], which is the base type for state
//! management.

use std::cell::UnsafeCell;
use std::collections::HashMap;

use crate::{MapItem, VecItem};

/// Marker trait for the root of the state.
///
/// This is only used when creating a new [`State`].
pub trait StateMarker {}

/// Error returned when a state update fails.
#[derive(Debug)]
pub struct UpdateError {
    /// The type name of the path that failed to update.
    pub type_name: &'static str,
}

impl UpdateError {
    /// Create an error for a given path type.
    fn with_path<Path>() -> Self {
        Self {
            type_name: std::any::type_name::<Path>(),
        }
    }
}

type StateChange<State> = Box<dyn FnOnce(&mut State) -> Result<(), UpdateError>>;

/// A wrapper around the root state. Can be read and mutated using
/// [`Path`](crate::Path)s.
///
/// The `State` wrapper allows you to work on data in the state and even keep
/// immutable references to it while simultaneously queuing state
/// changes on that same data.
pub struct State<Inner> {
    state: Inner,
    state_changes: UnsafeCell<Vec<StateChange<Inner>>>,
}

impl<Inner: StateMarker> State<Inner> {
    /// Create a new state for a root state.
    ///
    /// A state can only be created for the root state.
    ///
    /// You can mark the root of your state using the derive macro:
    ///
    /// ```
    /// #![feature(auto_traits, negative_impls)]
    /// use rust_state::{State, RustState};
    ///
    /// #[derive(RustState)]
    /// #[state_root]
    /// struct MyState;
    ///
    /// let _ = State::new(MyState);
    /// ```
    ///
    /// Trying to create a state for anything other than the root of the state
    /// will fail to compile.
    ///
    /// ```compile_fail
    /// #![feature(auto_traits, negative_impls)]
    /// use rust_state::{State, RustState};
    ///
    /// #[derive(RustState)]
    /// struct MyState;
    ///
    /// let _ = State::new(MyState);
    /// ```
    pub fn new(state: Inner) -> Self {
        Self {
            state,
            state_changes: UnsafeCell::new(Vec::new()),
        }
    }
}

impl<Inner> State<Inner> {
    fn push_change(&self, state_change: StateChange<Inner>) {
        let state_changes = unsafe { &mut *self.state_changes.get() };
        state_changes.push(state_change);
    }

    /// Update the value for a given path.
    ///
    /// Example:
    /// ```
    /// #![feature(auto_traits, negative_impls)]
    /// use rust_state::{State, RustState};
    ///
    /// #[derive(RustState)]
    /// #[state_root]
    /// struct MyState {
    ///     value: u32,
    /// }
    ///
    /// let mut state = State::new(MyState { value: 5 });
    /// let value_path = MyState::path().value();
    ///
    /// state.update_value(value_path, 10);
    /// state.apply().unwrap();
    ///
    /// assert_eq!(state.get(&value_path), &10);
    /// ```
    pub fn update_value<Path, Value, const SAFE: bool>(&self, path: Path, value: Value)
    where
        Path: crate::Path<Inner, Value, SAFE>,
        Value: 'static,
    {
        self.push_change(Box::new(move |state: &mut Inner| match path.follow_mut(state) {
            Some(reference) => Ok(*reference = value),
            None => Err(UpdateError::with_path::<Path>()),
        }));
    }

    /// Update the value for a given path with a closure.
    ///
    /// Example:
    /// ```
    /// #![feature(auto_traits, negative_impls)]
    /// use rust_state::{State, RustState};
    ///
    /// #[derive(RustState)]
    /// #[state_root]
    /// struct MyState {
    ///     value: u32,
    /// }
    ///
    /// let mut state = State::new(MyState { value: 5 });
    /// let value_path = MyState::path().value();
    ///
    /// state.update_value_with(value_path, |value| *value *= 2);
    /// state.apply().unwrap();
    ///
    /// assert_eq!(state.get(&value_path), &10);
    /// ```
    pub fn update_value_with<Path, Value, F, const SAFE: bool>(&self, path: Path, closure: F)
    where
        Path: crate::Path<Inner, Value, SAFE>,
        F: Fn(&mut Value) + 'static,
    {
        self.push_change(Box::new(move |state: &mut Inner| match path.follow_mut(state) {
            Some(reference) => Ok(closure(reference)),
            None => Err(UpdateError::with_path::<Path>()),
        }));
    }

    /// Push an item to a [`Vec`].
    ///
    /// Example:
    /// ```
    /// #![feature(auto_traits, negative_impls)]
    /// use rust_state::{State, RustState, VecItem};
    ///
    /// struct TestItem {
    ///     id: u32,
    /// }
    ///
    /// impl VecItem for TestItem {
    ///     type Id = u32;
    ///
    ///     fn get_id(&self) -> Self::Id {
    ///         self.id
    ///     }
    /// }
    ///
    /// #[derive(RustState)]
    /// #[state_root]
    /// struct MyState {
    ///     items: Vec<TestItem>,
    /// }
    ///
    /// let mut state = State::new(MyState { items: Vec::new() });
    /// let items_path = MyState::path().items();
    ///
    /// state.vec_push(items_path, TestItem { id: 10 });
    /// state.apply().unwrap();
    ///
    /// assert_eq!(state.get(&items_path).len(), 1);
    /// ```
    pub fn vec_push<Path, Value, const SAFE: bool>(&self, path: Path, value: Value)
    where
        Path: crate::Path<Inner, Vec<Value>, SAFE>,
        Value: 'static,
    {
        self.push_change(Box::new(move |state: &mut Inner| match path.follow_mut(state) {
            Some(reference) => Ok(reference.push(value)),
            None => Err(UpdateError::with_path::<Path>()),
        }));
    }

    /// Remove an item from a [`Vec`].
    ///
    /// Example:
    /// ```
    /// #![feature(auto_traits, negative_impls)]
    /// use rust_state::{State, RustState, VecItem};
    ///
    /// struct TestItem {
    ///     id: u32,
    /// }
    ///
    /// impl VecItem for TestItem {
    ///     type Id = u32;
    ///
    ///     fn get_id(&self) -> Self::Id {
    ///         self.id
    ///     }
    /// }
    ///
    /// #[derive(RustState)]
    /// #[state_root]
    /// struct MyState {
    ///     items: Vec<TestItem>,
    /// }
    ///
    /// let mut state = State::new(MyState { items: vec![TestItem { id: 10 }] });
    /// let items_path = MyState::path().items();
    ///
    /// state.vec_remove(items_path, 10);
    /// state.apply().unwrap();
    ///
    /// assert_eq!(state.get(&items_path).len(), 0);
    /// ```
    pub fn vec_remove<Path, Value, const SAFE: bool>(&self, path: Path, id: Value::Id)
    where
        Path: crate::Path<Inner, Vec<Value>, SAFE>,
        Value: VecItem + 'static,
    {
        self.push_change(Box::new(move |state: &mut Inner| match path.follow_mut(state) {
            Some(reference) => Ok(reference.retain(|item| item.get_id() != id)),
            None => Err(UpdateError::with_path::<Path>()),
        }));
    }

    /// Insert an item into a [`HashMap`].
    ///
    /// Example:
    /// ```
    /// #![feature(auto_traits, negative_impls)]
    /// use std::collections::HashMap;
    /// use rust_state::{State, RustState, MapItem};
    ///
    /// struct TestItem;
    //
    /// impl MapItem for TestItem {
    ///     type Id = u32;
    /// }
    ///
    /// #[derive(RustState)]
    /// #[state_root]
    /// struct MyState {
    ///     items: HashMap<u32, TestItem>,
    /// }
    ///
    /// let mut state = State::new(MyState { items: HashMap::new() });
    /// let items_path = MyState::path().items();
    ///
    /// state.map_insert(items_path, 10, TestItem);
    /// state.apply().unwrap();
    ///
    /// assert_eq!(state.get(&items_path).len(), 1);
    /// ```
    pub fn map_insert<Path, Value, const SAFE: bool>(&self, path: Path, id: Value::Id, value: Value)
    where
        Path: crate::Path<Inner, HashMap<Value::Id, Value>, SAFE>,
        Value: MapItem + 'static,
    {
        self.push_change(Box::new(move |state: &mut Inner| match path.follow_mut(state) {
            Some(reference) => {
                reference.insert(id, value);
                Ok(())
            }
            None => Err(UpdateError::with_path::<Path>()),
        }));
    }

    /// Insert an item with default value into a [`HashMap`].
    ///
    /// Example:
    /// ```
    /// #![feature(auto_traits, negative_impls)]
    /// use std::collections::HashMap;
    /// use rust_state::{State, RustState, MapItem};
    ///
    /// #[derive(Default)]
    /// struct TestItem;
    //
    /// impl MapItem for TestItem {
    ///     type Id = u32;
    /// }
    ///
    /// #[derive(RustState)]
    /// #[state_root]
    /// struct MyState {
    ///     items: HashMap<u32, TestItem>,
    /// }
    ///
    /// let mut state = State::new(MyState { items: HashMap::new() });
    /// let items_path = MyState::path().items();
    ///
    /// state.map_insert_default(items_path, 10);
    /// state.apply().unwrap();
    ///
    /// assert_eq!(state.get(&items_path).len(), 1);
    /// ```
    pub fn map_insert_default<Path, Value, const SAFE: bool>(&self, path: Path, id: Value::Id)
    where
        Path: crate::Path<Inner, HashMap<Value::Id, Value>, SAFE>,
        Value: MapItem + Default + 'static,
    {
        self.push_change(Box::new(move |state: &mut Inner| match path.follow_mut(state) {
            Some(reference) => {
                reference.entry(id).or_default();
                Ok(())
            }
            None => Err(UpdateError::with_path::<Path>()),
        }));
    }

    /// Remove an item from a [`HashMap`].
    ///
    /// Example:
    /// ```
    /// #![feature(auto_traits, negative_impls)]
    /// use std::collections::HashMap;
    /// use rust_state::{State, RustState, MapItem};
    ///
    /// struct TestItem;
    //
    /// impl MapItem for TestItem {
    ///     type Id = u32;
    /// }
    ///
    /// #[derive(RustState)]
    /// #[state_root]
    /// struct MyState {
    ///     items: HashMap<u32, TestItem>,
    /// }
    ///
    /// let mut state = State::new(MyState {
    ///     items: HashMap::from([(10, TestItem)]),
    /// });
    /// let items_path = MyState::path().items();
    ///
    /// state.map_remove(items_path, 10);
    /// state.apply().unwrap();
    ///
    /// assert_eq!(state.get(&items_path).len(), 0);
    /// ```
    pub fn map_remove<Path, Value, const SAFE: bool>(&self, path: Path, id: Value::Id)
    where
        Path: crate::Path<Inner, HashMap<Value::Id, Value>, SAFE>,
        Value: MapItem + 'static,
    {
        self.push_change(Box::new(move |state: &mut Inner| match path.follow_mut(state) {
            Some(reference) => {
                reference.remove(&id);
                Ok(())
            }
            None => Err(UpdateError::with_path::<Path>()),
        }));
    }

    /// Apply any pending changes.
    ///
    /// Example:
    /// ```
    /// #![feature(auto_traits, negative_impls)]
    /// use rust_state::{State, RustState};
    ///
    /// #[derive(RustState)]
    /// #[state_root]
    /// struct MyState {
    ///     value: &'static str,
    /// }
    ///
    /// let mut state = State::new(MyState { value: "Before" });
    /// let value_path = MyState::path().value();
    ///
    /// state.update_value(value_path, "After");
    ///
    /// assert_eq!(state.get(&value_path), &"Before");
    ///
    /// state.apply().unwrap();
    ///
    /// assert_eq!(state.get(&value_path), &"After");
    /// ```
    pub fn apply(&mut self) -> Result<(), Vec<UpdateError>> {
        let mut errors = Vec::new();

        for apply in UnsafeCell::get_mut(&mut self.state_changes).drain(..) {
            if let Err(error) = apply(&mut self.state) {
                errors.push(error);
            }
        }

        match errors.is_empty() {
            true => Ok(()),
            false => Err(errors),
        }
    }

    /// Get the output of a safe selector.
    ///
    /// # Panics
    ///
    /// This function will panic if the safe selector was unable to read the
    /// value. This might happen if
    /// [`ManuallyAssertExt`](crate::ManuallyAssertExt) is used incorrectly
    /// or the [`Selector`](crate::Selector) trait is implemented incorrectly.
    pub fn get<'a, Selector, Output>(&'a self, selector: &'a Selector) -> &'a Output
    where
        Selector: crate::Selector<Inner, Output>,
        Output: ?Sized,
    {
        selector.select(&self.state).unwrap()
    }

    /// Try to get the output of an unsafe selector.
    ///
    /// This is deliberately only implement for unsafe selectors and not for
    /// all selectors to encourage using the safe [`get`](Self::get)
    /// if possible.
    ///
    /// For a version of this function that accepts any selector see
    /// [`try_get_any`](Self::try_get_any).
    pub fn try_get<'a, Selector, Output>(&'a self, selector: &'a Selector) -> Option<&'a Output>
    where
        Selector: crate::Selector<Inner, Output, false>,
        Output: ?Sized,
    {
        selector.select(&self.state)
    }

    /// Try to get the output of any (safe or unsafe) selector.
    ///
    /// Use of this function is discouraged unless `SAFE` is not known in
    /// the current scope.
    ///
    /// Please use [`get`](Self::get) and [`try_get`](Self::try_get)
    /// otherwise.
    pub fn try_get_any<Selector, Output, const SAFE: bool>(&self, selector: Selector) -> Option<&'_ Output>
    where
        Selector: crate::Path<Inner, Output, SAFE>,
        Output: ?Sized,
    {
        selector.follow(&self.state)
    }

    /// Follow a safe path.
    ///
    /// # Panics
    ///
    /// This function will panic if the safe path was unable to read the
    /// value. This might happen if
    /// [`ManuallyAssertExt`](crate::ManuallyAssertExt) is used incorrectly
    /// or the [`Path`](crate::Path) trait is implemented incorrectly.
    pub fn follow<Path, Output>(&self, path: Path) -> &Output
    where
        Path: crate::Path<Inner, Output>,
        Output: ?Sized,
    {
        path.follow(&self.state).unwrap()
    }

    /// Try to follow an unsafe path.
    ///
    /// This is deliberately only implement for unsafe paths and not for
    /// all paths to encourage using the safe [`follow`](Self::follow)
    /// if possible.
    ///
    /// For a version of this function that accepts any path see
    /// [`try_follow_any`](Self::try_follow_any).
    pub fn try_follow<Path, Output>(&self, path: Path) -> Option<&Output>
    where
        Path: crate::Path<Inner, Output, false>,
        Output: ?Sized,
    {
        path.follow(&self.state)
    }

    /// Try to follow any (safe or unsafe) path.
    ///
    /// Use of this function is discouraged unless `SAFE` is not known in
    /// the current scope.
    ///
    /// Please use [`follow`](Self::follow) and [`try_follow`](Self::try_follow)
    /// otherwise.
    pub fn try_follow_any<Path, Output, const SAFE: bool>(&self, path: Path) -> Option<&Output>
    where
        Path: crate::Path<Inner, Output, SAFE>,
        Output: ?Sized,
    {
        path.follow(&self.state)
    }

    /// Follow a safe path.
    ///
    /// # Panics
    ///
    /// This function will panic if the safe path was unable to read the
    /// value. This might happen if
    /// [`ManuallyAssertExt`](crate::ManuallyAssertExt) is used incorrectly
    /// or the [`Path`](crate::Path) trait is implemented incorrectly.
    pub fn follow_mut<Path, Output>(&mut self, path: Path) -> &mut Output
    where
        Path: crate::Path<Inner, Output>,
        Output: ?Sized,
    {
        path.follow_mut(&mut self.state).unwrap()
    }

    /// Try to follow an unsafe path.
    ///
    /// This is deliberately only implement for unsafe paths and not for
    /// all paths to encourage using the safe [`follow_mut`](Self::follow_mut)
    /// if possible.
    ///
    /// For a version of this function that accepts any path see
    /// [`try_follow_any_mut`](Self::try_follow_any_mut).
    pub fn try_follow_mut<Path, Output>(&mut self, path: Path) -> Option<&mut Output>
    where
        Path: crate::Path<Inner, Output, false>,
        Output: ?Sized,
    {
        path.follow_mut(&mut self.state)
    }

    /// Try to follow any (safe or unsafe) path.
    ///
    /// Use of this function is discouraged unless `SAFE` is not known in
    /// the current scope.
    ///
    /// Please use [`follow`](Self::follow_mut) and
    /// [`try_follow`](Self::try_follow_mut) otherwise.
    pub fn try_follow_any_mut<Path, Output, const SAFE: bool>(&mut self, path: Path) -> Option<&mut Output>
    where
        Path: crate::Path<Inner, Output, SAFE>,
        Output: ?Sized,
    {
        path.follow_mut(&mut self.state)
    }
}
