use std::marker::PhantomData;

use bevy_ecs::{
	component::Component,
	entity::{Entity, EntityHashSet},
	system::{Command, Commands, EntityCommands},
	world::World,
};

pub trait OutputEntity: Component + Default + Clone {
	fn new_self(entity_set: EntityHashSet) -> Self;
	fn entity_set(self) -> EntityHashSet;
}

pub trait Many2Many {
	fn add_mod<T: ModExtent>(&mut self, child: Entity);
	fn remove_mod<T: ModExtent>(&mut self, child: Entity);
}

impl<'a> Many2Many for EntityCommands<'a> {
	fn add_mod<T: ModExtent>(&mut self, child: Entity) {
		let ent = self.id();
		T::add_it(self.commands_mut(), ent, child);
	}
	fn remove_mod<T: ModExtent>(&mut self, child: Entity) {
		let ent = self.id();
		T::remove_it(self.commands_mut(), ent, child);
	}
}

pub trait ModExtent {
	fn add_it(cmd: &mut Commands, ent_1: Entity, ent_2: Entity);
	fn remove_it(cmd: &mut Commands, ent_1: Entity, ent_2: Entity);
}

pub struct ShareMod<T: OutputEntity, U: OutputEntity> {
	type_moder: PhantomData<T>,
	type_entity: PhantomData<U>,
	moder: Entity,
	entity: Entity,
}

impl<T: OutputEntity, U: OutputEntity> ShareMod<T, U> {
	pub fn new(moder: Entity, entity: Entity) -> Self {
		Self {
			type_moder: PhantomData::default(),
			type_entity: PhantomData::default(),
			moder,
			entity,
		}
	}
}

impl<T: OutputEntity, U: OutputEntity> Command for ShareMod<T, U> {
	fn apply(self, world: &mut World) {
		{
			let mut mod_notif = world.get::<T>(self.moder).cloned().unwrap_or_default().entity_set();

			mod_notif.insert(self.entity);
			world.commands().entity(self.moder).insert(T::new_self(mod_notif));
		}

		{
			let mut get_notif = world.get::<U>(self.entity).cloned().unwrap_or_default().entity_set();

			get_notif.insert(self.moder);
			world.commands().entity(self.entity).insert(U::new_self(get_notif));
		}
	}
}

pub struct RemoveMod<T: OutputEntity, U: OutputEntity> {
	type_moder: PhantomData<T>,
	type_entity: PhantomData<U>,
	moder: Entity,
	entity: Entity,
}

impl<T: OutputEntity, U: OutputEntity> RemoveMod<T, U> {
	pub fn new(moder: Entity, entity: Entity) -> Self {
		Self {
			type_moder: PhantomData::default(),
			type_entity: PhantomData::default(),
			moder,
			entity,
		}
	}
}

impl<T: OutputEntity, U: OutputEntity> Command for RemoveMod<T, U> {
	fn apply(self, world: &mut World) {
		if let Some(mut mod_notif) = world.get::<T>(self.moder).cloned().map(|v| v.entity_set()) {
			mod_notif.remove(&self.entity);
			if let Ok(mut ent_cmd) = world.commands().get_entity(self.moder) {
				if mod_notif.is_empty() {
					ent_cmd.remove::<T>();
				} else {
					ent_cmd.insert(T::new_self(mod_notif));
				}
			}
		}

		if let Some(mut get_notif) = world.get::<U>(self.entity).cloned().map(|v| v.entity_set()) {
			get_notif.remove(&self.moder);

			if let Ok(mut ent_cmd) = world.commands().get_entity(self.entity) {
				if get_notif.is_empty() {
					ent_cmd.remove::<U>();
				} else {
					ent_cmd.insert(U::new_self(get_notif));
				}
			}
		}
	}
}

