use std::sync::Arc;

use crate::{
    colors,
    cursor::StyleBuilderCursor,
    focus_signal::CreateFocusSignal,
    hover_signal::CreateHoverSignal,
    input_dispatch::{FocusKeyboardInput, KeyboardFocus, KeyboardFocusVisible},
    prelude::RoundedCorners,
    size::Size,
    tab_navigation::{AutoFocus, TabIndex},
    typography,
};
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
use bevy_reactor_builder::{
    CreateChilden, EntityStyleBuilder, InsertComponentBuilder, TextBuilder, UiBuilder, UiTemplate,
};
use bevy_reactor_signals::{Callback, IntoSignal, RunCallback, Signal};

use super::{Disabled, IsDisabled};

/// The variant determines the button's color scheme
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum ButtonVariant {
    /// The default button apperance.
    #[default]
    Default,

    /// A more prominent, "call to action", appearance.
    Primary,

    /// An appearance indicating a potentially dangerous action.
    Danger,

    /// A button that is in a "toggled" state.
    Selected,
}

pub(crate) fn style_button(ss: &mut StyleBuilder) {
    ss.border(1)
        .display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .justify_content(ui::JustifyContent::Center)
        .align_items(ui::AlignItems::Center)
        .align_content(ui::AlignContent::Center)
        .padding((12, 0))
        .border(0)
        .color(colors::FOREGROUND)
        .cursor(CursorIcon::System(SystemCursorIcon::Pointer));
}

pub(crate) fn style_button_bg(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Grid)
        .position(ui::PositionType::Absolute)
        .left(0)
        .right(0)
        .top(0)
        .bottom(0);
}

#[derive(Component)]
pub struct ButtonPressed(pub bool);

#[derive(Component)]
pub(crate) struct ButtonState {
    on_click: Option<Callback>,
}

/// Button widget
pub struct Button {
    /// Color variant - default, primary or danger.
    pub variant: Signal<ButtonVariant>,

    /// Button size.
    pub size: Size,

    /// Whether the button is disabled.
    pub disabled: Signal<bool>,

    /// The content to display inside the button.
    pub children: Arc<dyn Fn(&mut UiBuilder)>,

    /// Additional styles to be applied to the button.
    pub style: StyleHandle,

    /// Callback called when clicked
    pub on_click: Option<Callback>,

    /// The tab index of the button (default 0).
    pub tab_index: i32,

    /// Which corners to render rounded.
    pub corners: RoundedCorners,

    /// If true, set focus to this button when it's added to the UI.
    pub autofocus: bool,

    /// If true, render the button in a 'minimal' style with no background and reduced padding.
    pub minimal: bool,
}

