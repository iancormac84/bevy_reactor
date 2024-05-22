use std::sync::Arc;

use bevy::{
    prelude::*,
    reflect::{OffsetAccess, ReflectMut, ReflectRef, TypeInfo},
    ui,
};
use bevy_reactor::*;
use bevy_reactor_signals::{Cx, RunContextRead, RunContextSetup, Signal};
use obsidian_ui::{
    colors,
    controls::{DisclosureToggle, IconButton},
    size::Size,
};

use crate::{templates::field_label::FieldLabelWide, InspectableField, InspectorFactoryRegistry};

pub struct FieldEditList(pub(crate) InspectableField);

impl ViewTemplate for FieldEditList {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let field = self.0.clone();
        let expanded = cx.create_mutable(false);
        let length = cx.create_memo(move |cx| {
            if let Some(value) = field.reflect(cx) {
                return if let ReflectRef::List(list) = value.reflect_ref() {
                    list.len()
                } else {
                    0
                };
            }
            0
        });

        let pop_disabled = cx.create_derived(move |cx| length.get(cx) == 0);

        let field = self.0.clone();
        let push = cx.create_callback(move |cx, _| {
            if let Some(list) = field.reflect(cx) {
                if let TypeInfo::List(list_type) = list.get_represented_type_info().unwrap() {
                    let registry = cx.world().resource::<AppTypeRegistry>().0.clone();
                    let registry_lock = registry.read();
                    let item_type =
                        registry_lock.get_type_data::<ReflectDefault>(list_type.item_type_id());
                    let default = item_type.unwrap().default();
                    field.update(cx, &|reflect| {
                        if let ReflectMut::List(list) = reflect.reflect_mut() {
                            list.push(default.clone_value());
                        }
                    });
                    // Auto expand when pushing.
                    expanded.set(cx, true);
                } else {
                    unreachable!("Expected List type ");
                }
            } else {
                unreachable!("Cannot push to non-list");
            }
        });

        let field = self.0.clone();
        let pop = cx.create_callback(move |cx, _| {
            field.update(cx, &|reflect| {
                if let ReflectMut::List(list) = reflect.reflect_mut() {
                    if !list.is_empty() {
                        list.pop();
                    }
                } else {
                    unreachable!("Cannot pop from non-list")
                }
            })
        });

        let field = self.0.clone();
        Fragment::new((
            FieldLabelWide::new(field.clone())
                .name(Fragment::new((
                    DisclosureToggle::new()
                        .size(Size::Xs)
                        .expanded(expanded)
                        .on_change(cx.create_callback(move |cx, value: bool| {
                            expanded.set(cx, value);
                        })),
                    TextComputed::new(move |cx| {
                        let length = length.get(cx);
                        format!("{} ({})", field.name.clone(), length)
                    }),
                )))
                .buttons(Fragment::new((
                    IconButton::new("obsidian_ui://icons/remove.png")
                        .size(Size::Xs)
                        .disabled(pop_disabled)
                        .minimal(true)
                        .on_click(pop),
                    IconButton::new("obsidian_ui://icons/add.png")
                        .size(Size::Xs)
                        .minimal(true)
                        .on_click(push),
                ))),
            Cond::new(
                expanded.signal(),
                {
                    let field = self.0.clone();
                    move || ListContentInspector {
                        field: field.clone(),
                        length,
                    }
                },
                || (),
            ),
        ))
    }
}

struct ListContentInspector {
    field: InspectableField,
    length: Signal<usize>,
}

impl ViewTemplate for ListContentInspector {
    fn create(&self, _cx: &mut Cx) -> impl IntoView {
        let field = self.field.clone();
        let length = self.length;
        Element::<NodeBundle>::new()
            .style(style_list_items)
            .children(
                For::index(
                    move |cx| 0..length.get(cx),
                    move |_, index| {
                        let mut path = field.value_path.clone();
                        path.0.push(OffsetAccess {
                            access: bevy::reflect::Access::ListIndex(index),
                            offset: None,
                        });
                        let access = Arc::new(InspectableField {
                            root: field.root.clone(),
                            name: format!("{}", index),
                            value_path: path,
                            field_path: field.value_path.clone(),
                            can_remove: false,
                            attributes: field.attributes,
                        });
                        ListItemInspector { field: access }.into_view()
                    },
                )
                .with_fallback(
                    Element::<NodeBundle>::new()
                        .style(style_empty_list)
                        .children("(empty list)"),
                ),
            )
    }
}

struct ListItemInspector {
    field: Arc<InspectableField>,
}

impl ViewTemplate for ListItemInspector {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let factories = cx.use_resource::<InspectorFactoryRegistry>();
        let field = self.field.clone();
        for factory in factories.0.iter().rev() {
            if let Some(view_ref) = factory.create_inspector(cx, &field) {
                return view_ref;
            }
        }

        // No inspector found, don't render anything. Note that default factory already
        // has a fallback, so this should never be reached.
        ().into_view()
    }
}

fn style_list_items(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Grid)
        .grid_auto_flow(ui::GridAutoFlow::Row)
        .grid_template_columns(vec![
            ui::RepeatedGridTrack::auto(1),
            ui::RepeatedGridTrack::flex(1, 1.),
        ])
        .column_gap(4)
        .row_gap(2)
        .align_items(ui::AlignItems::Stretch)
        .grid_column_span(2)
        .min_width(64)
        .color(colors::DIM)
        // .border(1)
        // .border_color(colors::X_RED)
        .margin_left(16);
}

// fn style_item_index(ss: &mut StyleBuilder) {
//     ss.justify_content(ui::JustifyContent::FlexEnd);
// }

fn style_empty_list(ss: &mut StyleBuilder) {
    ss.color(colors::DIM);
}
