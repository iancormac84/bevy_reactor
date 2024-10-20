use std::sync::Arc;

use bevy::{ecs::world::DeferredWorld, prelude::*, ui};
use bevy_mod_stylebuilder::*;
use bevy_reactor_builder::{CreateChilden, EntityStyleBuilder, UiBuilder, UiTemplate};
use bevy_reactor_signals::Mutable;

use crate::scrolling::{ScrollArea, ScrollBar, ScrollBarThumb, ScrollContent, ScrollWheelEvent};

// Style definitions for scrollview widget.

// The combined scroll view with scrolling region and scrollbars.
fn style_scroll_view(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Grid)
        .grid_template_columns(vec![
            ui::RepeatedGridTrack::flex(1, 1.),
            ui::RepeatedGridTrack::auto(1),
        ])
        .grid_template_rows(vec![
            ui::RepeatedGridTrack::flex(1, 1.),
            ui::RepeatedGridTrack::auto(1),
        ])
        .gap(2);
}

/// The scrolling region which defines the clipping bounds.
fn style_scroll_region(ss: &mut StyleBuilder) {
    ss.grid_column(ui::GridPlacement::start_span(1, 1))
        .grid_row(ui::GridPlacement::start_span(1, 1))
        .overflow(ui::OverflowAxis::Clip);
}

/// The scrolling content which is clipped.
fn style_scroll_content(ss: &mut StyleBuilder) {
    ss.position(ui::PositionType::Absolute)
        .height(ui::Val::Auto)
        .min_width(ui::Val::Percent(100.))
        .border(1);
}

fn style_scrollbar_x(ss: &mut StyleBuilder) {
    ss.grid_column(ui::GridPlacement::start_span(1, 1))
        .grid_row(ui::GridPlacement::start_span(2, 1))
        .height(8);
}

fn style_scrollbar_x_thumb(ss: &mut StyleBuilder) {
    ss.background_color("#334")
        .position(ui::PositionType::Absolute)
        .top(1)
        .bottom(1)
        .border_radius(3);
    // .selector(":hover > &,.drag", |ss| ss.background_color("#556"));
}

fn style_scrollbar_y(ss: &mut StyleBuilder) {
    ss.grid_column(ui::GridPlacement::start_span(2, 1))
        .grid_row(ui::GridPlacement::start_span(1, 1))
        .width(8);
}

fn style_scrollbar_y_thumb(ss: &mut StyleBuilder) {
    ss.background_color("#334")
        .position(ui::PositionType::Absolute)
        .left(1)
        .right(1)
        .border_radius(3);
    // .selector(":hover > &,.drag", |ss| ss.background_color("#556"));
}

#[derive(Clone, PartialEq, Default, Copy)]
enum DragMode {
    #[default]
    None,
    DragX,
    DragY,
}

#[derive(Clone, PartialEq, Default, Copy)]
struct DragState {
    mode: DragMode,
    offset: f32,
}

/// The scroll view widget.
pub struct ScrollView {
    /// Views for the scrolling content
    pub children: Arc<dyn Fn(&mut UiBuilder)>,
    /// Style to be applied to the entire scroll view,
    pub style: StyleHandle,
    /// Style to be applied to the content region,
    pub content_style: StyleHandle,
    /// Whether to enable horizontal scrolling.
    pub scroll_enable_x: bool,
    /// Whether to enable vertical scrolling.
    pub scroll_enable_y: bool,
    /// Optional entity id to use for the scrolling element. This is useful for querying the
    /// current scroll position.
    pub entity: Option<Entity>,
}

impl Default for ScrollView {
    fn default() -> Self {
        Self {
            children: Arc::new(|_| {}),
            style: Default::default(),
            content_style: Default::default(),
            scroll_enable_x: Default::default(),
            scroll_enable_y: Default::default(),
            entity: Default::default(),
        }
    }
}

