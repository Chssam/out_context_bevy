#![allow(unused)]
use std::ptr::NonNull;

use bevy_app::prelude::*;
use bevy_derive::*;
use bevy_ecs::{
	entity_disabling::Disabled, lifecycle::HookContext, prelude::*, world::DeferredWorld,
};
use bevy_log::prelude::*;
use bevy_platform::{collections::HashMap, prelude::*};
use bevy_ptr::OwningPtr;
use smol_str::SmolStr;

#[derive(Default)]
pub struct UniquePlugin;
impl Plugin for UniquePlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<UniqueHashed>()
			.add_systems(Update, delete_unique);
	}
}

/// Search by name, Entity are [UniqueEntity] and [UniqueName] with [Disabled]
#[derive(Resource, Default, Debug, Deref)]
pub struct UniqueHashed(HashMap<UniqueName, Entity>);

impl UniqueHashed {
	pub fn get_ent(&self, name: &str) -> Option<Entity> {
		self.get(&UniqueName::new(name)).cloned()
	}
}

#[derive(Component, Clone, Debug, Eq, Hash, PartialEq)]
#[require(UniqueEntity)]
#[component(immutable)]
pub struct UniqueName(pub SmolStr);

impl UniqueName {
	pub fn new(token: &str) -> Self {
		Self(SmolStr::new(token))
	}
}

#[derive(Component)]
struct UniqueDuplicateDespawn;

/// Must insert with [Name], technique used to make modding easy, maybe.
///
/// Contain any component used to clone.
#[derive(Component, Default)]
#[require(Disabled)]
#[component(on_add = UniqueEntity::on_add, on_remove = UniqueEntity::on_remove)]
pub struct UniqueEntity;

impl UniqueEntity {
	fn on_add(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
		let Some(name) = world.entity(entity).get::<UniqueName>().cloned() else {
			warn!("Adding Unique Component without Name: {}", entity);
			world.commands().entity(entity).despawn();
			return;
		};
		let mut hashed = world.resource_mut::<UniqueHashed>();
		match hashed.0.get(&name).cloned() {
			Some(ent_hashed) => {
				world.commands().queue(move |inner_world: &mut World| {
					let ent_ref = inner_world.entity(entity);
					let components = ent_ref
						.archetype()
						.iter_components()
						.filter_map(|component_id| {
							let get_ptr =
								ent_ref.get_by_id(component_id).map(|point| point.as_ptr());

							match get_ptr {
								Ok(ptr) => Some((component_id, ptr)),
								Err(err) => {
									warn!("Pointer failed: {:?}", err);
									None
								}
							}
						})
						.collect::<Vec<_>>();

					let mut ent_mut = inner_world.entity_mut(ent_hashed);
					for (component_id, ptr) in components {
						unsafe {
							let non_null = NonNull::new_unchecked(ptr.cast());
							let owned = OwningPtr::new(non_null);
							ent_mut.insert_by_id(component_id, owned);
						}
					}

					inner_world
						.commands()
						.entity(entity)
						.insert(UniqueDuplicateDespawn);
				});
			}
			None => {
				hashed.0.insert(name, entity);
			}
		};
	}

	fn on_remove(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
		let Some(name) = world.entity(entity).get::<UniqueName>().cloned() else {
			warn!("Removing Unique Component without Name: {}", entity);
			return;
		};
		let mut hashed = world.resource_mut::<UniqueHashed>();
		if let Some(ent_hashed) = hashed.get(&name) {
			if &entity == ent_hashed {
				hashed.0.remove(&name);
			}
		}
	}
}

fn delete_unique(
	query: Query<Entity, (With<UniqueDuplicateDespawn>, With<Disabled>)>,
	mut cmd: Commands,
) {
	query.iter().for_each(|entity| {
		cmd.entity(entity).despawn();
	});
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_unique() {
		let mut app = App::new();
		app.add_plugins(UniquePlugin)
			.add_systems(Startup, setup)
			.add_systems(Last, checkout);

		app.run();
	}

	#[derive(Component)]
	struct A(u8);
	#[derive(Component)]
	struct B(u8);
	#[derive(Component)]
	struct C(f32);

	fn setup(world: &mut World) {
		world.spawn((UniqueName::new("Test_1"), A(10)));
		world.spawn((UniqueName::new("Test_1"), B(5)));
		world.spawn((UniqueName::new("Test_1"), C(5.05)));

		world.spawn((UniqueName::new("Test_2"), A(100)));
		world.spawn((UniqueName::new("Test_2"), B(55)));
		world.spawn((UniqueName::new("Test_2"), C(56.25)));

		world.spawn((UniqueName::new("Test_3"), B(60)));

		let ent_1 = world.spawn((UniqueName::new("Test_4"), A(100))).id();
		let ent_2 = world.spawn((UniqueName::new("Test_5"), B(58))).id();

		world.commands().entity(ent_1).despawn();
		world.commands().entity(ent_2).despawn();

		let hashed = world.resource::<UniqueHashed>();
		assert_eq!(hashed.len(), 5);
	}

	fn checkout(world: &mut World) {
		let mut q_ent = world.query_filtered::<Entity, (With<UniqueEntity>, With<Disabled>)>();
		let out_ent = q_ent.iter(world);
		assert_eq!(out_ent.count(), 3);

		let mut q_b = world
			.query_filtered::<&B, (With<UniqueEntity>, With<Disabled>, Without<A>, Without<C>)>();
		let out_b = q_b.single(world).unwrap();
		assert_eq!(out_b.0, 60);

		let mut q_out = world.query_filtered::<(Entity, &A, &B, &C), With<Disabled>>();
		let out = q_out.iter(world);
		assert_eq!(out.count(), 2);

		let hashed = world.resource::<UniqueHashed>();
		assert_eq!(hashed.len(), 3);
	}
}
