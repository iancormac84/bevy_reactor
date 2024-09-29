//! Example of a simple UI layout
// mod node_graph_demo;
// mod reflect_demo;
// mod transform_overlay;

use bevy_mod_stylebuilder::*;
use bevy_reactor_obsidian::{cursor::StyleBuilderCursor, prelude::*};
// use bevy_reactor_overlays as overlays;
use bevy_reactor_signals::{Cx, Rcx, RunContextRead, RunContextSetup, TrackingScopeTracing};
// use node_graph_demo::{DemoGraphRoot, NodeGraphDemo};
// use obsidian_ui::{
//     colors,
//     controls::{
//         Button, ButtonVariant, Checkbox, Dialog, DialogFooter, DialogHeader, ListView, Slider,
//         Splitter, SplitterDirection, TextInput, TextInputProps, ToolButton, ToolPalette,
//     },
//     cursor::StyleBuilderCursor,
//     focus::TabGroup,
//     typography, viewport, ObsidianUiPlugin, RoundedCorners,
// };
// use obsidian_ui_inspect::InspectorPlugin;
// use reflect_demo::{ResourcePropertyInspector, TestStruct, TestStruct2, TestStruct3};
// use transform_overlay::TransformOverlay;

use std::f32::consts::PI;

use bevy::{
    asset::io::{file::FileAssetReader, AssetSource},
    color::palettes,
    ecs::world::DeferredWorld,
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    ui,
};
use bevy_reactor::*;

fn style_main(ss: &mut StyleBuilder) {
    ss.position(ui::PositionType::Absolute)
        .left(0)
        .top(0)
        .bottom(0)
        .right(0)
        .border(1)
        .border_color(colors::U2)
        .display(ui::Display::Flex)
        .pointer_events(false);
}

fn style_aside(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .background_color(colors::U2)
        .padding(8)
        .gap(8)
        .flex_direction(ui::FlexDirection::Column)
        .width(200)
        .border(1)
        .pointer_events(true);
}

fn style_button_row(ss: &mut StyleBuilder) {
    ss.gap(8);
}

fn style_button_flex(ss: &mut StyleBuilder) {
    ss.flex_grow(1.);
}

fn style_slider(ss: &mut StyleBuilder) {
    ss.align_self(ui::AlignSelf::Stretch);
}

fn style_column_group(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Column)
        .align_items(ui::AlignItems::FlexStart)
        .gap(8);
}

fn style_viewport(ss: &mut StyleBuilder) {
    ss.flex_grow(1.)
        .display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Column)
        .justify_content(ui::JustifyContent::FlexEnd)
        .border_left(1)
        .border_color(Color::BLACK)
        .pointer_events(false);
}

fn style_log(ss: &mut StyleBuilder) {
    ss.background_color("#0008")
        .display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .align_self(ui::AlignSelf::Stretch)
        .height(ui::Val::Percent(30.))
        .margin(8);
}

fn style_log_inner(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Column)
        .justify_content(ui::JustifyContent::FlexEnd)
        .align_self(ui::AlignSelf::Stretch)
        .flex_grow(1.)
        .flex_basis(0)
        .overflow(ui::OverflowAxis::Clip)
        .gap(3)
        .margin(8);
}

fn style_scroll_area(ss: &mut StyleBuilder) {
    ss.flex_grow(1.0);
}

// fn style_log_entry(ss: &mut StyleBuilder) {
//     ss.display(ui::Display::Flex)
//         .justify_content(ui::JustifyContent::SpaceBetween)
//         .align_self(ui::AlignSelf::Stretch);
// }

#[derive(Resource)]
pub struct PanelWidth(f32);

#[derive(Resource)]
pub struct PanelHeight(f32);

#[derive(Resource, Default)]
pub struct SelectedShape(Option<Entity>);

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum EditorState {
    #[default]
    Preview,
    Graph,
    Split,
}

#[derive(Resource)]
pub struct PreviewEntities {
    camera: Entity,
    overlay: Entity,
}

