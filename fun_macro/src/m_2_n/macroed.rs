#[macro_export]
macro_rules! m2n {
	($struct_1:ident, $struct_2:ident) => {
		out_entity_set!($struct_1);
		out_entity_set!($struct_2);

		impl $struct_1 {
			fn on_remove(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
				let ent_mut = world.entity(entity);
				let mut mod_notif = ent_mut.get::<$struct_1>().cloned().unwrap().entity_set();
				mod_notif.drain().for_each(|entity_notif| {
					world
						.commands()
						.queue(RemoveMod::<$struct_1, $struct_2>::new(entity, entity_notif));
				});
			}
		}

		impl $struct_2 {
			fn on_remove(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
				let ent_mut = world.entity(entity);
				let mut mod_notif = ent_mut.get::<$struct_2>().cloned().unwrap().entity_set();
				mod_notif.drain().for_each(|entity_mod| {
					world
						.commands()
						.queue(RemoveMod::<$struct_1, $struct_2>::new(entity_mod, entity));
				});
			}
		}

		impl ModExtent for $struct_1 {
			fn add_it(cmd: &mut Commands, ent_1: Entity, ent_2: Entity) {
				cmd.queue(ShareMod::<$struct_1, $struct_2>::new(ent_1, ent_2));
			}
			fn remove_it(cmd: &mut Commands, ent_1: Entity, ent_2: Entity) {
				cmd.queue(RemoveMod::<$struct_1, $struct_2>::new(ent_1, ent_2));
			}
		}

		impl ModExtent for $struct_2 {
			fn add_it(cmd: &mut Commands, ent_1: Entity, ent_2: Entity) {
				cmd.queue(ShareMod::<$struct_1, $struct_2>::new(ent_2, ent_1));
			}
			fn remove_it(cmd: &mut Commands, ent_1: Entity, ent_2: Entity) {
				cmd.queue(RemoveMod::<$struct_1, $struct_2>::new(ent_2, ent_1));
			}
		}
	};
}

#[macro_export]
macro_rules! out_entity_set {
	($struct_name:ident) => {
		#[derive(Reflect, Component, Default, Clone, Deref, MapEntities)]
		#[component(immutable, on_remove = $struct_name::on_remove)]
		pub struct $struct_name(#[entities] EntityHashSet);

		impl OutputEntity for $struct_name {
			fn new_self(entity_set: EntityHashSet) -> Self {
				Self(entity_set)
			}

			fn entity_set(self) -> EntityHashSet {
				self.0
			}
		}
	};
}
