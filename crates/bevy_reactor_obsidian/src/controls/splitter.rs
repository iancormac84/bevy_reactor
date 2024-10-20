use bevy::{
    color::Luminance, ecs::world::DeferredWorld, prelude::*, ui, window::SystemCursorIcon,
    winit::cursor::CursorIcon,
};
use bevy_mod_stylebuilder::*;
use bevy_reactor_builder::{CreateChilden, EntityStyleBuilder, UiBuilder, UiTemplate};
use bevy_reactor_signals::{Callback, IntoSignal, RunCallback, Signal};

use crate::{colors, cursor::StyleBuilderCursor, hover_signal::CreateHoverSignal};

/// The direction of the splitter. Represents the direction of the bar, not the items being split.
#[derive(Clone, PartialEq, Default)]
pub enum SplitterDirection {
    /// The splitter bar runs horizontally, and splits the items above and below it.
    Horizontal,

    /// The splitter bar runs horizontally, and splits the items above and below it; however
    /// dragging is inverted. Used for panels on the bottom.
    HorizontalReverse,

    /// The splitter bar runs vertically, and splits the items to the left and right of it.
    #[default]
    Vertical,

    /// The splitter bar runs vertically, and splits the items to the left and right of it.
    /// However, dragging is inverted. Used for panels on the right.
    VerticalReverse,
}

#[derive(Clone, PartialEq, Default, Copy)]
struct DragState {
    dragging: bool,
    offset: f32,
}

fn style_vsplitter(ss: &mut StyleBuilder) {
    ss.align_items(ui::AlignItems::Center)
        .justify_content(ui::JustifyContent::Center)
        .display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Column)
        .gap(8)
        .width(9)
        .background_color(colors::U2)
        .cursor(CursorIcon::System(SystemCursorIcon::ColResize));
}

// The decorative handle inside the splitter.
fn style_vsplitter_inner(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .width(3)
        // .pointer_events(PointerEvents::None)
        .height(ui::Val::Percent(20.));
}

fn style_hsplitter(ss: &mut StyleBuilder) {
    ss.align_items(ui::AlignItems::Center)
        .justify_content(ui::JustifyContent::Center)
        .display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Column)
        .gap(8)
        .height(9)
        .background_color(colors::U2)
        .cursor(CursorIcon::System(SystemCursorIcon::RowResize));
}

// The decorative handle inside the splitter.
fn style_hsplitter_inner(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .height(3)
        // .pointer_events(PointerEvents::None)
        .width(ui::Val::Percent(20.));
}

/// Splitter bar which can be dragged
pub struct Splitter {
    /// The current split value.
    pub value: Signal<f32>,

    /// Whether the splitter bar runs horizontally or vertically.
    pub direction: SplitterDirection,

    /// Callback involved with the new split value.
    pub on_change: Option<Callback<f32>>,
}

impl Splitter {
    /// Create a new splitter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the current split value.
    pub fn value(mut self, value: impl IntoSignal<f32>) -> Self {
        self.value = value.into_signal();
        self
    }

    /// Set the direction of the splitter.
    pub fn direction(mut self, direction: SplitterDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Set the callback to be invoked when the split value changes.
    pub fn on_change(mut self, on_change: Callback<f32>) -> Self {
        self.on_change = Some(on_change);
        self
    }
}

impl Default for Splitter {
    fn default() -> Self {
        Self {
            value: Signal::Constant(0.),
            direction: SplitterDirection::Vertical,
            on_change: None,
        }
    }
}

impl UiTemplate for Splitter {
    fn build(&self, builder: &mut UiBuilder) {
        let id = builder
            .spawn((Node::default(), Name::new("Splitter")))
            .id();
        let hovering = builder.create_hover_signal(id);
        let drag_state = builder.create_mutable::<DragState>(DragState::default());
        let on_change = self.on_change;
        let current_offset = self.value;
        let direction = self.direction.clone();
        let style_splitter = match self.direction {
            SplitterDirection::Horizontal | SplitterDirection::HorizontalReverse => style_hsplitter,
            SplitterDirection::Vertical | SplitterDirection::VerticalReverse => style_vsplitter,
        };
        let style_splitter_inner = match self.direction {
            SplitterDirection::Horizontal | SplitterDirection::HorizontalReverse => {
                style_hsplitter_inner
            }
            SplitterDirection::Vertical | SplitterDirection::VerticalReverse => {
                style_vsplitter_inner
            }
        };

        builder
            .entity_mut(id)
            .style(style_splitter)
            .observe(
                move |mut trigger: Trigger<Pointer<DragStart>>, mut world: DeferredWorld| {
                    // Save initial value to use as drag offset.
                    trigger.propagate(false);
                    let offset = current_offset.get(&world);
                    drag_state.set(
                        &mut world,
                        DragState {
                            dragging: true,
                            offset,
                        },
                    );
                },
            )
            .observe(
                move |mut trigger: Trigger<Pointer<DragEnd>>, mut world: DeferredWorld| {
                    trigger.propagate(false);
                    let offset = current_offset.get(&world);
                    drag_state.set(
                        &mut world,
                        DragState {
                            dragging: false,
                            offset,
                        },
                    );
                },
            )
            .observe(
                move |mut trigger: Trigger<Pointer<Cancel>>, mut world: DeferredWorld| {
                    trigger.propagate(false);
                    let offset = current_offset.get(&world);
                    drag_state.set(
                        &mut world,
                        DragState {
                            dragging: false,
                            offset,
                        },
                    );
                },
            )
            .observe(
                move |mut trigger: Trigger<Pointer<Drag>>, mut world: DeferredWorld| {
                    trigger.propagate(false);
                    let event = trigger.event();
                    let ev = event.distance;
                    let ds = drag_state.get(&world);
                    if let Some(on_change) = on_change {
                        if ds.dragging {
                            match direction {
                                SplitterDirection::Horizontal => {
                                    world.run_callback(on_change, ds.offset - ev.y);
                                }
                                SplitterDirection::HorizontalReverse => {
                                    world.run_callback(on_change, ds.offset + ev.y);
                                }
                                SplitterDirection::Vertical => {
                                    world.run_callback(on_change, ev.x + ds.offset);
                                }
                                SplitterDirection::VerticalReverse => {
                                    world.run_callback(on_change, ds.offset - ev.x);
                                }
                            }
                        }
                    }
                },
            )
            .create_children(|builder| {
                builder
                    .spawn(Node::default())
                    .style(style_splitter_inner)
                    .style_dyn(
                        move |rcx| {
                            // Color change on hover / drag
                            let ds = drag_state.get(rcx);
                            let is_hovering = hovering.get(rcx);
                            match (ds.dragging, is_hovering) {
                                (true, _) => colors::U3.lighter(0.05),
                                (false, true) => colors::U3.lighter(0.02),
                                (false, false) => colors::U3,
                            }
                        },
                        |color, sb| {
                            sb.background_color(color);
                        },
                    );
            });
    }
}
