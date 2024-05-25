use std::sync::Arc;

use bevy::prelude::*;
use bevy_reactor::*;
use bevy_reactor_signals::Cx;
use obsidian_ui_inspect::{InspectableResource, Inspector, Precision, ValueRange};

#[derive(Debug, Reflect, Clone, Default)]
pub enum TestEnum {
    #[default]
    Unit,
    Float(f32),
    Color(Srgba),
    Struct {
        position: Vec3,
        color: Srgba,
    },
}

#[derive(Resource, Debug, Reflect, Clone, Default)]
pub struct TestStruct {
    pub selected: bool,

    #[reflect(@ValueRange::<f32>(0.0..1.0))]
    pub scale: f32,

    pub color: Srgba,
    pub position: Vec3,
    pub unlit: Option<bool>,

    #[reflect(@ValueRange::<f32>(0.0..10.0))]
    pub roughness: Option<f32>,

    #[reflect(@Precision(2))]
    pub metalness: Option<f32>,

    #[reflect(@ValueRange::<f32>(0.0..1000.0))]
    pub factors: Vec<f32>,
}

#[derive(Resource, Debug, Reflect, Clone, Default)]
pub struct TestStruct2 {
    pub nested: TestStruct,
    pub choice: TestEnum,
}

#[derive(Resource, Debug, Reflect, Clone, Default)]
pub struct TestStruct3(pub bool);

pub struct ResourcePropertyInspector<T: Resource> {
    marker: std::marker::PhantomData<T>,
}

impl<T: Resource> ResourcePropertyInspector<T> {
    pub fn new() -> Self {
        Self {
            marker: std::marker::PhantomData,
        }
    }
}

impl<T: Resource + Reflect> ViewTemplate for ResourcePropertyInspector<T> {
    fn create(&self, _cx: &mut Cx) -> impl IntoView {
        Inspector::new(Arc::<InspectableResource<T>>::default())
    }
}
