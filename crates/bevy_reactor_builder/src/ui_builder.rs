use bevy::{
    prelude::{
        BuildChildren, Bundle, Component, Entity, EntityWorldMut, In, IntoSystem, Parent, World,
    },
    ui::GhostNode,
};
use bevy_reactor_signals::{
    create_derived, create_mutable, Callback, CallbackOwner, Ecx, Mutable, Rcx, Reaction,
    ReactionCell, Signal, TrackingScope,
};

pub struct UiBuilder<'w> {
    /// Bevy World
    world: &'w mut World,

    /// The entity that will be the parent of all of the children and other resources created
    /// in this scope.
    parent: Entity,
}

impl<'w> UiBuilder<'w> {
    /// Construct a new reactive context.
    pub fn new(world: &'w mut World, owner: Entity) -> Self {
        Self {
            world,
            parent: owner,
        }
    }

    /// Access to world from reactive context.
    pub fn world(&self) -> &World {
        self.world
    }

    /// Access to mutable world from reactive context.
    pub fn world_mut(&mut self) -> &mut World {
        self.world
    }

    /// Returns the parent entity
    pub fn parent(&self) -> Entity {
        self.parent
    }

    /// Spawn a new child of the parent entity with the given bundle.
    pub fn spawn(&mut self, bundle: impl Bundle) -> EntityWorldMut {
        let mut ent = self.world.spawn(bundle);
        ent.set_parent(self.parent);
        ent
    }

    /// Spawn a new, empty child of the parent entity.
    pub fn spawn_empty(&mut self) -> EntityWorldMut {
        let mut ent = self.world.spawn_empty();
        ent.set_parent(self.parent);
        ent
    }

    /// Return an `EntityWorldMut` for the given entity.
    pub fn entity_mut(&mut self, entity: Entity) -> EntityWorldMut<'_> {
        self.world.entity_mut(entity)
    }

    /// Create a new callback which is owned by the parent entity.
    pub fn create_callback<P: Send, M, S: IntoSystem<In<P>, (), M> + 'static>(
        &mut self,
        callback: S,
    ) -> Callback<P> {
        let id = self.world_mut().register_system(callback);
        let result = Callback::new(id);
        let parent = self.parent();
        match self.world.get_mut::<CallbackOwner>(parent) {
            Some(mut owner) => {
                owner.add(result);
            }
            None => {
                let mut owner = CallbackOwner::new();
                owner.add(result);
                self.world.entity_mut(parent).insert(owner);
            }
        }
        result
    }

    /// Create a new [`Mutable`] in this context.
    pub fn create_mutable<T>(&mut self, init: T) -> Mutable<T>
    where
        T: Send + Sync + 'static,
    {
        create_mutable(self.world, self.parent, init)
    }

    /// Create a new [`Derived`] in this context. This represents a readable signal which
    /// is computed from other signals. The result is not memoized, but is recomputed whenever
    /// the dependencies change.
    ///
    /// Arguments:
    /// * `compute` - The function that computes the output. This will be called with a single
    ///    parameter, which is an [`Rcx`] object.
    pub fn create_derived<R: 'static, F: Send + Sync + 'static + Fn(&mut Rcx) -> R>(
        &mut self,
        compute: F,
    ) -> Signal<R> {
        let derived = create_derived(self.world, compute);
        self.world.entity_mut(self.parent).add_child(derived.id());
        Signal::Derived(derived)
    }

    /// Create a reactive effect which is owned by the parent entity.
    pub fn create_effect<F: Send + Sync + 'static + FnMut(&mut Ecx)>(
        &mut self,
        effect: F,
    ) -> &mut Self {
        let mut scope = TrackingScope::new(self.world().last_change_tick());
        let mut reaction = EffectReaction { effect };
        let owner = self.parent;
        let effect_owner = self.world.spawn_empty().set_parent(owner).id();
        reaction.react(effect_owner, self.world, &mut scope);
        self.world
            .entity_mut(effect_owner)
            .insert((scope, ReactionCell::new(reaction), GhostNode));
        self
    }

    /// Return a reference to the Component `C` on the owner entity of the current
    /// context, or one of it's ancestors. This searches up the entity tree until it finds
    /// a component of the given type.
    pub fn use_inherited_component<C: Component>(&self) -> Option<&C> {
        let mut entity = self.parent;
        loop {
            let ec = self.world.entity(entity).get::<C>();
            if ec.is_some() {
                return ec;
            }
            match self.world.entity(entity).get::<Parent>() {
                Some(parent) => entity = **parent,
                _ => return None,
            }
        }
    }
}

pub trait CreateChilden {
    fn create_children(&mut self, spawn_children: impl FnOnce(&mut UiBuilder)) -> &mut Self;
    fn create_children_mut(&mut self, spawn_children: impl FnMut(&mut UiBuilder)) -> &mut Self;
}

impl<'w> CreateChilden for EntityWorldMut<'w> {
    fn create_children(&mut self, spawn_children: impl FnOnce(&mut UiBuilder)) -> &mut Self {
        let parent = self.id();
        self.world_scope(|world| {
            spawn_children(&mut UiBuilder { world, parent });
        });
        self
    }

    fn create_children_mut(&mut self, mut spawn_children: impl FnMut(&mut UiBuilder)) -> &mut Self {
        let parent = self.id();
        self.world_scope(|world| {
            spawn_children(&mut UiBuilder { world, parent });
        });
        self
    }
}

/// General effect reaction.
pub struct EffectReaction<F: FnMut(&mut Ecx)> {
    effect: F,
}

impl<F: FnMut(&mut Ecx)> Reaction for EffectReaction<F> {
    fn react(&mut self, owner: Entity, world: &mut World, tracking: &mut TrackingScope) {
        let mut ecx = Ecx::new(world, owner, tracking);
        (self.effect)(&mut ecx);
    }
}
