# rust-state

Typed, composable **paths** into a centralized state tree. Read *and* write
deeply nested data without passing references around, and let the type system
track which accesses can fail.

`rust-state` was built for — and is battle-tested in —
[Korangar](https://github.com/vE5li/korangar), a Ragnarok Online client written
in Rust, where it powers the entire user interface. The path abstraction itself
is general, but the design decisions are driven by that use case, so the
examples below lean on it.

> [!IMPORTANT]
> This crate is nightly-only and its API is still evolving. See
> [Status & caveats](#status--caveats) before depending on it.

## The idea

You mark the root of your state, derive `RustState` on the structs inside it,
and get back cheap, `Copy`, zero-sized *path* values that point at individual
fields:

```rust
#![feature(auto_traits, negative_impls)]
use rust_state::{State, RustState};

#[derive(RustState)]
#[state_root]
struct MyState {
    value: u32,
}

let mut state = State::new(MyState { value: 5 });
let value_path = MyState::path().value();

// Read through the path...
assert_eq!(state.get(&value_path), &5);

// ...and queue a write through the same path.
state.update_value(value_path, 10);
state.apply().unwrap();

assert_eq!(state.get(&value_path), &10);
```

A path is not a reference — it is a *description* of where something lives. That
means you can store it, pass it around, and hand it to code that has no idea
what the rest of your state looks like.

## Why that matters: state-agnostic components

Because a path is just a value, you can write a component that is generic over
*where* its data lives. Here is a real settings window from Korangar, trimmed
only slightly:

```rust,ignore
pub struct GameSettingsWindow<A> {
    game_settings_path: A,
}

impl<A> CustomWindow<ClientState> for GameSettingsWindow<A>
where
    A: Path<ClientState, GameSettings>,
{
    fn to_window<'a>(self) -> impl Window<ClientState> + 'a {
        window! {
            title: client_state().localization().game_settings_window_title(),
            elements: (
                state_button! {
                    text: client_state().localization().auto_attack_button_text(),
                    state: self.game_settings_path.auto_attack(),
                    event: Toggle(self.game_settings_path.auto_attack()),
                },
            ),
        }
    }
}
```

The window holds *no* reference to the state, no `Rc<RefCell<_>>`, no index —
just a path. The same path is used to **read** the current state and
to **produce a write** (`event: Toggle(..)`). Construct the window with a
different path and it works against any `GameSettings` anywhere in the tree.

This works because every input to a component is a
[`Selector`](https://docs.rs/rust-state) — and *any value is a selector to
itself*. So a field can be given a literal or a path, interchangeably:

```rust
#![feature(auto_traits, negative_impls)]
use rust_state::{State, RustState, Selector};

#[derive(Default, RustState)]
#[state_root]
struct GlobalState {
    number: u32,
}

struct Uses<S>(S);

impl<S: Selector<GlobalState, u32>> Uses<S> {
    pub fn do_work(&self, state: &State<GlobalState>) {
        let _: &u32 = state.get(&self.0);
    }
}

let state = State::new(GlobalState::default());

// A path to a field in the state...
Uses(GlobalState::path().number()).do_work(&state);
// ...or just a plain value. Both are `Selector<GlobalState, u32>`.
Uses(1u32).do_work(&state);
```

## Fallibility is tracked in the type

Following a struct field can never fail, so those paths stay *safe* and
`get`/`follow` hand you a `&T` directly. The moment you do something that
*might* not resolve — indexing a `Vec`, unwrapping an `Option`, downcasting —
the path type flips to *unsafe* and the API forces you to acknowledge it with
`try_get` returning an `Option`:

```rust
#![feature(auto_traits, negative_impls)]
use rust_state::{State, OptionExt, RustState};

#[derive(Debug, PartialEq, Eq)]
struct TestItem {
    value: usize,
}

#[derive(RustState)]
#[state_root]
struct MyState {
    option: Option<TestItem>,
}

let state = State::new(MyState {
    option: Some(TestItem { value: 20 }),
});

// `.unwrapped()` turns a `Path<_, Option<T>>` into a fallible `Path<_, T>`.
let path = MyState::path().option().unwrapped();

assert_eq!(state.try_get(&path), Some(&TestItem { value: 20 }));
```

If you *know* a fallible lookup will succeed, you can opt back into the safe API
with `manually_asserted()` — useful for items you just inserted and never
remove:

```rust
#![feature(auto_traits, negative_impls)]
use rust_state::{State, ManuallyAssertExt, RustState, VecItem, VecLookupExt};

struct TestItem {
    id: u32,
}

impl VecItem for TestItem {
    type Id = u32;

    fn get_id(&self) -> Self::Id {
        self.id
    }
}

#[derive(RustState)]
#[state_root]
struct MyState {
    items: Vec<TestItem>,
}

let state = State::new(MyState {
    items: vec![TestItem { id: 10 }],
});

// We *know* the item exists, so we assert the path is safe...
let item_path = MyState::path().items().lookup(10).manually_asserted();

// ...and can use the infallible `get`.
assert_eq!(state.get(&item_path).id, 10);
```

## Indexing collections

Paths can descend into `Vec`s, `HashMap`s and arrays. For `Vec`s you can either
**index** by position or **look up** a stable id (`VecItem`), so a path keeps
pointing at the same logical item even as the vector changes:

```rust
#![feature(auto_traits, negative_impls)]
use rust_state::{State, RustState, VecIndexExt, VecItem, VecLookupExt};

#[derive(Debug, PartialEq, Eq)]
struct TestItem {
    id: u32,
}

impl VecItem for TestItem {
    type Id = u32;

    fn get_id(&self) -> Self::Id {
        self.id
    }
}

#[derive(RustState)]
#[state_root]
struct MyState {
    items: Vec<TestItem>,
}

let state = State::new(MyState {
    items: vec![TestItem { id: 10 }],
});

// Stable lookup by id.
let lookup_path = MyState::path().items().lookup(10);
assert_eq!(state.try_get(&lookup_path), Some(&TestItem { id: 10 }));

// Positional index.
let index_path = MyState::path().items().index(0);
assert_eq!(state.try_get(&index_path), Some(&TestItem { id: 10 }));
```

## Queuing writes

Every mutation is *queued* against a path and flushed with `apply()`. This lets
you read the state and schedule edits to it in the same scope, without fighting
the borrow checker — a component can inspect the world and enqueue changes to it
at the same time:

```rust
#![feature(auto_traits, negative_impls)]
use rust_state::{State, RustState};

#[derive(RustState)]
#[state_root]
struct MyState {
    value: &'static str,
}

let mut state = State::new(MyState { value: "Before" });
let value_path = MyState::path().value();

state.update_value(value_path, "After");

// The change is only visible after `apply`.
assert_eq!(state.get(&value_path), &"Before");
state.apply().unwrap();
assert_eq!(state.get(&value_path), &"After");
```

There are queued operations for the common container mutations too:
`update_value_with` (in-place closure), `vec_push` / `vec_remove`,
`map_insert` / `map_insert_default` / `map_remove`.

## Going further: composing and customizing paths

Everything above uses derived paths, but `Path` and `Selector` are just traits —
you can implement them by hand and plug straight into the same machinery.
Korangar uses this for things the derive can't express. For example, a path that
resolves to "the currently selected service's settings" by combining two other
paths — using the value at one as the key into a map at another:

```rust,ignore
impl<P, S> Path<ClientState, ServiceSettings> for SelectedServicePath<P, S>
where
    P: Path<ClientState, LoginWindowState>,
    S: Path<ClientState, LoginSettings>,
{
    fn follow<'a>(&self, state: &'a ClientState) -> Option<&'a ServiceSettings> {
        let selected_service = self.window_state_path.selected_service().follow_safe(state);
        self.service_settings_path.follow_safe(state).service_settings.get(selected_service)
    }

    // ...follow_mut is analogous.
}
```

And because it composes cleanly with the derived container helpers, dynamic UI
lists fall out naturally — here Korangar builds one element per friend, each
holding a stable, asserted path to its own entry:

```rust,ignore
for index in self.elements.len()..friend_list.len() {
    let friend_path = self.friend_list_path.index(index).manually_asserted();

    self.elements.push(collapsible! {
        text: friend_path.name(),
        children: button! {
            text: client_state().localization().remove_button_text(),
            event: move |state: &State<ClientState>, queue: &mut EventQueue<ClientState>| {
                let &Friend { account_id, character_id, .. } = state.get(&friend_path);
                queue.queue(InputEvent::RemoveFriend { account_id, character_id });
            },
        },
    });
}
```

## Status & caveats

- **Nightly only.** The crate relies on the `auto_traits` and `negative_impls`
  features, and — because the derive macro emits a negative impl — *every
  downstream crate* must enable them too:
  ```rust
  #![feature(auto_traits, negative_impls)]
  ```
- **Evolving API.** This crate is developed alongside Korangar and changes to
  fit its needs. It has not been through a stabilization pass.
- **`apply()` is not transactional.** Queued changes are applied in order; if
  one fails the earlier ones are *not* rolled back. Errors are collected and
  returned together.
- **Derive coverage.** Path generation is implemented for structs (named and
  tuple). Enums derive `RustState` but do not currently generate per-variant
  path methods; unions are unsupported.
