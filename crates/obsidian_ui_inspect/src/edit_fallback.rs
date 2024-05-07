use bevy_reactor::*;
use obsidian_ui::controls::InspectorFieldReadonlyValue;

use crate::{field_label::FieldLabel, InspectableField};

/// Field editor for when no specific editor is available.
pub struct FieldEditFallback(pub(crate) InspectableField);

impl ViewTemplate for FieldEditFallback {
    fn create(&self, cx: &mut Cx) -> impl Into<ViewRef> {
        let field = self.0.clone();
        let reflect = field.reflect(cx);
        // let is_checked = cx.create_derived(move |cx| {
        //     let value = field.get_value(cx);
        //     if value.is::<bool>() {
        //         return *value.downcast_ref::<bool>().unwrap();
        //     }
        //     false
        // });

        // let field = self.field.clone();
        Fragment::new((
            FieldLabel {
                field: self.0.clone(),
            },
            InspectorFieldReadonlyValue::new()
                .children(format!("TODO: {}", reflect.reflect_type_path())),
        ))
    }
}