impl ScrollView {
    /// Create a new `ScrollView`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the child views for this element.
    pub fn children<V: 'static + Fn(&mut UiBuilder)>(mut self, children: V) -> Self {
        self.children = Arc::new(children);
        self
    }

    /// Set additional styles to be applied to the scroll view.
    pub fn style<S: StyleTuple + 'static>(mut self, style: S) -> Self {
        self.style = style.into_handle();
        self
    }

    /// Set additional styles to be applied to the scroll content.
    pub fn content_style<S: StyleTuple + 'static>(mut self, style: S) -> Self {
        self.content_style = style.into_handle();
        self
    }

    /// Enable horizontal scrolling.
    pub fn scroll_enable_x(mut self, enable: bool) -> Self {
        self.scroll_enable_x = enable;
        self
    }

    /// Enable vertical scrolling.
    pub fn scroll_enable_y(mut self, enable: bool) -> Self {
        self.scroll_enable_y = enable;
        self
    }

    /// Set the entity id to use for the scrolling element.
    /// This is useful for querying the current scroll position.
    pub fn entity(mut self, entity: Entity) -> Self {
        self.entity = Some(entity);
        self
    }
}

impl UiTemplate for ScrollView {
    fn build(&self, builder: &mut UiBuilder) {
        // A widget which displays a scrolling view of its children.
        let enable_x = self.scroll_enable_x;
        let enable_y = self.scroll_enable_y;
        let drag_state = builder.create_mutable::<DragState>(DragState::default());

        if let Some(entity) = self.entity {
            let mut e = builder.entity_mut(entity);
            e.insert(Name::new("ScrollView"));
            e
        } else {
            builder.spawn((Node::default(), Name::new("ScrollView")))
        }
        .styles((style_scroll_view, self.style.clone()))
        .create_children(|builder| {
            // Scroll area
            let id_scroll_area = builder
                .spawn((Node::default(), Name::new("ScrollView::ScrollArea")))
                .id();

            // Horizontal scroll bar
            let id_scrollbar_x = if enable_x {
                Some(build_scrollbar(builder, id_scroll_area, drag_state, false))
            } else {
                None
            };

            // Vertical scroll bar
            let id_scrollbar_y = if enable_y {
                Some(build_scrollbar(builder, id_scroll_area, drag_state, true))
            } else {
                None
            };

            builder
                .entity_mut(id_scroll_area)
                .insert((ScrollArea {
                    id_scrollbar_x,
                    id_scrollbar_y,
                    ..default()
                },))
                .style(style_scroll_region)
                .observe(
                    move |mut trigger: Trigger<ScrollWheelEvent>, mut world: DeferredWorld| {
                        trigger.propagate(false);
                        if let Some(mut scroll_area) = world.get_mut::<ScrollArea>(id_scroll_area) {
                            let event = &trigger.event().0;
                            match event.unit {
                                bevy::input::mouse::MouseScrollUnit::Line => {
                                    // TODO: Get inherited font size
                                    scroll_area.scroll_by(-event.x * 14., -event.y * 14.);
                                }
                                bevy::input::mouse::MouseScrollUnit::Pixel => {
                                    scroll_area.scroll_by(-event.x, -event.y);
                                }
                            }
                        }
                    },
                )
                .create_children(|builder| {
                    builder
                        .spawn((Node::default(), Name::new("ScrollView::ScrollRegion")))
                        .insert(ScrollContent)
                        .styles((style_scroll_content, self.content_style.clone()))
                        .create_children(|builder| {
                            (self.children.as_ref())(builder);
                        });
                });
        });
    }
}

