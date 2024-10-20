//! Example of a simple UI layout

use bevy::{ecs::world::DeferredWorld, prelude::*, ui};
use bevy_mod_stylebuilder::*;
use bevy_reactor_builder::{CreateChilden, EntityStyleBuilder, InvokeUiTemplate, TextBuilder};
use bevy_reactor_obsidian::{
    input_dispatch::DefaultKeyHandler, prelude::*, tab_navigation::TabGroup,
};
use bevy_reactor_signals::SignalsPlugin;

fn style_test(ss: &mut StyleBuilder) {
    ss.display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .align_items(ui::AlignItems::Stretch)
        .position(ui::PositionType::Absolute)
        .left(0)
        .right(0)
        .top(0)
        .bottom(0)
        .row_gap(4)
        .background_color(colors::BACKGROUND);
}

fn style_panel(ss: &mut StyleBuilder) {
    ss.display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .align_items(ui::AlignItems::Stretch)
        .column_gap(4);
}

#[derive(Resource)]
pub struct LeftPanelWidth(f32);

#[derive(Resource)]
pub struct RightPanelWidth(f32);

fn main() {
    App::new()
        .insert_resource(LeftPanelWidth(200.0))
        .insert_resource(RightPanelWidth(200.0))
        .add_plugins((
            DefaultPlugins,
            SignalsPlugin,
            StyleBuilderPlugin,
            ObsidianUiPlugin,
        ))
        .add_systems(Startup, setup_view_root)
        .add_systems(Update, close_on_esc)
        .run();
}

fn setup_view_root(world: &mut World) {
    let camera = world.spawn((Camera::default(), Camera2d)).id();

    world
        .spawn(Node::default())
        .insert((TargetCamera(camera), TabGroup::default(), DefaultKeyHandler))
        .observe(handle_tab_navigation)
        .style(style_test)
        .create_children(|builder| {
            let left_width = builder.create_derived(|rcx| rcx.read_resource::<LeftPanelWidth>().0);
            let on_resize_left =
                builder.create_callback(|value: In<f32>, mut world: DeferredWorld| {
                    world.resource_mut::<LeftPanelWidth>().0 = value.max(100.);
                });
            let right_width =
                builder.create_derived(|rcx| rcx.read_resource::<RightPanelWidth>().0);
            let on_resize_right =
                builder.create_callback(|value: In<f32>, mut world: DeferredWorld| {
                    world.resource_mut::<RightPanelWidth>().0 = value.max(100.);
                });

            let dummy_text = "The quick, brown fox jumps over a lazy dog. DJs flock by when MTV ax quiz prog. Junk MTV quiz graced by fox whelps. Bawds jog, flick quartz, vex nymphs. Waltz, bad nymph, for quick jigs vex! Fox nymphs grab quick-jived waltz.";

            builder
                .spawn(Node::default())
                .style(style_panel)
                .style_dyn(
                    |rcx| rcx.read_resource::<LeftPanelWidth>().0,
                    |width, sb| {
                        sb.width(width);
                    },
                )
                .create_children(|builder| {
                    builder.invoke(
                        ScrollView::new()
                            .style(|sb: &mut StyleBuilder| {
                                sb.flex_grow(1.);
                            })
                            .scroll_enable_x(true)
                            .scroll_enable_y(true)
                            .children(|builder| {
                                builder.text(dummy_text.to_owned());
                            }),
                    );
                });

            builder.invoke(Splitter::new().value(left_width).on_change(on_resize_left));

            builder
                .spawn(Node::default())
                .style(style_panel)
                .style(|sb| {
                    sb.flex_grow(1.);
                })
                .create_children(|builder| {
                    builder.text("Middle");
                });

            builder.invoke(
                Splitter::new()
                    .direction(SplitterDirection::VerticalReverse)
                    .value(right_width)
                    .on_change(on_resize_right),
            );

            builder
                .spawn(Node::default())
                .style(style_panel)
                .style_dyn(
                    |rcx| rcx.read_resource::<RightPanelWidth>().0,
                    |width, sb| {
                        sb.width(width);
                    },
                )
                .create_children(|builder| {
                    builder.text("Right");
                });
        });
}

pub fn close_on_esc(input: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if input.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
}
