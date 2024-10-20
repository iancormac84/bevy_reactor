//! Example of a simple UI layout

use bevy::{color::palettes, ecs::world::DeferredWorld, prelude::*, ui};
use bevy_mod_stylebuilder::*;
use bevy_reactor_builder::{CreateChilden, EntityStyleBuilder, InvokeUiTemplate, TextBuilder};
use bevy_reactor_obsidian::{
    animation::{BistableTransitionState, CreateBistableTransition},
    controls::Button,
    input_dispatch::DefaultKeyHandler,
    prelude::*,
};
use bevy_reactor_signals::SignalsPlugin;

fn style_test(ss: &mut StyleBuilder) {
    ss.display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .position(ui::PositionType::Absolute)
        .padding(3)
        .left(0)
        .right(0)
        .top(0)
        .bottom(0)
        .row_gap(4)
        .background_color(colors::BACKGROUND);
}

fn style_row(ss: &mut StyleBuilder) {
    ss.display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .align_items(ui::AlignItems::Center)
        .column_gap(4);
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            SignalsPlugin,
            StyleBuilderPlugin,
            ObsidianUiPlugin,
        ))
        .add_systems(Startup, setup_view_root)
        .add_systems(Update, (change_text_color, close_on_esc))
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
            builder.text("bistable_transition");
            let row = builder.spawn(Node::default());
            let row_id = row.id();
            let is_hover = builder.create_hover_signal(row_id);
            let transition_state = builder.create_bistable_transition(is_hover, 0.3);
            let color = builder.create_derived(move |rcx| match transition_state.get(rcx) {
                BistableTransitionState::Entering => palettes::css::GREEN,
                BistableTransitionState::Entered => palettes::css::YELLOW,
                BistableTransitionState::Exiting => palettes::css::ORANGE,
                BistableTransitionState::Exited => palettes::css::GRAY,
            });
            builder
                .entity_mut(row_id)
                .style(style_row)
                .create_children(|builder| {
                    builder.invoke(Swatch::new(color).style(|sb: &mut StyleBuilder| {
                        sb.width(64);
                    }));
                });

            builder.text("DisclosureToggle");
            builder
                .spawn(Node::default())
                .create_children(|builder| {
                    let expanded = builder.create_mutable(false);
                    let on_change = builder.create_callback(
                        move |value: In<bool>, mut world: DeferredWorld| {
                            expanded.set(&mut world, *value);
                        },
                    );
                    builder.invoke(
                        DisclosureToggle::new()
                            .expanded(expanded)
                            .on_change(on_change),
                    );
                });

            builder.text("Dialog");
            builder
                .spawn(Node::default())
                .create_children(|builder| {
                    let open = builder.create_mutable(false);
                    let on_open =
                        builder.create_callback(move |_: In<()>, mut world: DeferredWorld| {
                            open.set(&mut world, true);
                        });
                    let on_close =
                        builder.create_callback(move |_: In<()>, mut world: DeferredWorld| {
                            open.set(&mut world, false);
                        });
                    builder.invoke(Button::new().labeled("Open").on_click(on_open));
                    builder.invoke(
                        Dialog::new()
                            .open(open.signal())
                            .on_close(on_close)
                            .children(move |builder| {
                                builder.invoke(DialogHeader::new().children(|builder| {
                                    builder.text("Dialog Header");
                                }));
                                builder.invoke(DialogBody::new().children(|builder| {
                                    builder.text("Dialog Body");
                                }));
                                builder.invoke(DialogFooter::new().children(move |builder| {
                                    builder
                                        .invoke(Button::new().labeled("Close").on_click(on_close));
                                }));
                            }),
                    );
                });

            builder.text("Text");
            builder
                .spawn((TextLayout::default(), Text::default()))
                .styles((typography::text_default, |sb: &mut StyleBuilder| {
                    sb.font_size(32).color(palettes::css::GRAY);
                }))
                .create_children(|builder| {
                    builder.spawn((
                        Text("The quick brown fox jumps over the ".to_string()),
                        TextColor::default(),
                        UseInheritedTextStyles,
                    ));
                    builder.spawn((
                        Text("lazy".to_string()),
                        TextColor::default(),
                        UseInheritedTextStyles,
                        AnimateTextColor { hue: 0. },
                    ));
                    builder.spawn((
                        Text(" dog".to_string()),
                        TextColor::default(),
                        UseInheritedTextStyles,
                    ));
                });
        });
}

#[derive(Component)]
struct AnimateTextColor {
    hue: f32,
}

fn change_text_color(mut q_text: Query<(&mut TextColor, &mut AnimateTextColor)>, time: Res<Time>) {
    for (mut text_style, mut animate) in q_text.iter_mut() {
        animate.hue = (animate.hue + time.delta_secs() * 200.).rem_euclid(360.0);
        text_style.0 = Hsla::new(animate.hue, 1., 0.5, 1.).into();
    }
}

pub fn close_on_esc(input: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if input.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
}
