use bevy::prelude::*;
use bevy_mod_stylebuilder::UseInheritedTextStyles;
use bevy_reactor_signals::{Rcx, Reaction, ReactionCell, TrackingScope};

use crate::UiBuilder;

pub trait TextBuilder {
    fn text(&mut self, s: impl Into<String>) -> &mut Self;
    fn text_computed<F: FnMut(&Rcx) -> String + Send + Sync + 'static>(
        &mut self,
        text_fn: F,
    ) -> &mut Self;
}

impl<'w> TextBuilder for WorldChildBuilder<'w> {
    /// Create a static text entity with a single section.
    fn text(&mut self, s: impl Into<String>) -> &mut Self {
        self.spawn((
            Name::new("TextStatic"),
            TextLayout::default(),
            Text(s.into()),
            TextStyle::default(),
            UseInheritedTextStyles,
        ));
        self
    }

    /// Create a computed text entity.
    fn text_computed<F: FnMut(&Rcx) -> String + Send + Sync + 'static>(
        &mut self,
        text_fn: F,
    ) -> &mut Self {
        let mut node = self.spawn(Name::new("TextComputed"));
        let tick = node.world().last_change_tick();
        let mut tracking = TrackingScope::new(tick);
        let re = Rcx::new(node.world(), node.id(), &mut tracking);
        let mut reaction = TextComputedReaction {
            node: node.id(),
            text_fn,
        };
        let text = (reaction.text_fn)(&re);
        node.insert((
            tracking,
            Name::new("TextComputed"),
            TextLayout::default(),
            Text(text),
            TextStyle::default(),
            UseInheritedTextStyles,
            ReactionCell::new(reaction),
        ));
        self
    }
}

impl<'w> TextBuilder for UiBuilder<'w> {
    /// Create a static text entity with a single section.
    fn text(&mut self, s: impl Into<String>) -> &mut Self {
        self.spawn((
            Name::new("TextStatic"),
            TextLayout::default(),
            Text(s.into()),
            TextStyle::default(),
            UseInheritedTextStyles,
        ));
        self
    }

    /// Create a computed text entity.
    fn text_computed<F: FnMut(&Rcx) -> String + Send + Sync + 'static>(
        &mut self,
        text_fn: F,
    ) -> &mut Self {
        let mut node = self.spawn(Name::new("TextComputed"));
        let tick = node.world().last_change_tick();
        let mut tracking = TrackingScope::new(tick);
        let re = Rcx::new(node.world(), node.id(), &mut tracking);
        let mut reaction = TextComputedReaction {
            node: node.id(),
            text_fn,
        };
        let text = (reaction.text_fn)(&re);
        node.insert((
            tracking,
            Name::new("TextComputed"),
            TextLayout::default(),
            Text(text),
            TextStyle::default(),
            UseInheritedTextStyles,
            ReactionCell::new(reaction),
        ));
        self
    }
}

/// A UI element that displays text that is dynamically computed.
struct TextComputedReaction<F: FnMut(&Rcx) -> String + Send + Sync + 'static> {
    /// The visible UI node for this element.
    node: Entity,

    /// The function that produces the text to display
    text_fn: F,
}

impl<F: FnMut(&Rcx) -> String + Send + Sync + 'static> Reaction for TextComputedReaction<F> {
    fn react(&mut self, owner: Entity, world: &mut World, tracking: &mut TrackingScope) {
        let re = Rcx::new(world, owner, tracking);
        let text = (self.text_fn)(&re);
        world.entity_mut(self.node).get_mut::<Text>().unwrap().0 = text;
    }
}