fn build_scrollbar(
    builder: &mut UiBuilder,
    id_scroll_area: Entity,
    drag_state: Mutable<DragState>,
    vertical: bool,
) -> Entity {
    let scrollbar_id = builder
        .spawn((Node::default(), Name::new("Scrollbar")))
        .id();

    builder
        .entity_mut(scrollbar_id)
        .insert((ScrollBar {
            id_scroll_area,
            vertical,
            min_thumb_size: 10.,
        },))
        .style(if vertical {
            style_scrollbar_y
        } else {
            style_scrollbar_x
        })
        .observe(
            // Click outside of thumb
            move |mut trigger: Trigger<Pointer<Click>>,
                  mut world: DeferredWorld,
                  q_thumb: Query<(&ComputedNode, &GlobalTransform)>| {
                trigger.propagate(false);
                let id_scrollbar = trigger.entity();
                let Some(children) = world.get::<Children>(id_scrollbar) else {
                    return;
                };

                let Some(id_thumb) = children.iter().next() else {
                    return;
                };

                if let Ok((thumb, transform)) = q_thumb.get(*id_thumb) {
                    // Get thumb rectangle
                    let rect = Rect::from_center_size(transform.translation().xy(), thumb.size());
                    if let Some(mut scroll_area) = world.get_mut::<ScrollArea>(id_scroll_area) {
                        handle_track_click(
                            &mut scroll_area,
                            vertical,
                            trigger.event().pointer_location.position,
                            rect,
                        );
                    }
                }
            },
        )
        .create_children(|builder| {
            builder
                .spawn((Node::default(), Name::new("Scrollbar::Thumb")))
                .style(if vertical {
                    style_scrollbar_y_thumb
                } else {
                    style_scrollbar_x_thumb
                })
                .insert((ScrollBarThumb,));
        })
        .observe(
            move |mut trigger: Trigger<Pointer<DragStart>>, mut world: DeferredWorld| {
                trigger.propagate(false);
                if let Some(scroll_area) = world.get::<ScrollArea>(id_scroll_area) {
                    let scroll_top = scroll_area.scroll_top;
                    let scroll_left = scroll_area.scroll_left;
                    drag_state.set(
                        &mut world,
                        if vertical {
                            DragState {
                                mode: DragMode::DragY,
                                offset: scroll_top,
                            }
                        } else {
                            DragState {
                                mode: DragMode::DragX,
                                offset: scroll_left,
                            }
                        },
                    );
                }
            },
        )
        .observe(
            move |mut trigger: Trigger<Pointer<DragEnd>>, mut world: DeferredWorld| {
                trigger.propagate(false);
                drag_state.set(
                    &mut world,
                    DragState {
                        mode: DragMode::None,
                        offset: 0.,
                    },
                );
            },
        )
        .observe(
            move |mut trigger: Trigger<Pointer<Cancel>>, mut world: DeferredWorld| {
                trigger.propagate(false);
                drag_state.set(
                    &mut world,
                    DragState {
                        mode: DragMode::None,
                        offset: 0.,
                    },
                );
            },
        )
        .observe(
            move |mut trigger: Trigger<Pointer<Drag>>, mut world: DeferredWorld| {
                trigger.propagate(false);
                let ds = drag_state.get(&world);
                if let Some(mut scroll_area) = world.get_mut::<ScrollArea>(id_scroll_area) {
                    let distance = trigger.event().distance;
                    handle_thumb_drag(&mut scroll_area, &ds, distance);
                }
            },
        );

    scrollbar_id
}

fn handle_thumb_drag(scroll_area: &mut ScrollArea, ds: &DragState, distance: Vec2) {
    if ds.mode == DragMode::DragY {
        let left = scroll_area.scroll_left;
        let top = if scroll_area.visible_size.y > 0. {
            ds.offset + distance.y * scroll_area.content_size.y / scroll_area.visible_size.y
        } else {
            0.
        };
        scroll_area.scroll_to(left, top);
    } else if ds.mode == DragMode::DragX {
        let top = scroll_area.scroll_top;
        let left = if scroll_area.visible_size.x > 0. {
            ds.offset + distance.x * scroll_area.content_size.x / scroll_area.visible_size.x
        } else {
            0.
        };
        scroll_area.scroll_to(left, top);
    };
}

fn handle_track_click(scroll_area: &mut ScrollArea, vertical: bool, position: Vec2, rect: Rect) {
    if vertical {
        let page_size = scroll_area.visible_size.y;
        if position.y >= rect.max.y {
            scroll_area.scroll_by(0., page_size);
        } else if position.y < rect.min.y {
            scroll_area.scroll_by(0., -page_size);
        }
    } else {
        let page_size = scroll_area.visible_size.x;
        if position.x >= rect.max.x {
            scroll_area.scroll_by(page_size, 0.);
        } else if position.x < rect.min.x {
            scroll_area.scroll_by(-page_size, 0.);
        }
    }
}
