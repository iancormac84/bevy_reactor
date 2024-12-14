use std::sync::Arc;

use crate::{prelude::RoundedCorners, size::Size};
use accesskit::{Node as Accessible, Role};
use bevy::{
    a11y::AccessibilityNode,
    prelude::*,
    ui,
};
use bevy_mod_stylebuilder::*;
use bevy_reactor_builder::{
    CreateChilden, EntityStyleBuilder, InvokeUiTemplate, UiBuilder, UiTemplate,
};
use bevy_reactor_signals::{Callback, IntoSignal, Signal};

use super::{Button, ButtonVariant};

fn style_tool_palette(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Grid)
        .gap(1)
        // .justify_content(ui::JustifyContent::Center)
        // .align_items(ui::AlignItems::Center)
        // .align_content(ui::AlignContent::Center);
        .grid_auto_rows(vec![ui::GridTrack::default()]);
}

#[derive(Clone, Debug, Default, Component)]
struct ToolPaletteContext {
    size: Size,
}

/// ToolPalette - a grid of tool buttons
pub struct ToolPalette {
    /// Button size.
    pub size: Size,

    /// The buttons to display.
    pub children: Arc<dyn Fn(&mut UiBuilder)>,

    /// Additional styles to be applied to the palette.
    pub style: StyleHandle,

    /// Number of button columns
    pub columns: u16,
}

impl Default for ToolPalette {
    fn default() -> Self {
        Self {
            size: Default::default(),
            children: Arc::new(|_builder| {}),
            style: Default::default(),
            columns: Default::default(),
        }
    }
}

impl ToolPalette {
    /// Create a new tool palette.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the button size.
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Set the child views for this element.
    pub fn children<V: 'static + Fn(&mut UiBuilder)>(mut self, children: V) -> Self {
        self.children = Arc::new(children);
        self
    }

    /// Set additional styles to be applied to the palette.
    pub fn style<S: StyleTuple + 'static>(mut self, style: S) -> Self {
        self.style = style.into_handle();
        self
    }

    /// Set the number of button columns.
    pub fn columns(mut self, columns: u16) -> Self {
        self.columns = columns;
        self
    }
}

impl UiTemplate for ToolPalette {
    fn build(&self, builder: &mut UiBuilder) {
        let columns = self.columns;

        builder
            .spawn((Node::default(), Name::new("ToolPalette")))
            .styles((
                style_tool_palette,
                move |ss: &mut StyleBuilder| {
                    ss.grid_template_columns(vec![ui::RepeatedGridTrack::auto(columns)]);
                },
                self.style.clone(),
            ))
            .insert(ToolPaletteContext { size: self.size })
            .insert(AccessibilityNode::from(Accessible::new(Role::Group)))
            .create_children(|builder| {
                (self.children.as_ref())(builder);
            });
    }
}

/// A button in a ToolPalette.
pub struct ToolButton {
    /// Color variant - default, primary or danger.
    pub variant: Signal<ButtonVariant>,

    /// Whether the button is disabled.
    pub disabled: Signal<bool>,

    /// The content to display inside the button.
    pub children: Arc<dyn Fn(&mut UiBuilder)>,

    /// Callback called when clicked
    pub(crate) on_click: Option<Callback>,

    /// The tab index of the button (default 0).
    pub(crate) tab_index: i32,

    /// Which corners to render rounded.
    pub(crate) corners: RoundedCorners,

    /// If true, set focus to this button when it's added to the UI.
    pub(crate) autofocus: bool,
}

impl ToolButton {
    /// Create a new tool button.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the button color variant.
    pub fn variant(&mut self, variant: impl IntoSignal<ButtonVariant>) -> &mut Self {
        self.variant = variant.into_signal();
        self
    }

    /// Method which switches between `default` and `selected` style variants based on a boolean.
    /// Often used for toggle buttons or toolbar items.
    pub fn selected(mut self, selected: bool) -> Self {
        self.variant = if selected {
            ButtonVariant::Selected
        } else {
            ButtonVariant::Default
        }
        .into_signal();
        self
    }

    /// Set the button disabled state.
    pub fn disabled(&mut self, disabled: impl IntoSignal<bool>) -> &mut Self {
        self.disabled = disabled.into_signal();
        self
    }

    /// Set the child views for this element.
    pub fn children<V: 'static + Fn(&mut UiBuilder)>(mut self, children: V) -> Self {
        self.children = Arc::new(children);
        self
    }

    /// Set callback when clicked
    pub fn on_click(mut self, callback: Callback) -> Self {
        self.on_click = Some(callback);
        self
    }

    /// Set the tab index of the button.
    pub fn tab_index(mut self, tab_index: i32) -> Self {
        self.tab_index = tab_index;
        self
    }

    /// Set which corners to render rounded.
    pub fn corners(mut self, corners: RoundedCorners) -> Self {
        self.corners = corners;
        self
    }

    /// Set whether to autofocus the button when it's added to the UI.
    pub fn autofocus(mut self, autofocus: bool) -> Self {
        self.autofocus = autofocus;
        self
    }
}

impl Default for ToolButton {
    fn default() -> Self {
        Self {
            variant: Default::default(),
            disabled: Default::default(),
            children: Arc::new(|_builder| {}),
            on_click: Default::default(),
            tab_index: 0,
            corners: RoundedCorners::None,
            autofocus: false,
        }
    }
}

impl UiTemplate for ToolButton {
    fn build(&self, builder: &mut UiBuilder) {
        let context = builder
            .use_inherited_component::<ToolPaletteContext>()
            .unwrap();
        let mut btn = Button::new()
            .size(context.size)
            .variant(self.variant)
            .disabled(self.disabled)
            .tab_index(self.tab_index)
            .autofocus(self.autofocus)
            .corners(self.corners);
        btn.children = self.children.clone();
        btn.on_click = self.on_click;
        builder.invoke(btn);
    }
}
