mod attributes;
mod default_factory;
mod inspectable;
mod inspector;
mod inspector_factory;
mod inspectors;
mod templates;

use bevy::app::{App, Plugin};
use default_factory::DefaultInspectorFactory;

pub use attributes::*;
pub use inspectable::*;
pub use inspector::*;
pub use inspector_factory::*;
use templates::color_edit::RecentColors;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.register_inspector::<DefaultInspectorFactory>()
            .init_resource::<RecentColors>();
    }
}
