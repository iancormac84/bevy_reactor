use std::sync::Arc;

use bevy::{
    color::{Alpha, Luminance},
    prelude::*,
    ui,
};
use bevy_mod_stylebuilder::*;
use bevy_reactor_builder::{
    CondBuilder, CreateChilden, EntityEffectBuilder, EntityStyleBuilder, UiBuilder, UiTemplate,
};
use bevy_reactor_signals::{Callback, RunCallback, Signal};

use crate::{
    animation::{
        AnimatedBackgroundColor, AnimatedScale, AnimatedTransition, BistableTransitionState,
        CreateBistableTransition,
    },
    colors,
    prelude::TabGroup,
    typography::text_default,
};

use super::barrier::Barrier;

// Dialog background overlay
fn style_dialog_barrier(ss: &mut StyleBuilder) {
    ss.position(PositionType::Absolute)
        .display(ui::Display::Flex)
        .justify_content(ui::JustifyContent::Center)
        .align_items(ui::AlignItems::Center)
        .left(0)
        .top(0)
        .width(ui::Val::Vw(100.))
        .height(ui::Val::Vh(100.))
        .border(1)
        .border_color(colors::ANIMATION)
        .z_index(100)
        .background_color(colors::U2.with_alpha(0.0));
}

fn style_dialog(ss: &mut StyleBuilder) {
    ss.background_color(colors::U2)
        .border_radius(6.0)
        .position(PositionType::Relative)
        .display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Column)
        .justify_content(ui::JustifyContent::Center)
        .align_items(ui::AlignItems::Stretch)
        .border_color(colors::U1)
        .width(400)
        .border(3);
    // .scale(0.5)
    // .transition(&[Transition {
    //     property: TransitionProperty::Transform,
    //     duration: 0.3,
    //     timing: timing::EASE_IN_OUT,
    //     ..default()
    // }])
    // .selector(".entering > &,.entered > &", |ss| ss.scale(1.));
}

const TRANSITION_DURATION: f32 = 0.3;

/// Displays a modal dialog box. This will display the dialog frame and the backdrop overlay.
/// Use the dialog header/body/footer controls to get the standard layout.
pub struct Dialog {
    /// The width of the dialog, one of several standard widths.
    pub width: ui::Val,

    /// Signal that controls whether the dialog is open. Note that when this becomes false,
    /// the dialog will still remain visible until it completes its closing animation.
    pub open: Signal<bool>,

    /// The content of the dialog.
    pub children: Arc<dyn Fn(&mut UiBuilder) + Send + Sync + 'static>,

    /// Callback called when the dialog's close button is clicked.
    pub on_close: Option<Callback>,

    /// Callback called when the dialog has completed it's closing animation.
    pub on_exited: Option<Callback>,
}

impl Default for Dialog {
    fn default() -> Self {
        Self {
            width: ui::Val::Px(400.0),
            open: Signal::Constant(false),
            children: Arc::new(|_| {}),
            on_close: None,
            on_exited: None,
        }
    }
}

impl Dialog {
    /// Creates a new `Dialog`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the width of the dialog.
    pub fn width(mut self, width: ui::Val) -> Self {
        self.width = width;
        self
    }

    /// Sets the signal that controls whether the dialog is open.
    pub fn open(mut self, open: Signal<bool>) -> Self {
        self.open = open;
        self
    }

    /// Sets the content of the dialog.
    pub fn children<V: 'static + Send + Sync + Fn(&mut UiBuilder)>(mut self, children: V) -> Self {
        self.children = Arc::new(children);
        self
    }

    /// Sets the callback called when the dialog's close button is clicked.
    pub fn on_close(mut self, on_close: Callback) -> Self {
        self.on_close = Some(on_close);
        self
    }

    /// Sets the callback called when the dialog has completed it's closing animation.
    pub fn on_exited(mut self, on_exited: Callback) -> Self {
        self.on_exited = Some(on_exited);
        self
    }
}