fn main() {
    App::new()
        .register_asset_source(
            "demo",
            AssetSource::build()
                .with_reader(|| Box::new(FileAssetReader::new("examples/complex/assets"))),
        )
        .init_resource::<SelectedShape>()
        .init_resource::<TrackingScopeTracing>()
        // .init_resource::<DemoGraphRoot>()
        // .insert_resource(TestStruct {
        //     unlit: Some(true),
        //     ..default()
        // })
        // .insert_resource(TestStruct2 {
        //     nested: TestStruct::default(),
        //     ..default()
        // })
        // .insert_resource(TestStruct3(true))
        .insert_resource(PanelWidth(200.))
        .insert_resource(PanelHeight(300.))
        // .init_resource::<viewport::ViewportInset>()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(DefaultPickingPlugins)
        .insert_state(EditorState::Preview)
        // .insert_resource(RaycastBackendSettings {
        //     require_markers: true,
        //     ..default()
        // })
        // .add_plugins((
        //     CorePlugin,
        //     InputPlugin,
        //     InteractionPlugin,
        //     BevyUiBackend,
        //     RaycastBackend,
        // ))
        // .add_plugins(InspectorPlugin)
        .add_plugins((
            // ReactorPlugin,
            // ObsidianUiPlugin,
            // overlays::OverlaysPlugin,
            // BackdropBackend,
        ))
        .add_systems(Startup, (setup, setup_ui.pipe(setup_view_root)))
        .add_systems(
            Update,
            (
                close_on_esc,
                rotate.run_if(in_state(EditorState::Preview)),
                rotate.run_if(in_state(EditorState::Split)),
                // viewport::update_viewport_inset.run_if(in_state(EditorState::Preview)),
                // viewport::update_viewport_inset.run_if(in_state(EditorState::Split)),
                // viewport::update_camera_viewport.run_if(in_state(EditorState::Preview)),
                // viewport::update_camera_viewport.run_if(in_state(EditorState::Split)),
            ),
        )
        .add_systems(OnEnter(EditorState::Preview), enter_preview_mode)
        .add_systems(OnExit(EditorState::Preview), exit_preview_mode)
        .add_systems(OnEnter(EditorState::Split), enter_preview_mode)
        .add_systems(OnExit(EditorState::Split), exit_preview_mode)
        .run();
}

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
struct Shape;

const X_EXTENT: f32 = 14.5;

fn setup_view_root(camera: In<Entity>, mut commands: Commands) {
    commands.spawn(DemoUi(*camera).to_root());
}

struct DemoUi(Entity);

impl ViewTemplate for DemoUi {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let mut inc_count = 0;
        let mut dec_count = 0;
        let clicked_increment = cx.create_callback(move |_in: In<()>| {
            inc_count += 1;
            println!("Increment clicked: {} times", inc_count);
        });
        let clicked_decrement = cx.create_callback(move |_in: In<()>| {
            dec_count += 1;
            println!("Decrement clicked: {} times", dec_count);
        });

        let checked_1 = cx.create_mutable(false);
        let checked_2 = cx.create_mutable(true);
        let red = cx.create_mutable::<f32>(128.);
        let name = cx.create_mutable("filename.txt".to_string());

        let panel_width = cx.create_derived(|cx| {
            let res = cx.read_resource::<PanelWidth>();
            res.0
        });

