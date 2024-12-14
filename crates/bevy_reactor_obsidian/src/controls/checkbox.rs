use std::sync::Arc;

use accesskit::{Node as Accessible, Role};
use bevy::{
    a11y::AccessibilityNode,
    color::Luminance,
    prelude::*,
    ui,
    window::SystemCursorIcon,
    winit::cursor::CursorIcon,
};
use bevy_mod_stylebuilder::*;
use bevy_reactor_builder::*;
use bevy_reactor_signals::{Callback, IntoSignal, Signal};

use crate::{
    colors,
    cursor::StyleBuilderCursor,
    hover_signal::CreateHoverSignal,
    prelude::{CreateFocusSignal, TabIndex},
    typography,
};

use super::{toggle_state::ToggleState, Disabled};

fn style_checkbox(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .justify_content(ui::JustifyContent::Center)
        .align_items(ui::AlignItems::Center)
        .align_content(ui::AlignContent::Center)
        .gap(4)
        .color(colors::FOREGROUND)
        .cursor(CursorIcon::System(SystemCursorIcon::Pointer));
}

fn style_checkbox_border(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .width(16)
        .height(16)
        .border_radius(3.0);
}

fn style_checkbox_inner(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .background_image("embedded://bevy_reactor_obsidian/assets/icons/checkmark.png")
        .position(ui::PositionType::Absolute)
        .left(2)
        .top(2)
        .width(12)
        .height(12);
}

fn style_checkbox_label(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .justify_content(ui::JustifyContent::FlexStart)
        .align_items(ui::AlignItems::Center)
        .font_size(14)
        .color(colors::FOREGROUND);
}

/// A checkbox widget.
pub struct Checkbox {
    /// Whether the checkbox is checked.
    pub checked: Signal<bool>,

    /// Whether the checkbox is disabled.
    pub disabled: Signal<bool>,

    /// The content to display inside the button.
    pub label: Arc<dyn Fn(&mut UiBuilder)>,

    /// Additional styles to be applied to the button.
    pub style: StyleHandle,

    /// Callback called when clicked
    pub on_change: Option<Callback<bool>>,

    /// The tab index of the checkbox (default 0).
    pub tab_index: i32,
}

impl Default for Checkbox {
    fn default() -> Self {
        Self {
            checked: Default::default(),
            disabled: Default::default(),
            label: Arc::new(|_builder| {}),
            style: Default::default(),
            on_change: Default::default(),
            tab_index: Default::default(),
        }
    }
}

impl Checkbox {
    /// Create a new checkbox.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the checked state of the checkbox.
    pub fn checked(mut self, checked: impl IntoSignal<bool>) -> Self {
        self.checked = checked.into_signal();
        self
    }

    /// Set the disabled state of the checkbox.
    pub fn disabled(mut self, disabled: impl IntoSignal<bool>) -> Self {
        self.disabled = disabled.into_signal();
        self
    }

    /// Set the label of the checkbox.
    pub fn labeled(mut self, label: impl Into<String>) -> Self {
        let s: String = label.into();
        self.label = Arc::new(move |builder| {
            // TODO: Figure out how to avoid the double-copy here.
            builder.text(s.clone());
        });
        self
    }

    /// Set the style of the checkbox.
    pub fn style<S: StyleTuple + 'static>(mut self, style: S) -> Self {
        self.style = style.into_handle();
        self
    }

    /// Set the on_change callback of the checkbox.
    pub fn on_change(mut self, on_change: Callback<bool>) -> Self {
        self.on_change = Some(on_change);
        self
    }

    /// Set the tab index of the checkbox.
    pub fn tab_index(mut self, tab_index: i32) -> Self {
        self.tab_index = tab_index;
        self
    }
}

impl UiTemplate for Checkbox {
    /// Construct a checkbox widget.
    fn build(&self, builder: &mut UiBuilder) {
        let id = builder.spawn((Node::default(), Name::new("Checkbox"))).id();
        let hovering = builder.create_hover_signal(id);
        let focused = builder.create_focus_visible_signal(id);

        let checked = self.checked;
        let disabled = self.disabled;

        builder
            .world_mut()
            .entity_mut(id)
            .styles((style_checkbox, self.style.clone()))
            .insert((
                TabIndex(self.tab_index),
                ToggleState {
                    checked,
                    on_change: self.on_change,
                },
                AccessibilityNode::from(Accessible::new(Role::CheckBox)),
            ))
            .insert_if(self.disabled, || Disabled)
            .create_children(|builder| {
                builder
                    .spawn((Node::default(), Name::new("Checkbox::Border")))
                    .style(style_checkbox_border)
                    .style_dyn(
                        move |rcx| {
                            let is_checked = checked.get(rcx);
                            let is_disabled = disabled.get(rcx);
                            let is_hovering = hovering.get(rcx);
                            match (is_checked, is_disabled, is_hovering) {
                                (true, true, _) => colors::ACCENT.with_alpha(0.2),
                                (true, false, true) => colors::ACCENT.darker(0.15),
                                (true, _, _) => colors::ACCENT.darker(0.2),
                                (false, true, _) => colors::U1.with_alpha(0.7),
                                (false, false, true) => colors::U1.lighter(0.002),
                                (false, false, false) => colors::U1,
                            }
                        },
                        |color, sb| {
                            sb.background_color(color);
                        },
                    )
                    .style_dyn(
                        move |rcx| focused.get(rcx),
                        |is_focused, sb| {
                            if is_focused {
                                sb.outline_color(colors::FOCUS)
                                    .outline_offset(2)
                                    .outline_width(2);
                            } else {
                                sb.outline_color(colors::TRANSPARENT).outline_width(0);
                            }
                        },
                    )
                    .create_children(|builder| {
                        builder.cond(
                            checked,
                            move |builder| {
                                builder.spawn(Node::default()).style(style_checkbox_inner);
                            },
                            |_| {},
                        );
                    });

                builder
                    .spawn(Node::default())
                    .styles((typography::text_default, style_checkbox_label))
                    .style_dyn(
                        move |rcx| disabled.get(rcx),
                        |disabled, sb| {
                            if disabled {
                                sb.color(colors::FOREGROUND.with_alpha(0.2));
                            } else {
                                sb.color(colors::FOREGROUND);
                            }
                        },
                    )
                    .create_children(|builder| {
                        (self.label.as_ref())(builder);
                    });
            });
    }
}
