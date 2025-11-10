//! Module providing the base mechanism for indexing state, namely [`Path`] and
//! [`Selector`].

/// A `Selector` can be used to get an item from the state or itself.
///
/// `Selector<State, T>` is implemented for `T`, so any value is also a
/// `Selector` to itself.
///
/// An example of how this can be used:
///
/// ```rust
/// use rust_state::{State, RustState, Selector};
///
/// #[derive(Default, RustState)]
/// #[state_root]
/// struct GlobalState {
///     number: u32,
/// }
///
/// struct Uses<S>(S);
///
/// impl<S: Selector<GlobalState, u32>> Uses<S> {
///     pub fn new(selector: S) -> Self {
///         Self(selector)
///     }
///
///     pub fn do_work(&self, state: &State<GlobalState>) {
///         let _: &u32 = state.get(&self.0);
///     }
/// }
///
/// let state = State::new(GlobalState::default());
///
/// let uses_0 = Uses::new(GlobalState::path().number());
/// let uses_1 = Uses::new(1u32);
///
/// uses_0.do_work(&state);
/// uses_1.do_work(&state);
/// ```
///
/// As can be seen in the example above, the main purpose of [`Selector`] is to
/// abstract over `T` and [`Path`]s that follow to a `T`.
pub trait Selector<State, To: ?Sized, const SAFE: bool = true>: 'static {
    fn select<'a>(&'a self, state: &'a State) -> Option<&'a To>;
}

/// Selector extension trait that allows selecting a safe selector to a `T`
/// instead of an `Option<T>`.
///
/// See [`Selector`] for more documentation on paths.
pub trait SelectorExt<State, To: ?Sized> {
    /// Select the path and return a reference to its target.
    fn select_safe<'a>(&'a self, state: &'a State) -> &'a To;
}

// Blanket impl
impl<T, State, To> SelectorExt<State, To> for T
where
    T: Selector<State, To>,
{
    fn select_safe<'a>(&'a self, state: &'a State) -> &'a To {
        self.select(state).unwrap()
    }
}

/// Workaround for conflicting implementations when implementing [`Path`] for
/// generic types.
///
/// Currently this is only used when deriving [`RustState`](crate::RustState)
/// but can hopefully be completely removed in the future.
pub auto trait AutoImplSelector {}

// Blanket implementation so that any `T` is a `Selector` for `T`.
impl<State, T: 'static> Selector<State, T> for T
where
    T: AutoImplSelector,
{
    fn select<'a>(&'a self, _: &'a State) -> Option<&'a T> {
        Some(self)
    }
}

/// A `Path` can be followed to get a mutable or immutable reference to
/// arbitrary data from the state.
///
/// Paths are forced to be [`Copy`] so they are easier to pass around and
/// duplicate.
///
/// Additionally, every path is forced to implement [`Selector`] to improve the
/// ergonomics of the [`State`].
///
/// The [`Path`] trait is automatically implemented when deriving
/// [`RustState`](crate::RustState).
///
/// Example:
///```
/// use rust_state::{State, RustState, Path};
///
/// #[derive(Default, RustState)]
/// #[state_root]
/// struct GlobalState {
///     number: u32,
/// }
///
/// fn takes_path<T>(_: impl Path<GlobalState, T>) {}
///
/// takes_path::<GlobalState>(GlobalState::path());
/// takes_path::<u32>(GlobalState::path().number());
/// ```
///
/// Paths can also be generated for generic types:
///
/// ```
/// use rust_state::{State, RustState, Path};
///
/// #[derive(Default, RustState)]
/// #[state_root]
/// struct GlobalState {
///     generic: GenericStruct<u32>,
/// }
///
/// #[derive(Default, RustState)]
/// struct GenericStruct<T>
/// where
///     T: Default + 'static
/// {
///     inner: T,
/// }
///
/// let path = GlobalState::path().generic();
/// ```
pub trait Path<State, To: ?Sized, const SAFE: bool = true>: Selector<State, To, SAFE> + Copy {
    /// Follow the path and try to return a reference to its target.
    fn follow<'a>(&self, state: &'a State) -> Option<&'a To>;

    /// Follow the path and try to return a mutable reference to its target.
    fn follow_mut<'a>(&self, state: &'a mut State) -> Option<&'a mut To>;
}

/// Path extension trait that allows following a safe path to a `T` instead of
/// an `Option<T>`.
///
/// See [`Path`] for more documentation on paths.
pub trait PathExt<State, To: ?Sized> {
    /// Follow the path and return a reference to its target.
    fn follow_safe<'a>(&self, state: &'a State) -> &'a To;

    /// Follow the path and return a mutable reference to its target.
    fn follow_mut_safe<'a>(&self, state: &'a mut State) -> &'a mut To;
}

// Blanket impl
impl<T, State, To> PathExt<State, To> for T
where
    T: Path<State, To>,
{
    fn follow_safe<'a>(&self, state: &'a State) -> &'a To {
        self.follow(state).unwrap()
    }

    fn follow_mut_safe<'a>(&self, state: &'a mut State) -> &'a mut To {
        self.follow_mut(state).unwrap()
    }
}
