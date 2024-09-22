use std::{any::TypeId, sync::Arc};

use bevy::{
    ecs::{
        system::SystemId,
        world::{Command, DeferredWorld},
    },
    prelude::*,
};

/// Contains a reference to a callback. `P` is the type of the props.
#[derive(PartialEq, Debug)]
pub struct Callback<P = ()> {
    pub(crate) id: SystemId<P, ()>,
}

pub trait AnyCallback: 'static {
    fn remove(&self, world: &mut World);
    fn type_id(&self) -> TypeId;
}

impl dyn AnyCallback + Send + Sync {
    /// Get the original typed callback.
    pub fn downcast<P: 'static>(&self) -> Callback<P> {
        if TypeId::of::<P>() == self.type_id() {
            // Safe because we just checked the type.
            unsafe { *(self as *const dyn AnyCallback as *const Callback<P>) }
        } else {
            panic!("downcast failed")
        }
    }
}

impl<P: 'static> AnyCallback for Callback<P> {
    fn remove(&self, world: &mut World) {
        // println!("Removing callback");
        world.remove_system(self.id).unwrap();
    }
    fn type_id(&self) -> TypeId {
        TypeId::of::<P>()
    }
}

impl<P> Copy for Callback<P> {}
impl<P> Clone for Callback<P> {
    fn clone(&self) -> Self {
        *self
    }
}

/// A trait for invoking callbacks.
pub trait RunCallback {
    /// Invoke a callback with the given props.
    fn run_callback<P: 'static + Send>(&mut self, callback: Callback<P>, props: P);
}

/// A mutable reactive context. This allows write access to reactive data sources.
impl RunCallback for World {
    /// Invoke a callback with the given props.
    ///
    /// Arguments:
    /// * `callback` - The callback to invoke.
    /// * `props` - The props to pass to the callback.
    fn run_callback<P: 'static>(&mut self, callback: Callback<P>, props: P) {
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
    fn run_callback<P: 'static + Send>(&mut self, callback: Callback<P>, props: P) {
        self.commands().run_system_with_input(callback.id, props);
    }
}

// impl<'p, 'w> RunCallback for Rcx<'p, 'w> {
//     fn run_callback<P: 'static + Send>(&mut self, callback: Callback<P>, props: P) {
//         self.world().commands().run_callback(callback, props);
//         // self.world_mut().run_callback(callback, props);
//     }
// }

impl<'w, 's> RunCallback for Commands<'w, 's> {
    fn run_callback<P: 'static + Send>(&mut self, callback: Callback<P>, props: P) {
        self.run_system_with_input(callback.id, props)
    }
}

pub(crate) struct UnregisterCallbackCmd(pub(crate) Arc<dyn AnyCallback + Send + Sync>);

impl Command for UnregisterCallbackCmd {
    fn apply(self, world: &mut World) {
        self.0.remove(world)
    }
}
