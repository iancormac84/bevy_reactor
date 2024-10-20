//! Example of a simple UI layout

use bevy::{prelude::*, ui};
use bevy_mod_stylebuilder::*;
use bevy_reactor_builder::*;
use bevy_reactor_obsidian::{input_dispatch::DefaultKeyHandler, prelude::*};
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

fn main() {
    App::new()
        .init_resource::<List>()
        .init_resource::<Random32>()
        .add_plugins((
            DefaultPlugins,
            SignalsPlugin,
            StyleBuilderPlugin,
            ObsidianUiPlugin,
        ))
        .add_systems(Startup, setup_view_root)
        .add_systems(Update, (update_list, close_on_esc))
        .run();
}

const SUITS: &[&str] = &["hearts", "spades", "clubs", "diamonds"];

#[derive(Resource, Default)]
pub struct List {
    pub items: Vec<String>,
}

fn setup_view_root(world: &mut World) {
    let camera = world.spawn((Camera::default(), Camera2d)).id();

    world
        .spawn(Node::default())
        .insert((TargetCamera(camera), TabGroup::default(), DefaultKeyHandler))
        .observe(handle_tab_navigation)
        .style(style_test)
        .create_children(|builder| {
            // builder.spawn(GhostNode).create_children(|builder| {
            //     builder.spawn(GhostNode).create_children(|builder| {
            //         builder.text("One");
            //     });
            //     builder.spawn(GhostNode).create_children(|builder| {
            //         builder.text("Two");
            //     });
            //     builder.spawn(GhostNode).create_children(|builder| {
            //         builder.text("Three");
            //     });
            // });
            // builder.for_index(
            //     |_| vec!["One", "Two", "Three"].into_iter(),
            //     |item, _, builder| {
            //         builder.text(item.to_owned());
            //     },
            //     |_| {},
            // );
            builder.for_each(
                |rcx| {
                    let suits = rcx.read_resource::<List>();
                    suits.items.clone().into_iter()
                },
                |item, builder| {
                    builder.text(item.clone());
                },
                |_| {},
            );
        });
}

fn update_list(
    mut list: ResMut<List>,
    key: Res<ButtonInput<KeyCode>>,
    mut random: ResMut<Random32>,
) {
    if key.just_pressed(KeyCode::Space) {
        println!("-- Space pressed --");
        let i = (random.next() as usize) % SUITS.len();
        list.items.push(SUITS[i].to_string());
        while list.items.len() > 10 {
            list.items.remove(0);
        }
    } else if key.just_pressed(KeyCode::Minus) {
        println!("-- Minus pressed --");
        list.items.pop();
    }
}

pub fn close_on_esc(input: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if input.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
}

#[derive(Resource)]
struct Random32 {
    state: u32,
}

impl Random32 {
    // Generate a pseudo-random number
    fn next(&mut self) -> u32 {
        // Constants for 32-bit LCG (example values, you might want to choose different ones)
        let a: u32 = 1664525; // Multiplier
        let c: u32 = 1013904223; // Increment
        let m: u32 = 2u32.pow(31); // Modulus, often set to 2^31 for a 32-bit generator

        // Simple LCG formula: X_{n+1} = (aX_n + c) mod m
        self.state = (a.wrapping_mul(self.state).wrapping_add(c)) % m;
        self.state
    }
}

impl Default for Random32 {
    fn default() -> Self {
        Self { state: 17 }
    }
}