        Element::<NodeBundle>::new()
            .named("Main")
            .style((typography::text_default, style_main))
            .insert((TabGroup::default(), TargetCamera(self.0)))
            .children((
                Dialog::new()
                    .width(ui::Val::Px(400.))
                    .open(checked_1.signal())
                    .on_close(cx.create_callback(move |mut world: DeferredWorld| {
                        checked_1.set(&mut world, false);
                    }))
                    .children((
                        // DialogHeader::new().children("Dialog Header"),
                        // "Dialog Body",
                        // DialogFooter::new().children((
                        //     Button::new()
                        //         .children("Cancel")
                        //         .on_click(cx.create_callback(move |cx, _| {
                        //             checked_1.set(cx, false);
                        //         })),
                        //     Button::new()
                        //         .children("Close")
                        //         .variant(ButtonVariant::Primary)
                        //         .autofocus(true)
                        //         .on_click(cx.create_callback(move |cx, _| {
                        //             checked_1.set(cx, false);
                        //         })),
                        // )),
                    )),
                Element::<NodeBundle>::new()
                    .named("ControlPalette")
                    .style(style_aside)
                    .create_effect(move |cx, ent| {
                        let width = panel_width.get(cx);
                        let mut style = cx.world_mut().get_mut::<ui::Style>(ent).unwrap();
                        style.width = ui::Val::Px(width);
                    })
                    .children((
                        // ToolPalette::new().columns(3).children((
                        //     ToolButton::new()
                        //         .children("Preview")
                        //         .corners(RoundedCorners::Left)
                        //         .variant(cx.create_derived(|cx| {
                        //             let st = cx.read_resource::<State<EditorState>>();
                        //             if *st.get() == EditorState::Preview {
                        //                 ButtonVariant::Selected
                        //             } else {
                        //                 ButtonVariant::Default
                        //             }
                        //         }))
                        //         .on_click(cx.create_callback(|cx, _| {
                        //             if let Some(mut mode) =
                        //                 cx.world_mut().get_resource_mut::<NextState<EditorState>>()
                        //             {
                        //                 mode.set(EditorState::Preview);
                        //             }
                        //         })),
                        //     ToolButton::new()
                        //         .children("Materials")
                        //         .corners(RoundedCorners::None)
                        //         .variant(cx.create_derived(|cx| {
                        //             let st = cx.read_resource::<State<EditorState>>();
                        //             if *st.get() == EditorState::Graph {
                        //                 ButtonVariant::Selected
                        //             } else {
                        //                 ButtonVariant::Default
                        //             }
                        //         }))
                        //         .on_click(cx.create_callback(|cx, _| {
                        //             if let Some(mut mode) =
                        //                 cx.world_mut().get_resource_mut::<NextState<EditorState>>()
                        //             {
                        //                 mode.set(EditorState::Graph);
                        //             }
                        //         })),
                        //     ToolButton::new()
                        //         .children("Split")
                        //         .corners(RoundedCorners::Right)
                        //         .variant(cx.create_derived(|cx| {
                        //             let st = cx.read_resource::<State<EditorState>>();
                        //             if *st.get() == EditorState::Split {
                        //                 ButtonVariant::Selected
                        //             } else {
                        //                 ButtonVariant::Default
                        //             }
                        //         }))
                        //         .on_click(cx.create_callback(|cx, _| {
                        //             if let Some(mut mode) =
                        //                 cx.world_mut().get_resource_mut::<NextState<EditorState>>()
                        //             {
                        //                 mode.set(EditorState::Split);
                        //             }
                        //         })),
                        // )),
                        Element::<NodeBundle>::new()
                            .style(style_button_row)
                            .children((
                                // Button::new()
                                //     .children("Open…")
                                //     .on_click(clicked_increment)
                                //     .style(style_button_flex),
                                // Button::new()
                                //     .children("Save")
                                //     .on_click(clicked_decrement)
                                //     .style(style_button_flex),
                            )),
                        Element::<NodeBundle>::new()
                            .style(style_column_group)
                            .children((
                                Checkbox::new()
                                    .style(|ss: &mut StyleBuilder| {
                                        ss.cursor_image("demo://unlock.png", Vec2::new(8., 8.));
                                    })
                                    .labeled("Include Author Name")
                                    .checked(checked_1.signal())
                                    .on_change(cx.create_callback(
                                        move |mut world: DeferredWorld| {
                                            println!("Include Author Name: {}", checked);
                                            checked_1.set(&mut world, checked);
                                        },
                                    )),
                                Checkbox::new()
                                    .labeled("Include Metadata")
                                    .checked(checked_2.signal())
                                    .on_change(cx.create_callback(move |cx, checked| {
                                        println!("Include Metadata: {}", checked);
                                        checked_2.set(cx, checked);
                                    })),
                            )),
                        Element::<NodeBundle>::new()
                            .style(style_column_group)
                            .children(
                                Slider::new()
                                    .min(0.)
                                    .max(255.)
                                    .value(red.signal())
                                    .style(style_slider)
                                    .precision(1)
                                    .on_change(cx.create_callback(move |cx, value| {
                                        red.set(cx, value);
                                    })),
                            ),
                        TextInput::new(TextInputProps {
                            value: name.signal(),
                            on_change: Some(cx.create_callback(
                                move |cx: &mut Cx, value: String| {
                                    name.set_clone(cx, value.clone());
                                },
                            )),
                            ..default()
                        }),
                        // ResourcePropertyInspector::<TestStruct>::new(),
                        // ResourcePropertyInspector::<TestStruct2>::new(),
                        // ResourcePropertyInspector::<TestStruct3>::new(),
                        ReactionsTable,
                    )),
                Splitter::new()
                    .direction(SplitterDirection::Vertical)
                    .value(panel_width)
                    .on_change(cx.create_callback(|cx: &mut Cx, value: f32| {
                        let mut panel_width =
                            cx.world_mut().get_resource_mut::<PanelWidth>().unwrap();
                        panel_width.0 = value.max(200.);
                    })),
                CenterPanel,
            ))
    }
}

