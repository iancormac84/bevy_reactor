use std::ops::Range;

use bevy::reflect::Reflect;
use bevy_reactor::*;
use bevy_reactor_signals::{Cx, RunContextSetup};
use obsidian_ui::controls::Slider;

use crate::{
    templates::{field_label::FieldLabel, field_readonly_value::FieldReadonlyValue},
    InspectableField, Precision, Step, ValueRange,
};

#[derive(Clone, Debug)]
struct F32Attrs {
    range: Option<Range<f32>>,
    precision: usize,
    step: f32,
}

pub struct FieldEditF32(pub(crate) InspectableField);

impl ViewTemplate for FieldEditF32 {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let field = self.0.clone();
        let value = cx.create_memo(move |cx| {
            if let Some(value) = field.reflect(cx) {
                if value.is::<f32>() {
                    return *value.downcast_ref::<f32>().unwrap();
                }
            }
            0.0
        });

        let field = self.0.clone();
        let Some(reflect) = field.reflect(cx) else {
            return ().into_view();
        };

        let mut slider_params = F32Attrs {
            range: None,
            precision: 0,
            step: 1.0,
        };

        if let Some(attrs) = field.attributes {
            if let Some(range) = attrs.get::<ValueRange<f32>>() {
                slider_params.range = Some(range.0.clone());
                slider_params.precision =
                    (2. - (range.0.end - range.0.start).log10().ceil()).max(0.) as usize;
            }
            if let Some(precision) = attrs.get::<Precision>() {
                slider_params.precision = precision.0;
            }
            if let Some(step) = attrs.get::<Step<f32>>() {
                slider_params.step = step.0;
            } else {
                slider_params.step = 10.0f32.powi(-(slider_params.precision as i32));
            }
        }

        // let field = self.field.clone();
        Fragment::new((
            FieldLabel {
                field: field.clone(),
            },
            // Don't need `Cond` here because condition is not reactive; reflection data
            // is constant.
            match slider_params.range {
                Some(range) => Slider::new()
                    .min(range.start)
                    .max(range.end)
                    .precision(slider_params.precision)
                    .step(slider_params.step)
                    .value(value)
                    .on_change(cx.create_callback(move |cx, value: f32| {
                        field.update(cx, &|reflect| {
                            reflect.apply(value.as_reflect());
                        });
                    }))
                    .into_view(),
                None => FieldReadonlyValue::new()
                    .children(format!("NoRange: {}", reflect.reflect_type_path()))
                    .into_view(),
            },
        ))
        .into_view()
    }
}
