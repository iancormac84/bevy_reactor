use bevy::{
    core::Name,
    ecs::{observer::ObserverState, system::SystemIdMarker, world::DeferredWorld},
    pbr::{DirectionalLight, PointLight},
    prelude::{
        Camera2d, Camera3d, Children, Click, Component, Entity, In, Mesh3d, Parent,
        Pointer, Query, ResMut, Resource, Trigger, Without, World,
    },
    ui::{self, experimental::GhostNode, Node},
    window::{Monitor, Window},
};
use bevy_mod_stylebuilder::{
    StyleBuilder, StyleBuilderBackground, StyleBuilderBorderColor, StyleBuilderBorderRadius,
    StyleBuilderFont, StyleBuilderLayout, StyleBuilderZIndex,
};
use bevy_reactor_builder::{
    CondBuilder, CreateChilden, EntityStyleBuilder, ForEachBuilder, InvokeUiTemplate, TextBuilder,
    UiTemplate,
};
use bevy_reactor_obsidian::{
    colors,
    prelude::{DisclosureToggle, ScrollView},
    typography,
};
use bevy_reactor_signals::ReactionCell;

fn style_panel(sb: &mut StyleBuilder) {
    sb.position(ui::PositionType::Absolute)
        .display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Column)
        .align_items(ui::AlignItems::Stretch)
        .row_gap(4)
        .left(20)
        .top(20)
        .width(300)
        .height(400)
        .padding(4)
        .background_color(colors::BACKGROUND)
        .border(2)
        .border_color(colors::U1)
        .border_radius(4.)
        .z_index(1000);
}

pub(crate) fn create_inspector_panel(world: &mut World) {
    world
        .spawn((
            Node::default(),
            Name::new("InspectorPanel"),
            InspectorPanelRoot,
        ))
        .styles((typography::text_default, style_panel))
        .create_children(|builder| {
            builder.invoke(TopLevelItemList);
        });
}

#[derive(Component)]
pub struct InspectorPanelRoot;

fn style_item_list(sb: &mut StyleBuilder) {
    sb.flex_grow(1.).background_color(colors::U1).padding(2);
}

fn style_item_list_content(sb: &mut StyleBuilder) {
    sb.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Column)
        .color(colors::FOREGROUND);
}

struct TopLevelItemList;

impl UiTemplate for TopLevelItemList {
    fn build(&self, builder: &mut bevy_reactor_builder::UiBuilder) {
        builder.invoke(
            ScrollView::new()
                .style(style_item_list)
                .content_style((typography::text_default, style_item_list_content))
                .scroll_enable_y(true)
                .children(|builder| {
                    builder.for_each(
                        |rcx| {
                            rcx.read_resource::<TopLevelEntities>()
                                .0
                                .clone()
                                .into_iter()
                        },
                        |ent, builder| {
                            builder.invoke(EntityTreeNode(*ent));
                        },
                        |_| {},
                    );
                }),
        );
    }
}

#[derive(Resource, Default)]
pub(crate) struct TopLevelEntities(Vec<Entity>);

#[allow(clippy::type_complexity)]
pub fn copy_top_level_entities(
    q_entities: Query<
        Entity,
        (
            Without<Parent>,
            Without<InspectorPanelRoot>,
            Without<ObserverState>,
            Without<SystemIdMarker>,
        ),
    >,
    mut r_list: ResMut<TopLevelEntities>,
) {
    let mut entities: Vec<Entity> = q_entities.iter().collect();
    // TODO: Sort
    if entities != r_list.0 {
        println!("Entity count: {}", entities.len());
        std::mem::swap(&mut entities, &mut r_list.0);
    }
}

fn style_tree_node(sb: &mut StyleBuilder) {
    sb.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Column);
}

fn style_tree_node_label(sb: &mut StyleBuilder) {
    sb.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row);
}