impl UiTemplate for Dialog {
    fn build(&self, builder: &mut bevy_reactor_builder::UiBuilder) {
        let on_close = self.on_close;
        let on_exited = self.on_exited;
        let state = builder.create_bistable_transition(self.open, TRANSITION_DURATION);
        let width = self.width;

        builder.create_effect(move |ve| {
            let state = state.get(ve);
            if state == BistableTransitionState::Exited {
                if let Some(on_exited) = on_exited {
                    ve.run_callback(on_exited, ());
                }
            }
        });

        let is_shown =
            builder.create_derived(move |rcx| state.get(rcx) != BistableTransitionState::Exited);

        let children = self.children.clone();
        builder.cond(
            is_shown,
            move |builder| {
                let children = children.clone();
                // Portal::new(
                builder
                    .spawn((Node::default(), Name::new("Dialog::Overlay")))
                    .style(style_dialog_barrier)
                    .insert(Barrier { on_close })
                    .effect(
                        move |rcx| {
                            let state = state.get(rcx);
                            match state {
                                BistableTransitionState::Entering
                                | BistableTransitionState::Entered => colors::U2.with_alpha(0.7),
                                BistableTransitionState::Exiting
                                | BistableTransitionState::Exited => colors::U2.with_alpha(0.0),
                            }
                        },
                        move |color, ent| {
                            AnimatedTransition::<AnimatedBackgroundColor>::start(
                                ent,
                                color,
                                None,
                                TRANSITION_DURATION,
                            );
                        },
                    )
                    .create_children(|builder| {
                        builder
                            .spawn((Node::default(), Name::new("Dialog")))
                            .insert(TabGroup {
                                order: 0,
                                modal: true,
                            })
                            .observe(|mut trigger: Trigger<Pointer<Down>>| {
                                // Prevent clicks from propagating to the barrier and closing
                                // the dialog.
                                trigger.propagate(false);
                            })
                            .styles((text_default, style_dialog, move |ss: &mut StyleBuilder| {
                                ss.width(width);
                            }))
                            .effect(
                                move |rcx| {
                                    let state = state.get(rcx);
                                    match state {
                                        BistableTransitionState::Entering => (0.0, 1.0),
                                        BistableTransitionState::Exiting => (1.0, 0.0),
                                        BistableTransitionState::Entered => (1.0, 1.0),
                                        BistableTransitionState::Exited => (0.0, 0.0),
                                    }
                                },
                                move |(origin, target), ent| {
                                    AnimatedTransition::<AnimatedScale>::start(
                                        ent,
                                        Vec3::splat(target),
                                        Some(Vec3::splat(origin)),
                                        TRANSITION_DURATION,
                                    );
                                },
                            )
                            .create_children(|builder| {
                                (children.as_ref())(builder);
                            });
                        // ;
                    });
                // );
            },
            |_| {},
        );
    }
}

fn style_dialog_header(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .justify_content(ui::JustifyContent::SpaceBetween)
        .font_size(18)
        .border_color(colors::U2.darker(0.01))
        .border_bottom(1)
        .padding((12, 6));
}

/// Displays a standard dialog header.
#[derive(Clone)]
pub struct DialogHeader {
    /// The content of the dialog header.
    pub children: Arc<dyn Fn(&mut UiBuilder)>,
}

impl Default for DialogHeader {
    fn default() -> Self {
        Self {
            children: Arc::new(|_| {}),
        }
    }
}

impl DialogHeader {
    /// Create a new dialog header.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the content of the dialog header.
    pub fn children<V: 'static + Fn(&mut UiBuilder)>(mut self, children: V) -> Self {
        self.children = Arc::new(children);
        self
    }
}

impl UiTemplate for DialogHeader {
    fn build(&self, builder: &mut bevy_reactor_builder::UiBuilder) {
        builder
            .spawn(Node::default())
            .style(style_dialog_header)
            .create_children(|builder| {
                (self.children.as_ref())(builder);
            });
    }
}

fn style_dialog_body(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Column)
        .align_items(ui::AlignItems::Stretch)
        .justify_content(ui::JustifyContent::FlexStart)
        .padding((12, 6))
        .min_height(200);
}

/// Displays a standard dialog body.
#[derive(Clone)]
pub struct DialogBody {
    /// The content of the dialog header.
    pub children: Arc<dyn Fn(&mut UiBuilder)>,
}

impl Default for DialogBody {
    fn default() -> Self {
        Self {
            children: Arc::new(|_| {}),
        }
    }
}

impl DialogBody {
    /// Create a new dialog body.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the content of the dialog body.
    pub fn children<V: 'static + Fn(&mut UiBuilder)>(mut self, children: V) -> Self {
        self.children = Arc::new(children);
        self
    }
}

impl UiTemplate for DialogBody {
    fn build(&self, builder: &mut bevy_reactor_builder::UiBuilder) {
        builder
            .spawn(Node::default())
            .style(style_dialog_body)
            .create_children(|builder| {
                (self.children.as_ref())(builder);
            });
    }
}

fn style_dialog_footer(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .justify_content(ui::JustifyContent::FlexEnd)
        .align_items(ui::AlignItems::Center)
        .border_color(colors::U2.darker(0.01))
        .border_top(1)
        .column_gap(4)
        .padding((8, 6));
}

/// Displays a standard dialog footer.
#[derive(Clone)]
pub struct DialogFooter {
    /// The content of the dialog header.
    pub children: Arc<dyn Fn(&mut UiBuilder)>,
}

impl Default for DialogFooter {
    fn default() -> Self {
        Self {
            children: Arc::new(|_| {}),
        }
    }
}

impl DialogFooter {
    /// Create a new dialog footer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the content of the dialog footer.
    pub fn children<V: 'static + Fn(&mut UiBuilder)>(mut self, children: V) -> Self {
        self.children = Arc::new(children);
        self
    }
}

impl UiTemplate for DialogFooter {
    fn build(&self, builder: &mut bevy_reactor_builder::UiBuilder) {
        builder
            .spawn(Node::default())
            .style(style_dialog_footer)
            .create_children(|builder| {
                (self.children.as_ref())(builder);
            });
    }
}
