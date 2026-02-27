use ::serde::{Deserialize, Serialize, ser::SerializeSeq};
use bevy_app::{ScheduleRunnerPlugin, prelude::*};
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::{
	component::ComponentId, entity::EntityHashSet, lifecycle::HookContext, prelude::*,
	world::DeferredWorld,
};
use bevy_log::{LogPlugin, prelude::*};
use bevy_reflect::*;
use bevy_utils::default;
use ron::{
	Value,
	ser::{PrettyConfig, to_string_pretty},
};
use smol_str::{SmolStr, ToSmolStr};

use crate::{
	many_2_many::RemoveMod,
	recurer::DynamicSerde as _,
	serding::{MoreReflect, write_new_file},
};

mod many_2_many;
mod recurer;
mod serding;
mod unique;

use reflection_fun::*;

fn main() -> AppExit {
	let mut app = App::new();

	// app.add_plugins(ScheduleRunnerPlugin::default());

	app.add_plugins(LogPlugin::default())
		.add_systems(Startup, (ron_only, something));

	app.run()
}

#[derive(Bundle)]
pub struct ModBundle {
	pub component_id: ModID,
	pub value: AAA,
}

#[derive(Component)]
pub struct ModID(ComponentId);

#[derive(Component, Serialize, Deserialize, Clone, Deref, DerefMut)]
pub struct AAA(pub Value);

pub const THE_COMPONENT: &str = "Hello";

#[derive(Component, Debug, Deref, DerefMut)]
pub struct TheMod(Box<dyn PartialReflect>);

#[derive(Reflect, Serialize, Deserialize)]
struct Something {
	pub name: SmolStr,
	#[serde(rename = "type")]
	pub typed: SmolStr,
	#[reflect(ignore)]
	pub value: Option<Value>,
}

#[derive(Component)]
#[relationship_target(relationship = ModOf, linked_spawn)]
struct Moderes(Vec<Entity>);

/// To String
#[derive(Component)]
#[relationship(relationship_target = Moderes)]
struct ModOf(pub Entity);

pub fn ron_only() -> Result<()> {
	let smh = Something {
		name: "Orang".to_smolstr(),
		typed: String::new().reflect_type_path().to_smolstr(),
		value: Some(String::new().into()),
	};

	let val = Value::new_serde(smh.as_partial_reflect());

	let content = to_string_pretty(&val, PrettyConfig::new())?;
	write_new_file("Smh.ron".into(), content.as_bytes());

	// let valued: Value = from_str(&content)?;

	// let Value::Seq(v) = valued else {
	// 	return Err("Nope".into());
	// };

	// for mapped in v {
	// 	let Value::Map(mapy) = mapped else {
	// 		return Err("Noped".into());
	// 	};
	// 	for (key, valued) in mapy.into_iter() {
	// 		if key.eq(&"name".into()) {}
	// 	}
	// }

	Ok(())
}

#[derive(Reflect)]
enum Yapper {
	Yes,
	No,
}

#[derive(Reflect, Default)]
struct Opnes {
	one: u8,
	two: u16,
}

fn something(mut cmd: Commands, registry: Res<AppTypeRegistry>) -> Result<()> {
	let mut hashed = DynamicMap::default();
	hashed.insert(5, 5);
	hashed.insert(10, 8);
	let mut dyn_struct_in = DynamicStruct::default();
	dyn_struct_in.insert("inted", 5i32);
	dyn_struct_in.insert("floaty", 10.5f32);
	dyn_struct_in.insert("tuple", (5, 6, 7, 8, 9));
	dyn_struct_in.insert("tuple", (5, 6, 7.5, 8, "Epic Asia".to_string()));
	dyn_struct_in.insert("array", [5, 6, 7, 8, 9]);
	dyn_struct_in.insert("deep", Some((Opnes::default(), 8.55, Opnes::default())));
	let mut dyn_struct = DynamicStruct::default();
	dyn_struct.insert("name", "MyName".to_string());
	dyn_struct.insert("value", 105i32);
	dyn_struct.insert("sadess", -125i32);
	dyn_struct.insert("structed", dyn_struct_in);
	dyn_struct.insert("enumed", Yapper::Yes);
	dyn_struct.insert("no_more", Option::<u8>::None);
	dyn_struct.insert("have", Some(10.5621));
	dyn_struct.insert("multiple", Some((959, 95.5, Some("nopenope".to_string()))));
	dyn_struct.insert("hashed", hashed);
	dyn_struct.insert("two", (5, 6, 5.7f32));

	let hashed_ron = ron::Value::Seq(vec![ron::Value::new_serde(dyn_struct.as_partial_reflect())]);

	let well_ron = ron::ser::to_string_pretty(&hashed_ron, default())?;
	write_new_file("DynRon.ron".into(), well_ron.as_bytes());

	let de_serialize: ron::Value = ron::from_str(&well_ron)?;
	let outlet = format!("{:#?}", de_serialize);
	write_new_file("Desed.ron".into(), outlet.as_bytes());

	let well_ron = ron::ser::to_string_pretty(&de_serialize, default())?;
	write_new_file("DynRonNew.ron".into(), well_ron.as_bytes());

	Ok(())
}
