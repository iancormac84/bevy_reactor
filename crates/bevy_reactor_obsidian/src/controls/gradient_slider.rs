use bevy::{color::Srgba, ecs::world::DeferredWorld, prelude::*, ui};
use bevy_mod_stylebuilder::*;
use bevy_reactor_builder::{
    CreateChilden, EntityEffectBuilder, EntityStyleBuilder, UiBuilder, UiTemplate,
};
use bevy_reactor_signals::{Callback, IntoSignal, RunCallback, Signal};

use crate::materials::GradientRectMaterial;

use super::{core_slider::ValueChange, CoreSlider};

const THUMB_WIDTH: f32 = 12.;

/// Struct representing a sequence of color stops, evenly spaced. Up to 8 stops are supported.
#[derive(Debug, Copy, Clone)]
pub struct ColorGradient {
    /// Number of color stops.
    pub num_colors: usize,

    /// Array of color stops.
    pub colors: [Srgba; 8],
}

impl ColorGradient {
    /// Construct a new color gradient from an array of colors.
    pub fn new(colors: &[Srgba]) -> Self {
        assert!(colors.len() <= 8);
        let mut result = Self {
            num_colors: colors.len(),
            colors: [Srgba::default(); 8],
        };
        for (i, color) in colors.iter().enumerate() {
            result.colors[i] = *color;
        }
        result
    }

    /// Return the first color in the gradient, if any.
    pub fn first(&self) -> Option<Srgba> {
        if self.num_colors > 0 {
            Some(self.colors[0])
        } else {
            None
        }
    }

    /// Return the last color in the gradient, if any.
    pub fn last(&self) -> Option<Srgba> {
        if self.num_colors > 0 {
            Some(self.colors[self.num_colors - 1])
        } else {
            None
        }
    }

    /// Return the number of color stops in the gradient.
    pub fn len(&self) -> usize {
        self.num_colors
    }

    /// Check if the gradient is empty.
    pub fn is_empty(&self) -> bool {
        self.num_colors == 0
    }
}

impl Default for ColorGradient {
    fn default() -> Self {
        Self {
            num_colors: 1,
            colors: [Srgba::BLACK; 8],
        }
    }
}

fn style_slider(ss: &mut StyleBuilder) {
    ss.min_width(32)
        .height(14)
        .display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .align_items(ui::AlignItems::Stretch);
}

fn style_gradient(ss: &mut StyleBuilder) {
    ss.flex_grow(1.);
}

fn style_track(ss: &mut StyleBuilder) {
    ss.position(ui::PositionType::Absolute)
        .top(1)
        .bottom(1)
        .left(1)
        .right(THUMB_WIDTH + 1.);
}

fn style_thumb(ss: &mut StyleBuilder) {
    ss.background_image("embedded://bevy_reactor_obsidian/assets/icons/gradient_thumb.png")
        .position(ui::PositionType::Absolute)
        .top(0)
        .bottom(0)
        .width(THUMB_WIDTH);
}

/// Horizontal slider widget that displays a gradient bar and a draggable button.
pub struct GradientSlider {
    /// Gradient to display.
    pub gradient: Signal<ColorGradient>,

    /// Current slider value.
    pub value: Signal<f32>,

    /// Minimum slider value.
    pub min: Signal<f32>,

    /// Maximum slider value.
    pub max: Signal<f32>,

    /// Number of decimal places to round to (0 = integer).
    pub precision: usize,

    /// Whether the slider is disabled.
    pub disabled: Signal<bool>,

    /// Style handle for slider root element.
    pub style: StyleHandle,

    /// Callback called when value changes
    pub on_change: Option<Callback<f32>>,
}

impl GradientSlider {
    /// Create a new gradient slider.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the gradient to display.
    pub fn gradient(mut self, gradient: impl IntoSignal<ColorGradient>) -> Self {
        self.gradient = gradient.into_signal();
        self
    }

    /// Set the current slider value.
    pub fn value(mut self, value: impl IntoSignal<f32>) -> Self {
        self.value = value.into_signal();
        self
    }

    /// Set the minimum slider value.
    pub fn min(mut self, min: impl IntoSignal<f32>) -> Self {
        self.min = min.into_signal();
        self
    }

    /// Set the maximum slider value.
    pub fn max(mut self, max: impl IntoSignal<f32>) -> Self {
        self.max = max.into_signal();
        self
    }

    /// Set the number of decimal places to round to (0 = integer).
    pub fn precision(mut self, precision: usize) -> Self {
        self.precision = precision;
        self
    }

    /// Set whether the slider is disabled.
    pub fn disabled(mut self, disabled: impl IntoSignal<bool>) -> Self {
        self.disabled = disabled.into_signal();
        self
    }

    /// Set the style handle for the slider root element.
    pub fn style<S: StyleTuple + 'static>(mut self, style: S) -> Self {
        self.style = style.into_handle();
        self
    }

    /// Set the callback called when the value changes.
    pub fn on_change(mut self, on_change: Callback<f32>) -> Self {
        self.on_change = Some(on_change);
        self
    }
}

