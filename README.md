# Overview

**bevy_reactor** is a framework for fine-grained reactivity in Bevy. It implements reactive
concepts, such as signals, built on Bevy primitives: entities and components.

## Features

- Create hierarchies of entities that respond to reactive data sources such as mutable
  variables, Bevy resources and ECS components.
- Uses Bevy's built-in change detection for triggering reactions.
- Builder API allows easy construction of dynamic scenes and user interfaces.
- Copyable callback handles allows easily passing callbacks as parameters.
- Tracked ownership and automatic cleanup of reactions and callbacks.
- Simplified styling system eases the complexity of `bevy_ui`'s styling components, and allows
  for dynamic styling.

## Fine-Grained Reactivity

`bevy_reactor` implements "fine-grained" reactivity, which means that reactions can update
individual components or attributes, not just whole entities. In this respect, it is more like
[Leptos](https://www.leptos.dev/) or [Solid.js](https://www.solidjs.com/). There is no diffing or
VDOM as in [React.js](https://react.dev/), because the ability to do fine-grained updates makes a
VDOM unnecessary.

In coarse-grained frameworks like React or Dioxus, the "component" functions are re-run each
time the display graph is recomputed. In fine-grained frameworks, the component functions are
only executed once, but individual "micro-closures" defined within that function are run many times.

## Crates

The `bevy_reactor` project contains a number of crates:

- `bevy_reactor_signals` is the lowest-level layer, and represents the basic mechanisms
  for reactions and signals.
- `bevy_reactor_builders` contains the `UiBuilder` and `UiTemplate` types, which are used for
  constructing and updating dynamic user interfaces and scenes.
- `bevy_reactor_obsidian` provides the Obsidian widget library, an opinionated set of UI widgets
  intended for making editors and other utility programs. Obsidian widgets support common
  control types such as buttons, sliders, and splitter bars. The widgets also support accessibility
  and tab-navigation.
- `bevy_mod_stylebuilder` provides a set of convenience APIs for defining styles in Bevy user
  interfaces.
- Future crates:
  - `bevy_reactor_inspect` - a world inspector widget
  - `bevy_reactor_overlays` - reactive gizmos
  - `bevy_reactor_node_graph` - generic node-graph editor, for things like shader editors.

## Examples

There are several examples which demonstrate various aspects of `bevy_reactor`:

- The `builder` example shows the basic use of builders.
- The `buttons` example shows various kinds of buttons.
- The `controls` example shows other kinds of controls.

The most comprehensive example is named `complex` (note, currently broken):

```sh
cargo run --example complex
```

## Getting Started

To use this library, you'll need to install a number of plugins:

- `SignalsPlugin` enables the basic operations of signals and reactions.
- `StyleBuilderPlugin` is required for using style builders.
- `ObsidianUiPlugin` is required for using the Obsidian widget library.

# Usage

## Introduction to Reactive Contexts

A _reactive function_ is one that automatically re-runs whenever its dependencies change.
The dependencies are data structures, such as ECS resources and components, which know when they
have changed.

Reactive functions are always passed a _reaction context_ parameter whose type is `Rcx`. This
parameter has various methods that allow access to Bevy data, such as resources and components.
These methods automatically add the accessed item to a _tracking scope_. For example, if you
access a resource via `rcx.read_resource::<ResourceType>()`, the resource is added to the
current tracking scope; this means that in the future, each time the resource changes, the reactive
function will be run again.

Here's an example of a context parameter, which is traditionally named `rcx` (short for
"reactive context"):

```rust
// Spawn a new entity, returning an `EntityWorldMut`.
let element = world.spawn_empty();

// Conditionally add a component to the entity.
element.insert_if(|rcx| rcx.read_resource::<Counter>().count & 1 == 0, || Disabled)
```

The `insert_if` method takes two arguments:

- A reactive boolean function.
- A function which returns an ECS component.

The first function is run immediately, and then re-run whenever its dependencies change.
In this example, there's only one dependency, which in this case is the resource `Counter`.
Note that `Counter` was added as a dependency by the simple act of reading it's value, there's
no explicit `subscribe` or `unsubscribe` step.

This function returns a boolean value. Whenever this value changes, if it's true it will insert
the component generated by the second function, otherwise if it's false it will remove the
component.

In other words, the "if" isn't just evaluated one time: instead, it sticks around for the life
of the entity, and constantly ensures that the existence or non-existence of the component is
always kept in sync with the current state of the boolean predicate.

## Building Reactive Scenes and User Interfaces

`bevy_reactor` adds new builder methods, such as `.insert_if()` to Bevy's `EntityWorldMut` via
extension traits. These methods include:

- `insert_if` - conditionally inserts a component.
- `styles` - applies a list of _style builders_ to the entity (see later section for explanation
  of style builders).
- `style_dyn` - attaches a _dynamic style_ to the entity.
- `effect` - attaches a reactive effect function to the entity. The effect can perform any
  desired mutations to the entity.
- `create_children(builder_fn)` - calls `builder_fn` with a new `UiBuilder` instance.

`UiBuilder` is a struct that is used to build the children of a parent entity:

```rust
element.create_children(|builder| {
    builder.spawn((Node::default(), Name::new("Child 1")));
    builder.spawn((Node::default(), Name::new("Child 2")));
});
```

This is very similar to the existing `with_children()` method, except that the builder function
is passed a `UiBuilder` which has a lot of additional methods.

`UiBuilder` can do more than just spawning entities. It can also create reactions and
reactive data sources:

- `.cond()` creates a conditional branch, essentially an "if" statement which constructs either
  the "true" branch or the "false" branch depending on a test condition. If the condition changes,
  then the previous branch is despawned and the new branch constructed.
- `.switch()` is a more elaborate conditional branch, similar to a C "switch" statement.
- `.for_each()` will accept a reactive array value, and generate children for each array element.
  This generates a reactive loop which will "diff" the array from the previous value, and
  create and destroy the generated entities for array values which changed, preserving
  the ones that stayed the same.
- `.for_each_cmp()` is similar to `for_each` but allows specifying a custom comparator function
  for the array elements.
- `.for_index()` uses a different, and simpler algorithm for looping: instead of the "diff"
  algorithm used by `for_each`, it operates strictly by array index.
- `.text_computed()` creates dynamic text block.
- `.create_effect()` creates a generalized, side-effectful reaction which is "owned" by the
  parent entity, meaning that the reaction is despawned when the parent is.
- `.create_mutable()` creates a local mutable variable which is owned by the parent entity.
- `.create_derived()` creates a derived computation which is owned by the parent entity.
- `.invoke()` is used to call a template (see subsequent section).

`UiBuilder` also has some conveniece methods that are non-reactive, but useful when constructing
complex hierarchies:

- `.text()` creates a static (non-reactive) text block.
- `.create_callback()` registers a new one-shot system. This system is "owned" by the parent
  entity, meaning that it will be unregistered when the parent is despawned.

### Example using `cond`:

Here's an example of how to use `.cond()`:

```rust
builder.cond(
    // The test condition
    |rcx: &Rcx| {
        let counter = rcx.read_resource::<Counter>();
        counter.count & 1 == 0
    },
    // The true branch
    |builder| {
        builder.text("Counter is even");
    },
    // The false branch
    |builder| {
        builder.text("Counter is odd");
    },
)
```

The `cond` method takes three arguments:

- A reactive boolean condition. This can be a reactive function, as shown; but it can also
  be a `Signal<bool>` which we'll talk about subsequently.
- A builder function for the true branch. This is called when the condition is true.
- A builder function for the false branch. This is called when the condition is false.

When `.cond()` is called, the condition is evaluated immediately, and either the true or false
branch is constructed. Each branch can produce multiple children, or none. Later, if the data
dependencies change, the condition function will be run again, and if the result is different
from the previous time, then the old children will be despawned and new children built in their
place.

### Example using `for_each`:

Here's an example using `for_each`:

```rust
builder.for_each(
    // Reactive function which returns the array of items
    |rcx| {
        let suits = rcx.read_resource::<CardSuits>();
        suits.items.clone().into_iter()
    },
    // Function which builds the children for each array
    |item, builder| {
        builder.text(item.clone());
    },
    // Fallback function, called when the array is empty.
    |builder| {
        builder.text("List is empty!");
    },
);
```

In this example, we read a list of strings from a resource. (It doesn't have to be strings,
`for_each` will work with any data type that is `PartialEq` and `Clone`). This function returns
an iterator over the elements. The output of this iterator will be "diffed" with the previous
array (which is initially empty). For any "new" array elements (that is, elements which were not
present before), the second argument will be called to build child entities for that array.

In cases where the array is empty, it can be handy to render a placeholder, such as
"Search returned no results". The third argument can be used for this, it provides a "fallback"
function which is called when the array is empty. If you don't need a fallback, you can just
give it an empty builder.

## Mutables and Signals

Up to this point, all of our reactions have depended on Bevy resources via `.read_resource()`.
However, there are many other kinds of data dependencies available in `bevy_reactor`.

For example, you can also read components using `.read_component()` - but to use this, you have
to know the entity id.

In user-interfaces, it is often useful to be able to pass around references to reactive variables,
such that a parent widget can create a local variable and then pass a reference to that variable
to a child widget. This is a bit more complex than simply passing a Rust reference, because we
want the variable to remain reactive.

The `UiBuilder::create_mutable(value)` method creates a new `Mutable<T>`.

```rust
let pressed = builder.create_mutable::<bool>(false);
```

Mutables are like handles that point to a reactive variable. Internally, the mutable is simply a
Bevy `Entity`, but with some extra type information. Because it's an entity, it can be freely
passed around, copied, captured in closures, and so on.

Creating a mutable this way causes the entity to be added as a child of the builder's parent
entity. This means that when that parent is despawed, all of the mutables will also be despawned.

Accessing the data in a mutable can be done in one of several ways:

- Getting the data via `mutable.get(context)`;
- Setting the data via `mutable.set(context, value)`;
- Updating the data via `mutable.update(context, updater_fn)`;
- Getting a reference to the data via `mutable.as_ref(context)`;
- Transforming the data via `mutable.map(context, mapper_fn)`;
- Accessing the data via a signal: `mutable.signal()`;

The `context` object can be a reactive context like `Rcx`, but it can also be a `World` or
`DeferredWorld`. The reason we need the context object is because the actual data is stored in
Bevy's ECS and we need a way to retrieve it. `Mutable<T>` is just a handle, it doesn't contain
the data itself - but it does contain a type parameter which remembers what kind of data
is being stored.

If the context is a reactive context, such as `Rcx`, then reading the mutable will also create
a dependency on that variable. Passing a `World` or `DeferredWorld` will also read the value,
but will not create a dependency.

The `.get()`, `.set()` and `.signal()` methods given above assume that the data in the mutable
implements `Copy`. There is also a `.get_clone()` method, which works with data types that
implement `Clone`.

The call `mutable.signal()` returns a reactive signal object. Signals are objects that are used
to access the data from a reactive data source.

Why would you use a signal instead of simply calling `mutable.get()`? The reason is because
signals can represent other kinds of reactive data sources besides mutables. For example, say
you have a button widget that has a 'disabled' attribute. We want this attribute to be reactive,
so that the disabled state of the button will change when the data changes. By making `disabled`
a signal, we gain the flexibility to pass in different kinds of reactive values, including
constants, mutables, derived computations, and so on.

Signals have an API which is similar to mutables: `.get(context)`, `.map(context, mapper)` and so
on.

## Derived Computation

A derived computation is a signal resulting from a computation that depends on other signals.

```rust
/// A signal derived from a resource.
let panel_width = builder
    .create_derived(|cx| {
        let res = cx.read_resource::<PanelWidth>();
        res.0
    });
```

The `.create_derived()` method returns a `Signal<T>`. Reading a derived signal
adds all of the derived's dependencies to the current tracking scope, so if any of those
dependencies change, the caller of the derived will be re-run.

Derived signals are not memoized, however, for that we need to use `Memo` (still to be implemented).

## Internals: Tracking Scopes and Reactions

This section talks about some internal aspects of the framework which are not visible to the
outside, but which are important to understand.

A `TrackingScope` is a data structure which keeps track of all the reactive dependencies
accessed within a run context. This includes mutables, resources, components, and anything
else. It uses Bevy's change detection to determine whether a dependency has changed.

Tracking scopes are implemented as ECS components. They are often paired with "reactions",
which is another type of component that contains an action function.
The action function is run whenever the tracking scope indicates that one or more dependencies
have changed. When this happens, the tracking scope is first cleared; the reaction is expected
to re-subscribe to any dependencies that are needed.

Tracking scopes, reactions and mutables form the basis of more advanced reactive constructs
like memos and derivations.

## Callbacks

A `Callback` is just a wrapper around a one-shot `SystemId`. The only real difference between
callbacks and one-shot systems is that callbacks are "entity scoped", meaning that they are
automatically unregistered whenever their parent entity is despawned.

To create a callback, call `.create_callback()`:

```rust
let button_clicked = cx.create_callback(|_: In<()>| {
    println!("Button was clicked");
});
```

To call the callback, you need to call `.run_callback()`:

```rust
world.run_callback(button_click, ());
```

## UiTemplates

One important feature of the UI framework is to be able to define "widgets", that is, re-usable
modular sub-trees which have predefined behaviors.

One simple way to implement this would be to make widgets functions: to create a button, call
`button()`. The problem with this approach is that most widgets have a large number of parameters,
and so the function signature would quickly get complicated.

Also, most of the time we're only setting a small number of parameters, so we end up writing a lot
of boilerplate to pass in the default values.

To avoid this, we could use the "parameter builder" pattern, where you have some struct that
contains all the function parameters, and use a fluent API to populate them. This object then gets
passed into the function.

But if we're going to go through the trouble of defining a struct with all these properties, then
why do we need a standalone function at all? Why not just make it a method on the parameter object
and "cut out the middleman"?

This is what the `UiTemplate` trait does. It's actually a very simple trait:

```rust
pub trait UiTemplate {
    fn build(&self, builder: &mut UiBuilder);
}
```

A template is nothing more than a struct which implements this trait. The struct can have whatever
properties it needs, and can be initialized in whatever way you want. Once the template is
constructed, you can execute it by calling `builder.invoke(template)`.

Here's an example of invoking the Obsidian `Button` template:

```rust=
builder
    .invoke(
        Button::new()
            .variant(ButtonVariant::Primary)
            .labeled("Primary"),
    )
    .invoke(
        Button::new()
            .variant(ButtonVariant::Danger)
            .labeled("Danger"),
    )
```

Note that templates don't live very long: they are constructed and immediately executed; once
executed, they are dropped.

Interally, the button template just calls the builder argument:

```rust
impl UiTemplate for Button {
    fn build(&self, builder: &mut UiBuilder) {
        // Note: This example is massively simplified.
        // The real button template is 2 pages long.
        let label: String = self.label.into();
        builder.spawn()
           .style(button_style)
           .observe(click_handler)
           .create_children(|builder| {
               builder.text(label)
           });
    }
}
```

## Styles

An earlier version of this library implemented "CSS-like" stylesheets with dynamic selectors
and animation, but the current version provides a much simpler solution: "styles are just
functions":

```rust
fn style_button(ss: &mut StyleBuilder) {
    ss.border(1)
        .display(ui::Display::Flex)
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .padding_left(12)
        .padding_right(12)
        .border(1)
        .border_color(Color::WHITE);
}

fn style_button_size_md(ss: &mut StyleBuilder) {
    ss.min_height(Size::Xxxs.height());
}

builder.spawn(Node::default())
    .styles((
        style_button,
        style_button_size_md,
    ))
```

The `StyleBuilder` argument provides a fluent interface that allows the entity's styles to
be modified with lots of CSS-like shortcuts. For example, the following are all equivalent:

- `.border(ui::UiRect::all(ui::Val::Px(10.)))` -- a border of 10px on all sides.
- `.border(ui::Val::Px(10.))` -- Scalar is automatically converted to a rect.
- `.border(10.)` -- `Px` is assumed to be the default unit.
- `.border(10)` -- Integers are automatically converted to f32 type.

Unlike the CSS approach, there is no support for selectors, animated transitions, or serialization.
The style functions are simply executed once, in order, during the build phase. The main advantage
is that it provides a way to re-use styles without having to repeat the same properties over
and over again.

## Obsidian UI

The `bevy_reactor_obsidian` create defines a standard set of styles: colors, sizes, fonts and
other definitions that provide a consistent, "editor-like" look and feel, along with a collection
of common widgets.

Obsidian also provides a bunch of utilities for building widgets, including:

- Signals for detecting hover states.
- Signals for detecting keyboard focus states.
- Positioning algorithms for floating popups.
- Scrolling regions.

### Hover Signal

The `CreateHoverSignal` trait adds a `.create_hover_signal(entity)` method to `UiBuilder`. This
creates a derived reactive signal which can be used in conjunction with the `HoverMap`
which returns the entity currently being hovered by the mouse.