struct CenterPanel;

fn wrapper_style(ss: &mut StyleBuilder) {
    ss.display(Display::Flex)
        .width(ui::Val::Percent(100.))
        .height(ui::Val::Percent(100.))
        .flex_direction(FlexDirection::Column);
}

fn graph_view_style(ss: &mut StyleBuilder) {
    ss.display(Display::Flex).width(ui::Val::Percent(100.));
}

impl ViewTemplate for CenterPanel {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let panel_height = cx.create_derived(|cx| {
            let res = cx.read_resource::<PanelHeight>();
            res.0
        });

        let drag_call_back = cx.create_callback(|cx: &mut Cx, value: f32| {
            let mut panel_height = cx.world_mut().get_resource_mut::<PanelHeight>().unwrap();
            panel_height.0 = value.max(200.);
        });

        Element::<NodeBundle>::new()
            .children((Cond::new(
                |cx: &Rcx| *cx.read_resource::<State<EditorState>>().get() == EditorState::Graph,
                || NodeGraphDemo {},
                move || {
                    Fragment::new((
                        Element::<NodeBundle>::new()
                            .named("Preview")
                            .style(style_viewport)
                            .insert((viewport::ViewportInsetElement, Pickable::IGNORE))
                            .children(
                                Element::<NodeBundle>::new()
                                    .named("Log")
                                    .style(style_log)
                                    .children(Element::<NodeBundle>::new().style(style_log_inner)),
                            ),
                        Cond::new(
                            |cx: &Rcx| {
                                *cx.read_resource::<State<EditorState>>().get()
                                    == EditorState::Split
                            },
                            move || {
                                Fragment::new((
                                    Splitter::new()
                                        .direction(SplitterDirection::Horizontal)
                                        .value(panel_height)
                                        .on_change(drag_call_back),
                                    Element::<NodeBundle>::new()
                                        .style(graph_view_style)
                                        .create_effect(move |cx, ent| {
                                            let height = panel_height.get(cx);
                                            let mut style =
                                                cx.world_mut().get_mut::<ui::Style>(ent).unwrap();
                                            style.height = ui::Val::Px(height);
                                        })
                                        .children(NodeGraphDemo {}),
                                ))
                            },
                            || (),
                        ),
                    ))
                },
            ),))
            .style(wrapper_style)
    }
}

struct ReactionsTable;

impl ViewTemplate for ReactionsTable {
    fn create(&self, _cx: &mut Cx) -> impl IntoView {
        ListView::new()
            .children(For::each(
                |cx| {
                    let tracing = cx.read_resource::<TrackingScopeTracing>();
                    tracing.0.clone().into_iter()
                },
                |ent| {
                    text_computed({
                        let e = *ent;
                        move |cx| {
                            if let Some(name) = cx.world().get::<Name>(e) {
                                name.to_string()
                            } else {
                                e.to_string()
                            }
                        }
                    })
                },
            ))
            .style(style_scroll_area)
    }
}

// fn _overlay_views(cx: &mut Cx<Entity>) -> impl View {
//     let id = cx.create_entity();
//     let hovering = cx.create_hover_signal(id);
//     // let color = cx.create_derived(|cx| LinearRgba::from(cx.read_resource::<ColorEditState>().rgb));
//     let color: Signal<LinearRgba> = cx.create_derived(move |cx| {
//         if hovering.get(cx) {
//             colors::ACCENT.into()
//         } else {
//             colors::U1.into()
//         }
//     });

