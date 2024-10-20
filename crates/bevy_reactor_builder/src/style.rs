use bevy::{
    prelude::{BuildChildren, Entity, EntityWorldMut, World},
    ui::{self, experimental::GhostNode},
};
use bevy_mod_stylebuilder::{StyleBuilder, StyleTuple};
use bevy_reactor_signals::{Rcx, Reaction, ReactionCell, TrackingScope};

pub trait EntityStyleBuilder {
    fn style<S: FnOnce(&mut StyleBuilder)>(&mut self, style: S) -> &mut Self;
    fn styles(&mut self, styles: impl StyleTuple) -> &mut Self;
    fn style_dyn<
        D: 'static,
        VF: Fn(&Rcx) -> D + Send + Sync + 'static,
        SF: Fn(D, &mut StyleBuilder) + Send + Sync + 'static,
    >(
        &mut self,
        deps_fn: VF,
        style: SF,
    ) -> &mut Self;
}

impl<'w> EntityStyleBuilder for EntityWorldMut<'w> {
    fn style<S: FnOnce(&mut StyleBuilder)>(&mut self, style_fn: S) -> &mut Self {
        let mut style = ui::Node::default();
        if let Some(s) = self.get::<ui::Node>() {
            style.clone_from(s);
        }
        let mut sb = StyleBuilder::new(self, style);
        style_fn(&mut sb);
        sb.finish();
        self
    }

    fn styles(&mut self, styles: impl StyleTuple) -> &mut Self {
        let mut style = ui::Node::default();
        if let Some(s) = self.get::<ui::Node>() {
            style.clone_from(s);
        }
        let mut sb = StyleBuilder::new(self, style);
        styles.apply(&mut sb);
        sb.finish();
        self
    }

    fn style_dyn<
        D: 'static,
        VF: Fn(&Rcx) -> D + Send + Sync + 'static,
        SF: Fn(D, &mut StyleBuilder) + Send + Sync + 'static,
    >(
        &mut self,
        deps_fn: VF,
        style_fn: SF,
    ) -> &mut Self {
        let mut scope = TrackingScope::new(self.world().last_change_tick());
        let reaction = DynamicStyleReaction {
            target: self.id(),
            deps_fn,
            style_fn,
        };
        let owner = self.id();
        self.world_scope(|world| {
            // Spawn a new reaction entity to contain the effect.
            let effect_owner = world.spawn_empty().set_parent(owner).id();
            reaction.apply(effect_owner, world, &mut scope);
            world.entity_mut(effect_owner).insert((
                scope,
                ReactionCell::new(reaction),
                GhostNode::default(),
            ));
        });
        self
    }
}

struct DynamicStyleReaction<D, VF: Fn(&Rcx) -> D, SF: Fn(D, &mut StyleBuilder)> {
    target: Entity,
    deps_fn: VF,
    style_fn: SF,
}

impl<
        D,
        VF: Fn(&Rcx) -> D + Send + Sync + 'static,
        SF: Fn(D, &mut StyleBuilder) + Send + Sync + 'static,
    > DynamicStyleReaction<D, VF, SF>
{
    fn apply(&self, _owner: Entity, world: &mut World, tracking: &mut TrackingScope) {
        let rcx = Rcx::new(world, self.target, tracking);
        let val = (self.deps_fn)(&rcx);

        let mut target = world.entity_mut(self.target);
        let mut style = ui::Node::default();
        if let Some(s) = target.get::<ui::Node>() {
            style.clone_from(s);
        }
        let mut sb = StyleBuilder::new(&mut target, style);
        (self.style_fn)(val, &mut sb);
        sb.finish();
    }
}

impl<
        D,
        VF: Fn(&Rcx) -> D + Send + Sync + 'static,
        SF: Fn(D, &mut StyleBuilder) + Send + Sync + 'static,
    > Reaction for DynamicStyleReaction<D, VF, SF>
{
    fn react(&mut self, owner: Entity, world: &mut World, tracking: &mut TrackingScope) {
        self.apply(owner, world, tracking);
    }
}