impl Default for GradientSlider {
    fn default() -> Self {
        Self {
            gradient: Signal::Constant(ColorGradient::default()),
            value: Signal::Constant(0.),
            min: Signal::Constant(0.),
            max: Signal::Constant(1.),
            precision: 0,
            disabled: Signal::Constant(false),
            style: StyleHandle::default(),
            on_change: None,
        }
    }
}

impl UiTemplate for GradientSlider {
    fn build(&self, builder: &mut UiBuilder) {
        let slider_id = builder
            .spawn((Node::default(), Name::new("GradientSlider")))
            .id();

        // Pain point: Need to capture all props for closures.
        let min = self.min;
        let max = self.max;
        let value = self.value;
        let precision = self.precision;
        let on_change = self.on_change;

        // This should really be an effect.
        let color_stops: Signal<(usize, [Vec4; 8])> = {
            let gradient = self.gradient;
            builder.create_derived(move |rcx| {
                gradient.map(rcx, |g| {
                    let mut result: [Vec4; 8] = [Vec4::default(); 8];
                    let num_color_stops = g.len();
                    for (i, color) in g.colors[0..num_color_stops].iter().enumerate() {
                        // Note that we do *not* convert to linear here, because interpolating
                        // linear looks bad. That gets done in the shader.
                        result[i] = Vec4::new(color.red, color.green, color.blue, color.alpha);
                    }
                    (g.len(), result)
                })
            })
        };

        let mut gradient_material_assets = builder
            .world_mut()
            .get_resource_mut::<Assets<GradientRectMaterial>>()
            .unwrap();
        let gradient_material = gradient_material_assets.add(GradientRectMaterial {
            color_stops: [Srgba::default().to_vec4(); 8],
            num_color_stops: IVec4::new(2, 0, 0, 0),
            cap_size: THUMB_WIDTH * 0.5,
        });

        // Effect to update the material handle.
        builder.create_effect({
            let material = gradient_material.clone();
            move |rcx| {
                let (num_color_stops, color_stops) = color_stops.get(rcx);
                let mut ui_materials = rcx
                    .world_mut()
                    .get_resource_mut::<Assets<GradientRectMaterial>>()
                    .unwrap();
                let material = ui_materials.get_mut(material.id()).unwrap();
                material.num_color_stops.x = num_color_stops as i32;
                material.color_stops = color_stops;
            }
        });

        builder
            .entity_mut(slider_id)
            .styles((style_slider, self.style.clone()))
            .effect(
                move |rcx| CoreSlider::new(value.get(rcx), min.get(rcx), max.get(rcx)),
                |slider, ent| {
                    ent.insert(slider);
                },
            )
            .observe(
                move |mut trigger: Trigger<ValueChange<f32>>, mut world: DeferredWorld| {
                    trigger.propagate(false);
                    let event = trigger.event();
                    let rounding = f32::powi(10., precision as i32);
                    let value = value.get(&world);
                    let new_value = ((event.0 * rounding).round() / rounding)
                        .clamp(min.get(&world), max.get(&world));
                    if value != new_value {
                        if let Some(on_change) = on_change {
                            world.run_callback(on_change, new_value);
                        }
                    }
                },
            )
            .observe(
                move |mut trigger: Trigger<Pointer<Down>>, mut world: DeferredWorld| {
                    trigger.propagate(false);
                    let min = min.get(&world);
                    let max = max.get(&world);
                    let hit_x = trigger.event().pointer_location.position.x;
                    let ent = world.entity(slider_id);
                    let node = ent.get::<ComputedNode>();
                    let transform = ent.get::<GlobalTransform>();
                    if let (Some(node), Some(transform)) = (node, transform) {
                        // If not clicking on thumb, then snap thumb to new location.
                        let rect =
                            Rect::from_center_size(transform.translation().xy(), node.size());
                        let slider_width = rect.width() - THUMB_WIDTH;
                        let range = max - min;
                        let pointer_pos = hit_x - rect.min.x - THUMB_WIDTH / 2.;
                        let thumb_pos =
                            value.get(&world) - min * slider_width / range + THUMB_WIDTH / 2.;
                        if range > 0. && (pointer_pos - thumb_pos).abs() >= THUMB_WIDTH / 2. {
                            let new_value = min + (pointer_pos * range) / slider_width;
                            if let Some(on_change) = on_change {
                                world.run_callback(on_change, new_value.clamp(min, max));
                            }
                        };
                    }
                },
            )
            .create_children(|builder| {
                builder
                    .spawn(MaterialNode::<GradientRectMaterial>(gradient_material.clone()))
                    .style(style_gradient);
                builder
                    .spawn((Node::default(), Name::new("GradientSlider::Track")))
                    .style(style_track)
                    .create_children(|builder| {
                        builder
                            .spawn((Node::default(), Name::new("GradientSlider::Thumb")))
                            .style(style_thumb)
                            .style_dyn(
                                move |rcx| {
                                    let min = min.get(rcx);
                                    let max = max.get(rcx);
                                    let value = value.get(rcx);
                                    if max > min {
                                        ((value - min) / (max - min)).clamp(0., 1.)
                                    } else {
                                        0.
                                    }
                                },
                                |percent, sb| {
                                    sb.left(ui::Val::Percent(percent * 100.));
                                },
                            );
                    });
            });
    }
}