#[macro_export]
macro_rules! m_2_n {
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

#[cfg(test)]
#[allow(unused)]
mod tests {
	use super::*;

	use bevy_app::prelude::*;
	use bevy_derive::*;
	use bevy_ecs::{
		entity::{EntityHashSet, MapEntities},
		lifecycle::HookContext,
		prelude::*,
		world::DeferredWorld,
	};
	use bevy_reflect::Reflect;

	// Many to Many Micro used
	m_2_n!(ModNotif, ModGetNotif);

	mod total {
		use super::*;

		/// Total [ModNotif] size equal to [ModGetNotif]
		#[test]
		fn verify_total() {
			let mut app = App::new();
			app.add_systems(Startup, setup::<ModNotif, ModGetNotif>)
				.add_systems(Last, checkout);

			app.run();
		}

		fn setup<T: OutputEntity, U: OutputEntity>(world: &mut World) {
			let modder_0 = world.spawn_empty().id();
			let modder_1 = world.spawn_empty().id();
			let modder_2 = world.spawn_empty().id();
			let ent_3 = world.spawn_empty().id();
			let ent_4 = world.spawn_empty().id();
			let ent_5 = world.spawn_empty().id();
			let ent_6 = world.spawn_empty().id();
			let ent_7 = world.spawn_empty().id();

			// Total valid 5 | 1 Duplicate
			world.commands().queue(ShareMod::<T, U>::new(modder_0, ent_3)); // Valid
			world.commands().queue(ShareMod::<T, U>::new(modder_1, ent_3)); // Valid
			world.commands().queue(ShareMod::<T, U>::new(modder_0, ent_4)); // Valid
			world.commands().queue(ShareMod::<T, U>::new(modder_0, ent_4)); // Duplicate
			world.commands().queue(ShareMod::<T, U>::new(modder_0, ent_5)); // Valid
			world.commands().queue(ShareMod::<T, U>::new(modder_1, ent_6)); // Valid

			// Remove 2 but only 1 is matched, so total valid now 4
			world.commands().queue(RemoveMod::<T, U>::new(modder_0, ent_4)); // Valid
			world.commands().queue(RemoveMod::<T, U>::new(modder_0, ent_6)); // Unmatched

			// Below here all not count toward
			world.commands().queue(ShareMod::<T, U>::new(modder_2, ent_3)); // Mod will be despawned so not in ModGetNotif
			world.commands().queue(ShareMod::<T, U>::new(modder_2, ent_6)); // Mod will be despawned so not in ModGetNotif

			world.commands().queue(ShareMod::<T, U>::new(modder_0, ent_7)); // Entity 7 will be despawned so not in ModNotif

			world.commands().entity(ent_7).despawn();
			world.commands().entity(modder_2).despawn();
		}

		fn checkout(query_mod: Query<&ModNotif>, query_notif: Query<&ModGetNotif>) {
			let total_mod = query_mod.iter().len();
			assert_eq!(total_mod, 2);

			let mut total_mod_to_notif = 0;
			for mod_notif in query_mod {
				total_mod_to_notif += mod_notif.iter().len();
			}
			assert_eq!(total_mod_to_notif, 4);

			// Since ent_4 is empty, [ModGetNotif] will be removed, total will be 3
			let total_get = query_notif.iter().len();
			assert_eq!(total_get, 3);

			let mut total_get_notif = 0;
			for get_notif in query_notif {
				total_get_notif += get_notif.iter().len();
			}
			assert_eq!(total_get_notif, 4);
		}
	}

	mod text_extend {
		use super::*;

		#[test]
		fn test_extent_mod() {
			let mut app = App::new();
			app.add_systems(PreStartup, setup);
			app.add_systems(Startup, check);
			app.add_systems(PostStartup, check_empty);
			app.run();
		}

		fn setup(world: &mut World) {
			let modder = world.spawn_empty().id();
			let get_mod = world.spawn_empty().id();
			world.commands().entity(modder).add_mod::<ModNotif>(get_mod);
		}

		fn check(one_mod: Single<(Entity, &ModNotif)>, one_get_mod: Single<(Entity, &ModGetNotif)>, mut cmd: Commands) {
			let mod_has_get = one_mod.1.contains(&one_get_mod.0);
			let get_has_mod = one_get_mod.1.contains(&one_mod.0);

			assert_eq!(one_mod.0, Entity::from_raw_u32(0).unwrap());
			assert_eq!(one_get_mod.0, Entity::from_raw_u32(1).unwrap());
			assert!(mod_has_get);
			assert!(get_has_mod);

			cmd.entity(one_get_mod.0).remove_mod::<ModGetNotif>(one_mod.0);
		}

		fn check_empty(one_mod: Query<&ModNotif>, one_get_mod: Query<&ModGetNotif>) {
			assert!(one_mod.is_empty());
			assert!(one_get_mod.is_empty());
		}
	}
}
