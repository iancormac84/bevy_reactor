use super::{toggle_state::ToggleState, Icon};
use crate::{
    animation::{AnimatedRotation, AnimatedTransition},
    colors,
    cursor::StyleBuilderCursor,
    hover_signal::CreateHoverSignal,
    prelude::{CreateFocusSignal, TabIndex},
    size::Size,
};
use accesskit::{Node as Accessible, Role};
use bevy::{
    a11y::AccessibilityNode,
    prelude::*,
    ui,
    window::SystemCursorIcon,
    winit::cursor::CursorIcon,
};
use bevy_mod_stylebuilder::*;
use bevy_reactor_builder::{
    CreateChilden, EntityStyleBuilder, InvokeUiTemplate, UiBuilder, UiTemplate,
};
use bevy_reactor_signals::{Callback, IntoSignal, Signal};

fn style_toggle(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .justify_content(ui::JustifyContent::Center)
        .align_items(ui::AlignItems::Center)
        .align_content(ui::AlignContent::Center)
        .color(colors::FOREGROUND)
        .cursor(CursorIcon::System(SystemCursorIcon::Pointer));
}

/// A widget which displays small toggleable chevron that can be used to control whether
/// a panel is visible or hidden.
#[derive(Default)]
pub struct DisclosureToggle {
    /// Whether the toggle is in an expanded state.
    pub expanded: Signal<bool>,

    /// Button size.
    pub size: Size,

    /// Whether the button is disabled.
    pub disabled: Signal<bool>,

    /// Additional styles to be applied to the button.
    pub style: StyleHandle,

    /// Callback called when the state is toggled
    pub on_change: Option<Callback<bool>>,

    /// The tab index of the button (default 0).
    pub tab_index: i32,

    /// If true, set focus to this button when it's added to the UI.
    pub autofocus: bool,
}

impl DisclosureToggle {
    /// Construct a new `DisclosureToggle`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the expanded state of the button.
    pub fn expanded(mut self, expanded: impl IntoSignal<bool>) -> Self {
        self.expanded = expanded.into_signal();
        self
    }

    /// Set the button size.
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Set the button disabled state.
    pub fn disabled(mut self, disabled: impl IntoSignal<bool>) -> Self {
        self.disabled = disabled.into_signal();
        self
    }

    /// Set the additional styles for the button.
    pub fn style<S: StyleTuple + 'static>(mut self, style: S) -> Self {
        self.style = style.into_handle();
        self
    }

    /// Set callback when clicked
    pub fn on_change(mut self, callback: Callback<bool>) -> Self {
        self.on_change = Some(callback);
        self
    }

    /// Set the tab index of the button.
    pub fn tab_index(mut self, tab_index: i32) -> Self {
        self.tab_index = tab_index;
        self
    }

    /// Set whether to autofocus the button when it's added to the UI.
    pub fn autofocus(mut self, autofocus: bool) -> Self {
        self.autofocus = autofocus;
        self
    }
}

impl UiTemplate for DisclosureToggle {
    fn build(&self, builder: &mut UiBuilder) {
        let disabled = self.disabled;
        let checked = self.expanded;
        let id = builder
            .spawn((Node::default(), Name::new("DisclosureToggle")))
            .id();
        let hovering = builder.create_hover_signal(id);
        let focused = builder.create_focus_visible_signal(id);

        builder.create_effect(move |ecx| {
            let checked = checked.get(ecx);
            let mut entt = ecx.world_mut().entity_mut(id);
            let angle = if checked {
                std::f32::consts::PI * 0.5
            } else {
                0.
            };
            let target = Quat::from_rotation_z(angle);
            AnimatedTransition::<AnimatedRotation>::start(&mut entt, target, None, 0.3);
        });

        builder
            .entity_mut(id)
            .styles((style_toggle, self.style.clone()))
            .insert((
                ToggleState {
                    on_change: self.on_change,
                    checked: self.expanded,
                },
                TabIndex(self.tab_index),
                AccessibilityNode::from(Accessible::new(Role::CheckBox)),
            ))
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
                let icon_color = builder.create_derived(move |rcx| {
                    let is_disabled = disabled.get(rcx);
                    let is_hover = hovering.get(rcx);
                    match (is_disabled, is_hover) {
                        (true, _) => Color::from(colors::DIM).with_alpha(0.2),
                        (false, true) => Color::from(colors::FOREGROUND),
                        (false, false) => Color::from(colors::DIM),
                    }
                });

                builder.invoke(
                    Icon::new("embedded://bevy_reactor_obsidian/assets/icons/chevron_right.png")
                        .color(icon_color)
                        .size(match self.size {
                            Size::Xl => Vec2::splat(24.),
                            Size::Lg => Vec2::splat(20.),
                            Size::Md => Vec2::splat(18.),
                            Size::Sm => Vec2::splat(16.),
                            Size::Xs => Vec2::splat(13.),
                            Size::Xxs => Vec2::splat(12.),
                            Size::Xxxs => Vec2::splat(11.),
                        })
                        .style(|ss: &mut StyleBuilder| {
                            ss.margin_right(2);
                        }),
                );
            });
    }
}
