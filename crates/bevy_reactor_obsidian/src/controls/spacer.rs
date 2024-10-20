use bevy::prelude::*;
use bevy_mod_stylebuilder::*;
use bevy_reactor_builder::{EntityStyleBuilder, UiBuilder, UiTemplate};

fn style_spacer(ss: &mut StyleBuilder) {
    ss.flex_grow(1.);
}

/// A spacer widget that fills the available space.
#[derive(Clone, Default)]
pub struct Spacer;

impl UiTemplate for Spacer {
    fn build(&self, builder: &mut UiBuilder) {
        builder.spawn(Node::default()).style(style_spacer);
    }
}
