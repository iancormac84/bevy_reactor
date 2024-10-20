//! Example which uses states and a switch view.

use bevy::{color::palettes, prelude::*};
use bevy_mod_stylebuilder::*;
use bevy_reactor_builder::{
    CreateChilden, EntityStyleBuilder, InvokeUiTemplate, SwitchBuilder, TextBuilder, UiBuilder,
    UiTemplate,
};
use bevy_reactor_signals::{Rcx, SignalsPlugin};

fn style_test(ss: &mut StyleBuilder) {
    ss.display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .border(3)
        .padding(3);
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Play,
    Pause,
    Intro,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .insert_state(GameState::Intro)
        .add_plugins((SignalsPlugin, StyleBuilderPlugin))
        .add_systems(Startup, setup_view_root)
        .add_systems(Update, (close_on_esc, handle_key_input))
        .run();
}

fn setup_view_root(world: &mut World) {
    let camera = world.spawn((Camera::default(), Camera2d)).id();

    world
        .spawn(Node::default())
        .style(style_test)
        .insert(TargetCamera(camera))
        .insert(BorderColor(palettes::css::LIME.into()))
        .create_children(|builder| {
            builder.text("State: ");
            builder.invoke(StateName);
        });
}

struct StateName;

impl UiTemplate for StateName {
    fn build(&self, builder: &mut UiBuilder) {
        builder.switch(
            |rcx: &Rcx| rcx.read_resource::<State<GameState>>().get().clone(),
            |builder| {
                builder
                    .case(GameState::Intro, |builder| {
                        builder.text("Intro");
                    })
                    .case(GameState::Pause, |builder| {
                        builder.text("Pause");
                    })
                    .fallback(|builder| {
                        builder.text("Play");
                    });
            },
        );
    }
}

fn handle_key_input(
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    if key.just_pressed(KeyCode::Space) {
        match state.get() {
            GameState::Intro => next_state.set(GameState::Play),
            GameState::Play => next_state.set(GameState::Pause),
            GameState::Pause => next_state.set(GameState::Play),
        }
    }
}

pub fn close_on_esc(input: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if input.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
}
