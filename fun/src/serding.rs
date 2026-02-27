#![allow(dead_code)]
use std::path::PathBuf;

use bevy_ecs::{error::Result, reflect::AppTypeRegistry};
use bevy_log::*;
use bevy_reflect::{
	FromReflect, PartialReflect,
	serde::{ReflectDeserializer, ReflectSerializer},
};
use serde::de::DeserializeSeed;

pub fn write_new_file(mut path: PathBuf, content: &[u8]) {
	if let Err(err) = std::fs::write(&path, content) {
		warn!(
			"Fail to save path: {:#?}\nPath: {:?}\nCreating directory.",
			err,
			path.file_name()
		);
		path.pop();
		if let Err(err) = std::fs::create_dir_all(path) {
			error!("Fail to create path: {:?}", err);
		};
	}
}

pub trait MoreReflect {
	// RON FORMAT --------------------------------------------------------------------------------
	fn save_assets_ron(&self, saving_settings: &dyn PartialReflect, path: PathBuf);
	fn read_into_typed_ron<T: PartialReflect + FromReflect>(&self, path: PathBuf) -> Result<T>;
	fn into_typed_ron<T: PartialReflect + FromReflect>(&self, data: &[u8]) -> Result<T>;
	// RON FORMAT --------------------------------------------------------------------------------

	// JSON FORMAT -------------------------------------------------------------------------------
	fn save_assets_json(&self, saving_settings: &dyn PartialReflect, path: PathBuf);
	fn read_into_typed_json<T: PartialReflect + FromReflect>(&self, path: PathBuf) -> Result<T>;
	fn into_typed_json<T: PartialReflect + FromReflect>(&self, data: &[u8]) -> Result<T>;
	// JSON FORMAT -------------------------------------------------------------------------------

	// BIN FORMAT --------------------------------------------------------------------------------
	fn save_assets_bin(&self, saving_settings: &dyn PartialReflect, path: PathBuf) -> Result;
	fn read_into_typed_bin<T: PartialReflect + FromReflect>(&self, path: PathBuf) -> Result<T>;
	fn into_typed_bin<T: PartialReflect + FromReflect>(&self, data: &[u8]) -> Result<T>;
	// BIN FORMAT --------------------------------------------------------------------------------
}

impl MoreReflect for AppTypeRegistry {
	// RON FORMAT --------------------------------------------------------------------------------
	fn save_assets_ron(&self, saving_settings: &dyn PartialReflect, mut path: PathBuf) {
		path.set_extension("ron");
		let type_registry = self.read();

		let serializer = ReflectSerializer::new(saving_settings, &type_registry);
		let pretty = ron::ser::PrettyConfig::new();

		let prepare_well = match ron::ser::to_string_pretty(&serializer, pretty) {
			Ok(ready) => ready,
			Err(err) => {
				return warn!("FAILED TO CONVERT: {:#?}", err);
			}
		};

		write_new_file(path, prepare_well.as_bytes());
	}

	fn read_into_typed_ron<T: PartialReflect + FromReflect>(&self, mut path: PathBuf) -> Result<T> {
		path.set_extension("ron");
		let data = std::fs::read(path)?;
		self.into_typed_ron::<T>(&data)
	}

	fn into_typed_ron<T: PartialReflect + FromReflect>(&self, data: &[u8]) -> Result<T> {
		let mut value = ron::Deserializer::from_bytes(data)?;
		let type_registry = self.read();
		let deserializer = ReflectDeserializer::new(&type_registry);
		let reflect_value = deserializer.deserialize(&mut value)?;
		let reflected_type = <T as FromReflect>::from_reflect(&*reflect_value);
		reflected_type.ok_or("Unable to FromReflect".into())
	}
	// RON FORMAT --------------------------------------------------------------------------------

	// JSON FORMAT -------------------------------------------------------------------------------
	fn save_assets_json(&self, saving_settings: &dyn PartialReflect, mut path: PathBuf) {
		path.set_extension("json");
		let type_registry = self.read();

		let serializer = ReflectSerializer::new(saving_settings, &type_registry);

		let prepare_well = match serde_json::ser::to_vec(&serializer) {
			Ok(ready) => ready,
			Err(err) => {
				return warn!("FAILED TO CONVERT: {:#?}", err);
			}
		};

		write_new_file(path, &prepare_well);
	}

	fn read_into_typed_json<T: PartialReflect + FromReflect>(
		&self,
		mut path: PathBuf,
	) -> Result<T> {
		path.set_extension("json");
		let data = std::fs::read(path)?;
		self.into_typed_json::<T>(&data)
	}

	fn into_typed_json<T: PartialReflect + FromReflect>(&self, data: &[u8]) -> Result<T> {
		let value: serde_json::Value = serde_json::from_slice(data)?;
		let type_registry = self.read();
		let deserializer = ReflectDeserializer::new(&type_registry);
		let reflect_value = deserializer.deserialize(value)?;
		let reflected_type = <T as FromReflect>::from_reflect(&*reflect_value);
		reflected_type.ok_or("Unable to FromReflect".into())
	}
	// JSON FORMAT -------------------------------------------------------------------------------

	// BIN FORMAT --------------------------------------------------------------------------------
	fn save_assets_bin(&self, saving_settings: &dyn PartialReflect, mut path: PathBuf) -> Result {
		path.set_extension("bin");
		let type_registry = self.read();

		let serializer = ReflectSerializer::new(saving_settings, &type_registry);
		let prepare_well = bincode::serde::encode_to_vec(serializer, bincode::config::standard())?;

		write_new_file(path, &prepare_well);
		Ok(())
	}

	fn read_into_typed_bin<T: PartialReflect + FromReflect>(&self, mut path: PathBuf) -> Result<T> {
		path.set_extension("bin");
		let data = std::fs::read(path)?;
		self.into_typed_bin::<T>(&data)
	}

	fn into_typed_bin<T: PartialReflect + FromReflect>(&self, data: &[u8]) -> Result<T> {
		let type_registry = self.read();

		let deserializer = ReflectDeserializer::new(&type_registry);
		let (reflect_value, _) = bincode::serde::seed_decode_from_slice(
			deserializer,
			data,
			bincode::config::standard(),
		)?;
		let reflected_type = <T as FromReflect>::from_reflect(&*reflect_value);
		reflected_type.ok_or("Unable to FromReflect".into())
	}
	// BIN FORMAT --------------------------------------------------------------------------------
}
