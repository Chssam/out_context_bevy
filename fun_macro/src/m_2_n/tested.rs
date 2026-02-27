#[cfg(test)]
#[allow(unused)]
mod tests {
	use bevy_app::prelude::*;
	use bevy_derive::*;
	use bevy_ecs::{
		entity::{EntityHashSet, MapEntities},
		lifecycle::HookContext,
		prelude::*,
		world::DeferredWorld,
	};
	use bevy_reflect::Reflect;

	use crate::*;

	// Many to Many Micro used
	relation_many! {
		pub struct ModNotif(EntityHashSet);

		pub struct ModGetNotif(EntityHashSet);
	}

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

		fn setup<T: Many2Many, U: Many2Many>(world: &mut World) {
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
			world.commands().entity(modder).add_mod::<ModGetNotif>(get_mod);
		}

		fn check(one_mod: Single<(Entity, &ModNotif)>, one_get_mod: Single<(Entity, &ModGetNotif)>, mut cmd: Commands) {
			let mod_has_get = one_mod.1.contains(&one_get_mod.0);
			let get_has_mod = one_get_mod.1.contains(&one_mod.0);

			assert_eq!(one_mod.0, Entity::from_raw_u32(0).unwrap());
			assert_eq!(one_get_mod.0, Entity::from_raw_u32(1).unwrap());
			assert!(mod_has_get);
			assert!(get_has_mod);

			cmd.entity(one_get_mod.0).remove_mod::<ModNotif>(one_mod.0);
		}

		fn check_empty(one_mod: Query<&ModNotif>, one_get_mod: Query<&ModGetNotif>) {
			assert!(one_mod.is_empty());
			assert!(one_get_mod.is_empty());
		}
	}
}
