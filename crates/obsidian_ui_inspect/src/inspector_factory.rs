use bevy::prelude::*;
use bevy_reactor::*;
use bevy_reactor_signals::Cx;

use crate::Inspectable;

/// Trait that defines a factory for creating inspectors. Multiple factories can be registered,
/// and the first one that returns true will be used to create the inspector.
pub trait InspectorFactory: Sync + Send {
    /// Examine the reflect data and decide what kind of widget to create to edit the
    /// data. Can return false if the data is not in a supported format.
    fn create_inspector(&self, reflect: &Cx, field: &Inspectable) -> Option<ViewRef>;
}

#[derive(Resource, Default)]
pub struct InspectorFactoryRegistry(pub Vec<Box<dyn InspectorFactory>>);

pub trait RegisterInspectorFactory {
    fn register_inspector<T: InspectorFactory + Default + 'static>(&mut self) -> &mut Self;
}

impl RegisterInspectorFactory for App {
    fn register_inspector<T: InspectorFactory + Default + 'static>(&mut self) -> &mut Self {
        match self
            .world_mut()
            .get_resource_mut::<InspectorFactoryRegistry>()
        {
            Some(mut registry) => {
                registry.0.push(Box::<T>::default());
            }
            None => {
                self.world_mut()
                    .insert_resource(InspectorFactoryRegistry(vec![Box::<T>::default()]));
            }
        }
        self
    }
}
