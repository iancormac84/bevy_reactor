use std::sync::{Arc, Mutex};

use bevy::ecs::world::World;
use bevy::prelude::*;

use crate::node_span::NodeSpan;
use crate::{
    DespawnScopes, DisplayNodeChanged, IntoView, Rcx, TrackingScope, View, ViewContext, ViewRef,
};

// Cond

pub enum CondState {
    Unset,
    True((ViewRef, Entity)),
    False((ViewRef, Entity)),
}

/// A conditional view which renders one of two children depending on the condition expression.
pub struct Cond<Test: 'static, Pos: IntoView, PosFn: Fn() -> Pos, Neg: IntoView, NegFn: Fn() -> Neg>
{
    test: Test,
    pos: PosFn,
    neg: NegFn,
    state: CondState,
}

impl<
        Test: Fn(&Rcx) -> bool,
        Pos: IntoView,
        PosFn: Fn() -> Pos,
        Neg: IntoView,
        NegFn: Fn() -> Neg,
    > Cond<Test, Pos, PosFn, Neg, NegFn>
{
    /// Construct a new conditional View.
    pub fn new(test: Test, pos: PosFn, neg: NegFn) -> Self {
        Self {
            test,
            pos,
            neg,
            state: CondState::Unset,
        }
    }

    fn build_branch_state<Result: IntoView, Factory: Fn() -> Result>(
        &self,
        branch: &Factory,
        parent: Entity,
        vc: &mut ViewContext,
    ) -> (ViewRef, Entity) {
        let state_entity = vc.world.spawn_empty().id();
        let state_view = (branch)().into_view();
        vc.world.entity_mut(state_entity).set_parent(parent);
        state_view.lock().unwrap().build(state_entity, vc);
        vc.world.entity_mut(parent).insert(DisplayNodeChanged);
        (state_view, state_entity)
    }
}

impl<
        Test: Fn(&Rcx) -> bool,
        Pos: IntoView,
        PosFn: Fn() -> Pos,
        Neg: IntoView,
        NegFn: Fn() -> Neg,
    > View for Cond<Test, Pos, PosFn, Neg, NegFn>
{
    fn nodes(&self) -> NodeSpan {
        match self.state {
            CondState::Unset => NodeSpan::Empty,
            CondState::True(ref true_state) => true_state.0.lock().unwrap().nodes(),
            CondState::False(ref false_state) => false_state.0.lock().unwrap().nodes(),
        }
    }

    fn build(&mut self, view_entity: Entity, vc: &mut crate::ViewContext) {
        let mut tracking = TrackingScope::new(vc.world.change_tick());
        self.react(view_entity, vc, &mut tracking);
        vc.world.entity_mut(view_entity).insert(tracking);
        assert!(
            vc.world.entity_mut(view_entity).get::<Parent>().is_some(),
            "Cond should have a parent view"
        );
    }

    fn react(&mut self, view_entity: Entity, vc: &mut ViewContext, tracking: &mut TrackingScope) {
        let re = Rcx::new(vc.world, tracking);
        let cond = (self.test)(&re);
        if cond {
            match self.state {
                CondState::True(_) => {
                    // Already true, do nothing.
                }
                CondState::False((ref mut false_state, entity)) => {
                    false_state.lock().unwrap().raze(entity, vc.world);
                    self.state = CondState::True(self.build_branch_state::<Pos, PosFn>(
                        &self.pos,
                        view_entity,
                        vc,
                    ));
                }
                CondState::Unset => {
                    self.state = CondState::True(self.build_branch_state::<Pos, PosFn>(
                        &self.pos,
                        view_entity,
                        vc,
                    ));
                }
            }
        } else {
            match self.state {
                CondState::False(_) => {
                    // Already false, do nothing.
                }
                CondState::True((ref mut true_state, entity)) => {
                    true_state.lock().unwrap().raze(entity, vc.world);
                    self.state = CondState::False(self.build_branch_state::<Neg, NegFn>(
                        &self.neg,
                        view_entity,
                        vc,
                    ));
                }
                CondState::Unset => {
                    self.state = CondState::False(self.build_branch_state::<Neg, NegFn>(
                        &self.neg,
                        view_entity,
                        vc,
                    ));
                }
            }
        }
    }

    fn raze(&mut self, view_entity: Entity, world: &mut World) {
        match self.state {
            CondState::True((ref mut true_state, entity)) => {
                true_state.lock().unwrap().raze(entity, world)
            }
            CondState::False((ref mut false_state, entity)) => {
                false_state.lock().unwrap().raze(entity, world)
            }
            CondState::Unset => {}
        }
        world.despawn_owned_recursive(view_entity);
    }
}

/// Creates a conditional branch view.
pub fn cond<
    Test: Send + Sync + Fn(&Rcx) -> bool,
    Pos: 'static + IntoView,
    PosFn: Send + Sync + 'static + Fn() -> Pos,
    Neg: 'static + IntoView,
    NegFn: Send + Sync + 'static + Fn() -> Neg,
>(
    test: Test,
    pos: PosFn,
    neg: NegFn,
) -> Cond<Test, Pos, PosFn, Neg, NegFn> {
    Cond::new(test, pos, neg)
}

impl<
        Test: Send + Sync + Fn(&Rcx) -> bool,
        Pos: 'static + IntoView,
        PosFn: Send + Sync + 'static + Fn() -> Pos,
        Neg: 'static + IntoView,
        NegFn: Send + Sync + 'static + Fn() -> Neg,
    > IntoView for Cond<Test, Pos, PosFn, Neg, NegFn>
{
    fn into_view(self) -> ViewRef {
        Arc::new(Mutex::new(self))
    }
}