#[macro_export]
macro_rules! relation_many {
	(
        $(#[$outer_1:meta])*
		$vis_1:vis struct $ident_1:ident($(#[$inner_1:meta])* $hashy_1:ty);

        $(#[$outer_2:meta])*
		$vis_2:vis struct $ident_2:ident($(#[$inner_2:meta])* $hashy_2:ty);
    ) => {
        $(#[$outer_1])*
		#[derive(Component, Default, Clone, Deref, MapEntities)]
		#[component(immutable, on_remove = $ident_1::on_remove)]
		$vis_1 struct $ident_1(#[entities] $(#[$inner_1])* $hashy_1);

        $(#[$outer_2])*
		#[derive(Component, Default, Clone, Deref, MapEntities)]
		#[component(immutable, on_remove = $ident_2::on_remove)]
		$vis_2 struct $ident_2(#[entities] $(#[$inner_2])* $hashy_2);

		out_entity_set!($ident_1);
		out_entity_set!($ident_2);

		impl $ident_1 {
			fn on_remove(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
				let ent_mut = world.entity(entity);
				let mut mod_notif = ent_mut.get::<$ident_1>().cloned().unwrap().entity_set();
				mod_notif.drain().for_each(|entity_notif| {
					world
						.commands()
						.queue(RemoveMod::<$ident_1, $ident_2>::new(entity, entity_notif));
				});
			}
		}

		impl $ident_2 {
			fn on_remove(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
				let ent_mut = world.entity(entity);
				let mut mod_notif = ent_mut.get::<$ident_2>().cloned().unwrap().entity_set();
				mod_notif.drain().for_each(|entity_mod| {
					world
						.commands()
						.queue(RemoveMod::<$ident_1, $ident_2>::new(entity_mod, entity));
				});
			}
		}

		impl ModExtent for $ident_1 {
			fn add_it(cmd: &mut Commands, ent_1: Entity, ent_2: Entity) {
				cmd.queue(ShareMod::<$ident_1, $ident_2>::new(ent_2, ent_1));
			}
			fn remove_it(cmd: &mut Commands, ent_1: Entity, ent_2: Entity) {
				cmd.queue(RemoveMod::<$ident_1, $ident_2>::new(ent_2, ent_1));
			}
		}

		impl ModExtent for $ident_2 {
			fn add_it(cmd: &mut Commands, ent_1: Entity, ent_2: Entity) {
				cmd.queue(ShareMod::<$ident_1, $ident_2>::new(ent_1, ent_2));
			}
			fn remove_it(cmd: &mut Commands, ent_1: Entity, ent_2: Entity) {
				cmd.queue(RemoveMod::<$ident_1, $ident_2>::new(ent_1, ent_2));
			}
		}
	};
}

#[macro_export]
macro_rules! out_entity_set {
	($struct_name:ident) => {
		impl Many2Many for $struct_name {
			fn new_self(entity_set: EntityHashSet) -> Self {
				Self(entity_set)
			}

			fn entity_set(self) -> EntityHashSet {
				self.0
			}
		}
	};
}
