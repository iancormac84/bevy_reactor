// mod atlas_loader;
mod builder;
mod builder_background;
mod builder_border_color;
mod builder_border_radius;
mod builder_font;
mod builder_layout;
mod builder_outline;
mod builder_pointer_events;
// mod builder_texture_atlas;
mod builder_z_index;

use std::sync::Arc;

// pub use atlas_loader::TextureAtlasLoader;
pub use builder::*;
pub use builder_background::StyleBuilderBackground;
pub use builder_border_color::StyleBuilderBorderColor;
pub use builder_border_radius::StyleBuilderBorderRadius;
pub use builder_font::{InheritableFontStyles, StyleBuilderFont, TextStyleChanged};
pub use builder_layout::StyleBuilderLayout;
pub use builder_outline::StyleBuilderOutline;
pub use builder_pointer_events::StyleBuilderPointerEvents;
// pub use builder_texture_atlas::StyleBuilderTextureAtlas;
pub use builder_z_index::StyleBuilderZIndex;
use impl_trait_for_tuples::*;

/// `StyleTuple` - a variable-length tuple of [`StyleHandle`]s.
pub trait StyleTuple: Sync + Send {
    /// Method to apply the style to a target entity.
    fn apply(&self, ctx: &mut StyleBuilder);

    /// Wrap the tuple in a [`StyleHandle`].
    fn into_handle(self) -> StyleHandle;
}

/// Empty tuple.
impl StyleTuple for () {
    fn apply(&self, _ctx: &mut StyleBuilder) {}

    fn into_handle(self) -> StyleHandle {
        StyleHandle::none()
    }
}

impl<F: Fn(&mut StyleBuilder) + Send + Sync + 'static> StyleTuple for F {
    fn apply(&self, ctx: &mut StyleBuilder) {
        (self)(ctx);
    }

    fn into_handle(self) -> StyleHandle {
        StyleHandle::new(self)
    }
}

impl StyleTuple for StyleHandle {
    fn apply(&self, ctx: &mut StyleBuilder) {
        if let Some(s) = self.style.as_ref() {
            s.apply(ctx);
        }
    }

    fn into_handle(self) -> StyleHandle {
        StyleHandle::new(self)
    }
}

#[impl_for_tuples(1, 16)]
impl StyleTuple for Tuple {
    for_tuples!( where #( Tuple: StyleTuple + 'static )* );

    fn apply(&self, ctx: &mut StyleBuilder) {
        for_tuples!( #( self.Tuple.apply(ctx); )* );
    }

    fn into_handle(self) -> StyleHandle {
        StyleHandle::new(self)
    }
}

/// Wrapper type that allows [`StyleTuple`]s to be passed from parent to child views.
#[derive(Default, Clone)]
pub struct StyleHandle {
    /// Reference to the collection of styles.
    pub style: Option<Arc<dyn StyleTuple>>,
}

impl StyleHandle {
    /// Construct a new style handle.
    pub fn new<S: StyleTuple + 'static>(style: S) -> Self {
        Self {
            style: Some(Arc::new(style)),
        }
    }

    /// Construct a placeholder style handle.
    pub fn none() -> Self {
        Self { style: None }
    }
}