fn style_tree_node_children(sb: &mut StyleBuilder) {
    sb.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Column)
        .padding_left(16);
}

struct EntityTreeNode(Entity);

impl UiTemplate for EntityTreeNode {
    fn build(&self, builder: &mut bevy_reactor_builder::UiBuilder) {
        if builder.world().get_entity(self.0).is_err() {
            return;
        }
        builder
            .spawn((Node::default(), Name::new("EntityTreeNode")))
            .style(style_tree_node)
            .create_children(|builder| {
                let entid = self.0;
                let expanded = builder.create_mutable(false);
                let on_expand =
                    builder.create_callback(move |value: In<bool>, mut world: DeferredWorld| {
                        expanded.set(&mut world, *value);
                    });
                builder
                    .spawn(Node::default())
                    .style(style_tree_node_label)
                    .observe(
                        move |mut trigger: Trigger<Pointer<Click>>, mut world: DeferredWorld| {
                            trigger.propagate(false);
                            let value = expanded.get(&world);
                            expanded.set(&mut world, !value);
                        },
                    )
                    .create_children(|builder| {
                        builder.invoke(
                            DisclosureToggle::new()
                                .expanded(expanded)
                                .on_change(on_expand),
                        );
                        builder.text(entid.to_string());
                        builder.text(" ");
                        // Compute a label for the entity, based on either the explicit name,
                        // or the component that is most likely to be the defining trait.
                        builder.computed(
                            move |rcx| {
                                if let Some(name) = rcx.read_component::<Name>(entid) {
                                    return Some(name.to_string());
                                };

                                // Note: Should be using read_component here, but not all of
                                // these component types may be registered, which panics.
                                let ent = rcx.world().entity(entid);
                                if ent.get::<Window>().is_some() {
                                    Some("Window".to_string())
                                } else if ent.get::<Monitor>().is_some() {
                                    Some("Monitor".to_string())
                                } else if ent.get::<Camera2d>().is_some() {
                                    Some("Camera2d".to_string())
                                } else if ent.get::<Camera3d>().is_some() {
                                    Some("Camera3d".to_string())
                                } else if ent.get::<PointLight>().is_some() {
                                    Some("PointLight".to_string())
                                } else if ent.get::<DirectionalLight>().is_some() {
                                    Some("DirectionalLight".to_string())
                                } else if ent.get::<Mesh3d>().is_some() {
                                    Some("Mesh3d".to_string())
                                } else if ent.get::<Node>().is_some() {
                                    Some("Node".to_string())
                                } else if ent.get::<GhostNode>().is_some() {
                                    Some("Ghost".to_string())
                                } else if ent.get::<ReactionCell>().is_some() {
                                    Some("ReactionCell".to_string())
                                } else {
                                    None
                                }
                            },
                            |name, builder| {
                                if let Some(name) = name {
                                    builder.text(name);
                                }
                            },
                        );
                    });
                builder.cond(
                    expanded.signal(),
                    move |builder| {
                        builder
                            .spawn(Node::default())
                            .style(style_tree_node_children)
                            .create_children(|builder| {
                                let component_names: Vec<String> = builder
                                    .world()
                                    .inspect_entity(entid)
                                    .map(|c| c.name().to_string())
                                    .collect();
                                for comp in component_names {
                                    match comp.rsplit_once("::") {
                                        Some((_, suffix)) => {
                                            builder.text(suffix);
                                        }
                                        None => {
                                            builder.text(comp);
                                        }
                                    }
                                }

                                // drop(components);
                                builder.for_each(
                                    move |rcx| {
                                        rcx.read_component::<Children>(entid)
                                            .map(|c| c.to_vec())
                                            .unwrap_or_default()
                                            .into_iter()
                                    },
                                    |item, builder| {
                                        builder.invoke(EntityTreeNode(*item));
                                    },
                                    |_| {},
                                );
                            });
                    },
                    |_| {},
                );
            });
    }
}
