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