impl Button {
    /// Construct a new `Button`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the button color variant.
    pub fn variant(mut self, variant: impl IntoSignal<ButtonVariant>) -> Self {
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

    /// Set whether to render the button in a 'minimal' style with no background and reduced padding.
    pub fn minimal(mut self, minimal: bool) -> Self {
        self.minimal = minimal;
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

    /// Set the child views for this element.
    pub fn children<V: 'static + Fn(&mut UiBuilder)>(mut self, children: V) -> Self {
        self.children = Arc::new(children);
        self
    }

    /// Set a child which is a text label.
    pub fn labeled(mut self, label: impl Into<String>) -> Self {
        let s: String = label.into();
        self.children = Arc::new(move |builder| {
            // TODO: Figure out how to avoid the double-copy here.
            builder.text(s.clone());
        });
        self
    }

    /// Set the additional styles for the button.
    pub fn style<S: StyleTuple + 'static>(mut self, style: S) -> Self {
        self.style = style.into_handle();
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

impl Default for Button {
    fn default() -> Self {
        Self {
            variant: Signal::default(),
            size: Size::default(),
            disabled: default(),
            children: Arc::new(|_builder| {}),
            style: StyleHandle::none(),
            on_click: None,
            tab_index: 0,
            corners: RoundedCorners::default(),
            autofocus: false,
            minimal: false,
        }
    }
}

impl UiTemplate for Button {
    fn build(&self, builder: &mut UiBuilder) {
        let variant = self.variant;

        let corners = self.corners;
        let minimal = self.minimal;

        let size = self.size;
        let on_click = self.on_click;

        let button = builder.spawn((Node::default(), Name::new("Button")));
        let button_id = button.id();
        let hovering = builder.create_hover_signal(button_id);
        let focused = builder.create_focus_visible_signal(button_id);
        let mut button = builder.world_mut().entity_mut(button_id);

        button
            .styles((
                typography::text_default,
                style_button,
                move |ss: &mut StyleBuilder| {
                    ss.min_height(size.height())
                        .min_width(size.height().floor())
                        .font_size(size.font_size());
                    if minimal {
                        ss.padding(0);
                    } else {
                        ss.padding((size.font_size() * 0.75, 0));
                    }
                },
                self.style.clone(),
            ))
            .insert_if(self.disabled, || Disabled)
            .insert((
                TabIndex(self.tab_index),
                ButtonPressed(false),
                ButtonState { on_click },
                AccessibilityNode::from(Accessible::new(Role::Button)),
            ))
            .insert_if(self.autofocus, || AutoFocus)
            .create_children(|builder| {
                builder
                    .spawn((Node::default(), Name::new("Button::Background")))
                    .style(style_button_bg)
                    .insert(corners.to_border_radius(self.size.border_radius()))
                    .style_dyn(
                        move |rcx| {
                            if minimal {
                                colors::TRANSPARENT
                            } else {
                                let pressed =
                                    rcx.read_component::<ButtonPressed>(button_id).unwrap();
                                let disabled = rcx.is_disabled(button_id);
                                button_bg_color(
                                    variant.get(rcx),
                                    disabled,
                                    pressed.0,
                                    hovering.get(rcx),
                                )
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
                                    .outline_width(2)
                                    .outline_offset(2);
                            } else {
                                sb.outline_color(colors::TRANSPARENT)
                                    .outline_width(0)
                                    .outline_offset(0);
                            }
                        },
                    );
                let children = self.children.as_ref();
                (children)(builder);
            });
    }
}

pub(crate) fn button_bg_color(
    variant: ButtonVariant,
    is_disabled: bool,
    is_pressed: bool,
    is_hovering: bool,
) -> Srgba {
    let base_color = match variant {
        ButtonVariant::Default => colors::U3,
        ButtonVariant::Primary => colors::PRIMARY,
        ButtonVariant::Danger => colors::DESTRUCTIVE,
        ButtonVariant::Selected => colors::U4,
    };
    // println!("Disabled: {}", is_disabled);
    match (is_disabled, is_pressed, is_hovering) {
        (true, _, _) => base_color.with_alpha(0.2),
        (_, true, true) => base_color.lighter(0.07),
        (_, false, true) => base_color.lighter(0.03),
        _ => base_color,
    }
}

pub(crate) fn button_on_key_event(
    mut trigger: Trigger<FocusKeyboardInput>,
    q_state: Query<(&ButtonState, Has<Disabled>)>,
    mut commands: Commands,
) {
    if let Ok((bstate, disabled)) = q_state.get(trigger.entity()) {
        if !disabled {
            let event = &trigger.event().0;
            if !event.repeat
                && (event.key_code == KeyCode::Enter || event.key_code == KeyCode::Space)
            {
                if let Some(on_click) = bstate.on_click {
                    trigger.propagate(false);
                    commands.run_callback(on_click, ());
                }
            }
        }
    }
}

pub(crate) fn button_on_pointer_click(
    mut trigger: Trigger<Pointer<Click>>,
    mut q_state: Query<(&ButtonState, &mut ButtonPressed, Has<Disabled>)>,
    mut commands: Commands,
) {
    if let Ok((bstate, pressed, disabled)) = q_state.get_mut(trigger.entity()) {
        trigger.propagate(false);
        if pressed.0 && !disabled {
            // println!("Click: {}", pressed.0);
            if let Some(on_click) = bstate.on_click {
                commands.run_callback(on_click, ());
            }
        }
    }
}

pub(crate) fn button_on_pointer_down(
    mut trigger: Trigger<Pointer<Down>>,
    mut q_state: Query<(&mut ButtonPressed, Has<Disabled>)>,
    mut focus: ResMut<KeyboardFocus>,
    mut focus_visible: ResMut<KeyboardFocusVisible>,
) {
    if let Ok((mut pressed, disabled)) = q_state.get_mut(trigger.entity()) {
        trigger.propagate(false);
        if !disabled {
            pressed.0 = true;
            focus.0 = Some(trigger.entity());
            focus_visible.0 = false;
        }
    }
}

pub(crate) fn button_on_pointer_up(
    mut trigger: Trigger<Pointer<Up>>,
    mut q_state: Query<(&mut ButtonPressed, Has<Disabled>)>,
) {
    if let Ok((mut pressed, disabled)) = q_state.get_mut(trigger.entity()) {
        trigger.propagate(false);
        if !disabled {
            pressed.0 = false;
        }
    }
}

pub(crate) fn button_on_pointer_drag_end(
    mut trigger: Trigger<Pointer<DragEnd>>,
    mut q_state: Query<(&mut ButtonPressed, Has<Disabled>)>,
) {
    if let Ok((mut pressed, disabled)) = q_state.get_mut(trigger.entity()) {
        trigger.propagate(false);
        if !disabled {
            pressed.0 = false;
        }
    }
}

pub(crate) fn button_on_pointer_cancel(
    mut trigger: Trigger<Pointer<Cancel>>,
    mut q_state: Query<(&mut ButtonPressed, Has<Disabled>)>,
) {
    if let Ok((mut pressed, disabled)) = q_state.get_mut(trigger.entity()) {
        trigger.propagate(false);
        if !disabled {
            pressed.0 = false;
        }
    }
}
