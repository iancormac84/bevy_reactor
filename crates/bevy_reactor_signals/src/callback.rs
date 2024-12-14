use std::sync::Arc;

use bevy::{
    ecs::{
        system::SystemId,
        world::{Command, DeferredWorld},
    },
    prelude::*,
};

use crate::Ecx;

/// Contains a reference to a callback. `P` is the type of the props.
#[derive(PartialEq, Debug)]
pub struct Callback<P: 'static = ()> {
    pub(crate) id: SystemId<In<P>, ()>,
}

impl<P> Callback<P> {
    /// Construct a new callback
    pub fn new(id: SystemId<In<P>, ()>) -> Self {
        Self { id }
    }
}

impl<P> Copy for Callback<P> {}
impl<P> Clone for Callback<P> {
    fn clone(&self) -> Self {
        *self
    }
}

pub trait AnyCallback: 'static {
    fn remove(&self, world: &mut World);
}

impl<P: 'static> AnyCallback for Callback<P> {
    fn remove(&self, world: &mut World) {
        // println!("Removing callback");
        world.unregister_system(self.id).unwrap();
    }
}

/// Component which tracks ownership of callbacks.
#[derive(Component, Default)]
pub struct CallbackOwner(Vec<Arc<dyn AnyCallback + Send + Sync>>);

impl CallbackOwner {
    /// Construct a new `CallbackOwner` component.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an entry to the list of owned callbacks.
    pub fn add<P: 'static>(&mut self, callback: Callback<P>) {
        self.0.push(Arc::new(callback));
    }
}

pub(crate) fn cleanup_callbacks(world: &mut World) {
    world
        .register_component_hooks::<CallbackOwner>()
        .on_remove(|mut world, entity, _component| {
            let mut callbacks = world.get_mut::<CallbackOwner>(entity).unwrap();
            let mut callbacks = std::mem::take(&mut callbacks.0);
            for callback in callbacks.drain(..) {
                world.commands().queue(UnregisterCallbackCmd(callback));
            }
        });
}

/// A trait for invoking callbacks.
pub trait RunCallback {
    /// Invoke a callback with the given props.
    fn run_callback<P: Send>(&mut self, callback: Callback<P>, props: P);
}

/// A mutable reactive context. This allows write access to reactive data sources.
impl RunCallback for World {
    /// Invoke a callback with the given props.
    ///
    /// Arguments:
    /// * `callback` - The callback to invoke.
    /// * `props` - The props to pass to the callback.
    fn run_callback<P>(&mut self, callback: Callback<P>, props: P) {
        self.run_system_with_input(callback.id, props).unwrap();
    }
}

/// A mutable reactive context. This allows write access to reactive data sources.
impl<'w> RunCallback for DeferredWorld<'w> {
    /// Invoke a callback with the given props.
    ///
    /// Arguments:
    /// * `callback` - The callback to invoke.
    /// * `props` - The props to pass to the callback.
    fn run_callback<P: Send>(&mut self, callback: Callback<P>, props: P) {
        self.commands().run_system_with_input(callback.id, props);
    }
}

impl<'p, 'w> RunCallback for Ecx<'p, 'w> {
    fn run_callback<P: Send>(&mut self, callback: Callback<P>, props: P) {
        self.world_mut().run_callback(callback, props);
    }
}

impl<'w, 's> RunCallback for Commands<'w, 's> {
    fn run_callback<P: Send>(&mut self, callback: Callback<P>, props: P) {
        self.run_system_with_input(callback.id, props)
    }
}

pub(crate) struct UnregisterCallbackCmd(pub(crate) Arc<dyn AnyCallback + Send + Sync>);

impl Command for UnregisterCallbackCmd {
    fn apply(self, world: &mut World) {
        self.0.remove(world)
    }
}
