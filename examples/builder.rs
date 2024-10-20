//! Example of a simple UI layout

use std::f32::consts::PI;

use bevy::{
    color::palettes,
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};
use bevy_mod_stylebuilder::*;
use bevy_reactor_builder::*;
use bevy_reactor_signals::{Callback, Rcx, RunCallback, SignalsPlugin};

fn style_test(ss: &mut StyleBuilder) {
    ss.display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .border(3)
        .border_color(palettes::css::ALICE_BLUE)
        .padding(3);
}

fn main() {
    App::new()
        .init_resource::<Counter>()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((SignalsPlugin, StyleBuilderPlugin))
        .add_systems(Startup, (setup, setup_view_root))
        .add_systems(Update, (close_on_esc, rotate, update_counter))
        .run();
}

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
struct Shape;

const X_EXTENT: f32 = 14.5;

fn setup_view_root(world: &mut World) {
    world
        .spawn(Node::default())
        .styles(style_test)
        .style_dyn(
            |rcx| {
                let counter = rcx.read_resource::<Counter>();
                counter.count & 1 == 0
            },
            |even, sb| {
                sb.border_color(if even {
                    palettes::css::LIME
                } else {
                    palettes::css::MAROON
                });
            },
        )
        .create_children(|builder| {
            let on_click = builder.create_callback(|_in: In<()>| {
                println!("Clicked!");
            });
            builder
                .text("Count: ")
                .invoke(NestedView)
                .cond(
                    |rcx: &Rcx| {
                        let counter = rcx.read_resource::<Counter>();
                        counter.count & 1 == 0
                    },
                    |builder| {
                        builder.text("[Even]");
                    },
                    |builder| {
                        builder.text("[Odd]");
                    },
                )
                .invoke(Clickable { on_click });
        });
}

struct NestedView;

impl UiTemplate for NestedView {
    fn build(&self, builder: &mut UiBuilder) {
        builder.text_computed(|rcx| {
            let counter = rcx.read_resource::<Counter>();
            format!("{}", counter.count)
        });
    }
}

struct Clickable {
    on_click: Callback,
}

impl UiTemplate for Clickable {
    fn build(&self, builder: &mut UiBuilder) {
        let on_click = self.on_click;
        builder
            .spawn(Node::default())
            .observe(
                move |_event: Trigger<Pointer<Click>>, mut commands: Commands| {
                    commands.run_callback(on_click, ());
                },
            )
            .create_children(|builder| {
                builder.text("Click Me!");
            });
    }
}

#[derive(Resource, Default)]
pub struct Counter {
    pub count: u32,
}

fn update_counter(mut counter: ResMut<Counter>, key: Res<ButtonInput<KeyCode>>) {
    if key.pressed(KeyCode::Space) {
        counter.count += 1;
    }
}

// Setup 3d shapes
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    let shapes = [
        meshes.add(Cuboid::default()),
        meshes.add(Capsule3d::default()),
        meshes.add(Torus::default()),
        meshes.add(Cylinder::default()),
        meshes.add(Sphere::default().mesh().ico(5).unwrap()),
        meshes.add(Sphere::default().mesh().uv(32, 18)),
    ];

    let num_shapes = shapes.len();

    for (i, shape) in shapes.into_iter().enumerate() {
        commands.spawn((
            Mesh3d(shape),
            MeshMaterial3d(debug_material.clone()),
            Transform::from_xyz(
                -X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * X_EXTENT,
                2.0,
                0.0,
            )
            .with_rotation(Quat::from_rotation_x(-PI / 4.)),
            Shape,
        ));
    }

    commands.spawn((
        PointLight {
            // intensity: 9000.0,
            intensity: 10000000.0,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(8.0, 16.0, 8.0),
    ));

    // ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0))),
        MeshMaterial3d(materials.add(Color::from(palettes::css::SILVER))),
    ));

    commands.spawn((
        Camera::default(),
        Camera3d::default(),
        Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
    ));
}

fn rotate(mut query: Query<&mut Transform, With<Shape>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs() / 2.);
    }
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

pub fn close_on_esc(input: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if input.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
}