//     overlays::OverlayShape::for_entity(id, |_cx, sb| {
//         sb.with_stroke_width(0.3)
//             .stroke_circle(Vec2::new(0., 0.), 5., 64)
//             .stroke_polygon(
//                 &[Vec2::new(-4., -4.), Vec2::new(0., -4.), Vec2::new(-4., 0.)],
//                 overlays::PolygonOptions {
//                     start_marker: overlays::StrokeMarker::Arrowhead,
//                     end_marker: overlays::StrokeMarker::Arrowhead,
//                     // dash_length: 0.1,
//                     // gap_length: 0.1,
//                     closed: true,
//                     ..default()
//                 },
//             );
//     })
//     .with_color_signal(color)
//     .with_pickable(true)
//     // .with_transform(Transform::from_rotation(Quat::from_rotation_y(PI * 0.5)))
//     .insert(TargetCamera(cx.props))
// }

// struct TransformOverlayDemo;

// impl ViewTemplate for TransformOverlayDemo {
//     fn create(&self, cx: &mut Cx) -> impl IntoView {
//         let selected = cx.create_derived(|cx| cx.read_resource::<SelectedShape>().0);

//         let on_change = Some(cx.create_callback(move |cx, new_pos| {
//             let selected = selected.get(cx).unwrap();
//             let mut entity = cx.world_mut().entity_mut(selected);
//             let mut transform = entity.get_mut::<Transform>().unwrap();
//             transform.translation = new_pos;
//         }));

//         TransformOverlay {
//             target: selected,
//             on_change,
//         }
//     }
// }

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

    let shapes_parent = commands
        .spawn((
            SpatialBundle { ..default() },
            // BackdropPickable,
            // On::<Pointer<Down>>::run(
            //     |mut event: ListenerMut<Pointer<Down>>,
            //      shapes: Query<&Shape>,
            //      mut selection: ResMut<SelectedShape>| {
            //         if shapes.get(event.target).is_ok() {
            //             selection.0 = Some(event.target);
            //             // println!("Pointer down on shape {:?}", event.target);
            //         } else {
            //             selection.0 = None;
            //             // println!("Pointer down on backdrop {:?}", event.target);
            //         }
            //         event.stop_propagation();
            //     },
            // ),
        ))
        .id();

    for (i, shape) in shapes.into_iter().enumerate() {
        commands
            .spawn((
                PbrBundle {
                    mesh: shape,
                    material: debug_material.clone(),
                    transform: Transform::from_xyz(
                        -X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * X_EXTENT,
                        2.0,
                        0.0,
                    )
                    .with_rotation(Quat::from_rotation_x(-PI / 4.)),
                    ..default()
                },
                Shape,
                // PickableBundle::default(),
                // RaycastPickable,
            ))
            .set_parent(shapes_parent);
    }

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            // intensity: 9000.0,
            intensity: 10000000.0,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });

    // ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(50.0, 50.0)),
        material: materials.add(Color::from(palettes::css::SILVER)),
        ..default()
    });
}

fn setup_ui(mut commands: Commands) -> Entity {
    commands
        .spawn((Camera2dBundle {
            camera: Camera {
                // HUD goes on top of 3D
                order: 1,
                clear_color: ClearColorConfig::None,
                ..default()
            },
            camera_2d: Camera2d {},
            ..default()
        },))
        .id()
}

fn enter_preview_mode(mut commands: Commands) {
    let camera = commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(0.0, 6., 12.0)
                    .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
                ..default()
            },
            // viewport::ViewportCamera,
            RaycastPickable,
            // BackdropPickable,
        ))
        .id();

    let overlay = commands.spawn(TransformOverlayDemo.to_root()).id();
    commands.insert_resource(PreviewEntities { camera, overlay });
}

fn exit_preview_mode(mut commands: Commands, preview: Res<PreviewEntities>) {
    commands.entity(preview.camera).despawn();
    commands.queue(DespawnViewRoot::new(preview.overlay));
    commands.remove_resource::<PreviewEntities>()
}

fn rotate(mut query: Query<&mut Transform, With<Shape>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_seconds() / 2.);
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
