use bevy::ecs::world::DeferredWorld;
use bevy::{color::Srgba, prelude::*, ui};
use bevy_mod_stylebuilder::*;
use bevy_reactor_builder::{CondBuilder, CreateChilden, EntityStyleBuilder, UiTemplate};
use bevy_reactor_signals::{Callback, IntoSignal, RunCallback, Signal};
// use bevy_tabindex::TabIndex;

use crate::materials::SwatchRectMaterial;

use crate::colors;

fn style_swatch(ss: &mut StyleBuilder) {
    ss.min_width(12)
        .min_height(12)
        .display(ui::Display::Flex)
        .color(colors::FOREGROUND)
        .padding(2);
}

fn style_selection(ss: &mut StyleBuilder) {
    ss.border(1)
        .border_color(colors::U1)
        .outline_color(colors::FOREGROUND)
        .outline_width(2)
        .outline_offset(0)
        .align_self(ui::AlignSelf::Stretch)
        .justify_self(ui::JustifySelf::Stretch)
        .flex_grow(1.);
}

/// Color swatch widget. This displays a solid color, and can also display a checkerboard
/// pattern behind the color if it has an alpha of less than 1.
#[derive(Default)]
pub struct Swatch {
    /// Color to display
    pub color: Signal<Srgba>,

    /// For swatch grids, whether this swatch is selected.
    pub selected: Signal<bool>,

    /// Additional styles to be applied to the widget.
    pub style: StyleHandle,

    /// Callback called when clicked
    pub on_click: Option<Callback<Srgba>>,
}

impl Swatch {
    /// Create a new swatch.
    pub fn new(color: impl IntoSignal<Srgba>) -> Self {
        Self::default().color(color.into_signal())
    }

    /// Set the color to display.
    pub fn color(mut self, color: impl Into<Signal<Srgba>>) -> Self {
        self.color = color.into();
        self
    }

    /// Set additional styles to be applied to the button.
    pub fn style<S: StyleTuple + 'static>(mut self, style: S) -> Self {
        self.style = style.into_handle();
        self
    }

    /// Set the callback called when clicked.
    pub fn on_click(mut self, on_click: Callback<Srgba>) -> Self {
        self.on_click = Some(on_click);
        self
    }

    /// Set whether the swatch should be rendered in a 'selected' state.
    pub fn selected(mut self, selected: impl IntoSignal<bool>) -> Self {
        self.selected = selected.into_signal();
        self
    }
}

impl UiTemplate for Swatch {
    fn build(&self, builder: &mut bevy_reactor_builder::UiBuilder) {
        let color = self.color;
        let selected = self.selected;
        let on_click = self.on_click;

        let mut ui_materials = builder
            .world_mut()
            .get_resource_mut::<Assets<SwatchRectMaterial>>()
            .unwrap();
        let material = ui_materials.add(SwatchRectMaterial {
            color: LinearRgba::from(colors::U1).to_vec4(),
            border_radius: Vec4::splat(0.),
        });

        // Update material color
        builder.create_effect({
            let material = material.clone();
            move |rcx| {
                let color = color.get(rcx);
                // println!("Update color: {}", color.to_hex());
                let mut ui_materials = rcx
                    .world_mut()
                    .get_resource_mut::<Assets<SwatchRectMaterial>>()
                    .unwrap();
                let material = ui_materials.get_mut(material.id()).unwrap();
                material.color = color.to_vec4();
            }
        });

        builder
            .spawn((MaterialNode::<SwatchRectMaterial>(material.clone()), Name::new("Swatch")))
            .styles((style_swatch, self.style.clone()))
            .observe(
                move |mut trigger: Trigger<Pointer<Click>>,
                world: DeferredWorld,
                      mut commands: Commands| {
                    trigger.propagate(false);
                        if let Some(on_click) = on_click {
                            let c = color.get(&world);
                            commands.run_callback(on_click, c);
                        }
                },
            )
            .create_children(|builder| {
                builder.cond(
                    selected,
                    |builder| { builder.spawn(Node::default()).style(style_selection);},
                    |_builder| {},
                );
            })
        // .effect(move |cx, ent| {
        //     let radius = cx.use_component::<BorderRadius>(ent);
        //     if let Some(radius) = radius {
        //         let radius = Vec4::from_array(resolve_border_radius(radius));
        //         let mut ui_materials = cx
        //             .world_mut()
        //             .get_resource_mut::<Assets<SwatchRectMaterial>>()
        //             .unwrap();
        //         let material = ui_materials.get_mut(material.id()).unwrap();
        //         material.border_radius = radius;
        //     }
        // })
        ;
    }
}

// For now we only support pixel units.
fn _resolve_border_radius(&values: &BorderRadius) -> [f32; 4] {
    [
        values.top_left,
        values.top_right,
        values.bottom_right,
        values.bottom_left,
    ]
    .map(|value| match value {
        Val::Px(px) => px,
        _ => 0.,
    })
}
