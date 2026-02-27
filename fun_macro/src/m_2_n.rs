use std::marker::PhantomData;

use bevy_ecs::{
	component::Component,
	entity::{Entity, EntityHashSet},
	system::Command,
	world::World,
};

pub trait OutputEntity: Component + Default + Clone {
	fn new_self(entity_set: EntityHashSet) -> Self;
	fn entity_set(self) -> EntityHashSet;
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
				ent_cmd.insert(T::new_self(mod_notif));
			}
		}

		if let Some(mut get_notif) = world.get::<U>(self.entity).cloned().map(|v| v.entity_set()) {
			get_notif.remove(&self.moder);
			if let Ok(mut ent_cmd) = world.commands().get_entity(self.entity) {
				ent_cmd.insert(U::new_self(get_notif));
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
	m_2_n!(ModNotif, GetModNotif);

	/// Total [ModNotif] size equal to [GetModNotif]
	#[test]
	fn verify_total() {
		let mut app = App::new();
		app.add_systems(Startup, setup::<ModNotif, GetModNotif>)
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
		world.commands().queue(ShareMod::<T, U>::new(modder_2, ent_3)); // Mod will be despawned so not in GetModNotif
		world.commands().queue(ShareMod::<T, U>::new(modder_2, ent_6)); // Mod will be despawned so not in GetModNotif

		world.commands().queue(ShareMod::<T, U>::new(modder_0, ent_7)); // Entity 7 will be despawned so not in ModNotif

		world.commands().entity(ent_7).despawn();
		world.commands().entity(modder_2).despawn();
	}

	fn checkout(query_mod: Query<&ModNotif>, query_notif: Query<&GetModNotif>) {
		let total_mod = query_mod.iter().len();
		assert_eq!(total_mod, 2);

		let mut total_mod_to_notif = 0;
		for mod_notif in query_mod {
			total_mod_to_notif += mod_notif.iter().len();
		}
		assert_eq!(total_mod_to_notif, 4);

		let total_get = query_notif.iter().len();
		assert_eq!(total_get, 4);

		let mut total_get_notif = 0;
		for get_notif in query_notif {
			total_get_notif += get_notif.iter().len();
		}
		assert_eq!(total_get_notif, 4);
	}
}
