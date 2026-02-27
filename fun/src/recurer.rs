use bevy_reflect::*;

pub trait DynamicSerde {
	fn new_serde(reflect: &dyn PartialReflect) -> Self;
	fn recursive(self, reflect: &dyn PartialReflect) -> Self;
	fn new_deserde(self) -> Box<dyn PartialReflect>;
	fn decursive(self, reflect: &mut dyn PartialReflect) -> Box<dyn PartialReflect>;
}

// pub struct HashedType(HashMap<SmolStr, Entity>);

mod construct_ron {
	use super::*;
	use ron::{Map, *};

	impl DynamicSerde for Value {
		fn new_serde(reflect: &dyn PartialReflect) -> Self {
			Value::Unit.recursive(reflect)
		}

		fn recursive(mut self, reflect: &dyn PartialReflect) -> Self {
			match reflect.reflect_ref() {
				ReflectRef::Struct(structed) => {
					let mut new_map = Map::new();
					for index in 0..structed.field_len() {
						let field_name = structed.name_at(index).unwrap().to_owned();
						let ref_value = structed.field_at(index).unwrap();
						new_map.insert(field_name, Value::new_serde(ref_value));
					}
					self = Value::Map(new_map);
				}
				ReflectRef::TupleStruct(tuple_struct) => {
					let mut new_map = Map::new();
					for index in 0..tuple_struct.field_len() {
						let ref_value = tuple_struct.field(index).unwrap();
						let numbered = Number::U8(index as u8);
						new_map.insert(numbered, Value::new_serde(ref_value));
					}
					self = Value::Map(new_map);
				}
				ReflectRef::Tuple(tuple) => {
					let mut new_map = Map::new();
					for index in 0..tuple.field_len() {
						let ref_value = tuple.field(index).unwrap();
						let numbered = Number::U8(index as u8);
						new_map.insert(numbered, Value::new_serde(ref_value));
					}
					self = Value::Map(new_map);
				}
				ReflectRef::List(list) => {
					let mut vec_value = Vec::with_capacity(list.len());
					for field in list.iter() {
						vec_value.push(Value::new_serde(field));
					}
					self = Value::Seq(vec_value);
				}
				ReflectRef::Array(array) => {
					let mut vec_value = Vec::with_capacity(array.len());
					for field in array.iter() {
						vec_value.push(Value::new_serde(field));
					}
					self = Value::Seq(vec_value);
				}
				ReflectRef::Map(map) => {
					let mut new_map = Map::new();
					for (field_key, field_value) in map.iter() {
						let k = Value::new_serde(field_key);
						let v = Value::new_serde(field_value);
						new_map.insert(k, v);
					}
					self = Value::Map(new_map);
				}
				ReflectRef::Set(set) => {
					let mut vec_value = Vec::with_capacity(set.len());
					for field in set.iter() {
						vec_value.push(Value::new_serde(field));
					}
					self = Value::Seq(vec_value);
				}
				// TODO! Incomeplete for many field struct and tuple
				ReflectRef::Enum(enumed) => {
					let var_name = enumed.variant_name();

					if var_name == "Some" {
						let mut new_map = Map::new();
						for index in 0..enumed.field_len() {
							let ref_value = enumed.field_at(index).unwrap();
							let numbered = Number::U8(index as u8);
							new_map.insert(numbered, Value::new_serde(ref_value));
						}

						let the_value = Value::Map(new_map);
						let the_some = Box::new(the_value);
						self = Value::Option(Some(the_some));
					} else if var_name == "None" {
						self = Value::Option(None);
					} else {
						match enumed.variant_type() {
							VariantType::Struct => {
								let mut new_map = Map::new();
								for index in 0..enumed.field_len() {
									let field_name = enumed.name_at(index).unwrap().to_owned();
									let ref_value = enumed.field_at(index).unwrap();
									new_map.insert(field_name, Value::new_serde(ref_value));
								}
								self = Value::Map(new_map);
							}
							VariantType::Tuple => {
								let mut new_map = Map::new();
								for index in 0..enumed.field_len() {
									let ref_value = enumed.field_at(index).unwrap();
									let numbered = Number::U8(index as u8);
									new_map.insert(numbered, Value::new_serde(ref_value));
								}
								self = Value::Map(new_map);
							}
							VariantType::Unit => self = var_name.into(),
						}
					}
				}
				ReflectRef::Opaque(partial_reflect) => {
					let output = format!("{:?}", partial_reflect);
					if let Ok(uinted) = output.parse::<u64>() {
						self = uinted.into();
					} else if let Ok(inted) = output.parse::<i64>() {
						self = inted.into();
					} else if let Ok(floated) = output.parse::<f64>() {
						self = floated.into();
					} else {
						self = output.trim_matches('\"').into();
					}
				}
				_ => unimplemented!("Ignore"),
			}

			self.clone()
		}

		fn new_deserde(self) -> Box<dyn PartialReflect> {
			let mut reflect = DynamicTuple::default();
			self.decursive(reflect.as_partial_reflect_mut())
		}

		fn decursive(self, reflect: &mut dyn PartialReflect) -> Box<dyn PartialReflect> {
			match self {
				Value::Bool(v) => todo!(),
				Value::Char(_) => todo!(),
				Value::Map(map) => todo!(),
				Value::Number(number) => todo!(),
				Value::Option(value) => todo!(),
				Value::String(_) => todo!(),
				Value::Bytes(items) => todo!(),
				Value::Seq(values) => todo!(),
				Value::Unit => todo!(),
			}
			todo!()
		}
	}
}

mod construct_json {
	use super::*;
	use serde_json::{Map, *};

	impl DynamicSerde for Value {
		fn new_serde(reflect: &dyn PartialReflect) -> Self {
			Self::Null.recursive(reflect)
		}

		fn recursive(mut self, reflect: &dyn PartialReflect) -> Value {
			match reflect.reflect_ref() {
				ReflectRef::Struct(structed) => {
					let mut new_map = Map::new();
					for index in 0..structed.field_len() {
						let field_name = structed.name_at(index).unwrap().to_owned();
						let ref_value = structed.field_at(index).unwrap();
						new_map.insert(field_name, Value::new_serde(ref_value));
					}
					self = Value::Object(new_map);
				}
				ReflectRef::TupleStruct(tuple_struct) => {
					let mut new_map = Map::new();
					for index in 0..tuple_struct.field_len() {
						let ref_value = tuple_struct.field(index).unwrap();
						new_map.insert(index.to_string(), Value::new_serde(ref_value));
					}
					self = Value::Object(new_map);
				}
				ReflectRef::Tuple(tuple) => {
					let mut new_map = Map::new();
					for index in 0..tuple.field_len() {
						let ref_value = tuple.field(index).unwrap();
						new_map.insert(index.to_string(), Value::new_serde(ref_value));
					}
					self = Value::Object(new_map);
				}
				ReflectRef::List(list) => {
					let mut vec_value = Vec::with_capacity(list.len());
					for field in list.iter() {
						vec_value.push(Value::new_serde(field));
					}
					self = Value::Array(vec_value);
				}
				ReflectRef::Array(array) => {
					let mut vec_value = Vec::with_capacity(array.len());
					for field in array.iter() {
						vec_value.push(Value::new_serde(field));
					}
					self = Value::Array(vec_value);
				}
				ReflectRef::Map(map) => {
					let mut new_map = Map::new();
					for (field_key, field_value) in map.iter() {
						let k = Value::new_serde(field_key);
						let v = Value::new_serde(field_value);
						new_map.insert(k.to_string(), v);
					}
					self = Value::Object(new_map);
				}
				ReflectRef::Set(set) => {
					let mut vec_value = Vec::with_capacity(set.len());
					for field in set.iter() {
						vec_value.push(Value::new_serde(field));
					}
					self = Value::Array(vec_value);
				}
				ReflectRef::Enum(enumed) => {
					let var_name = enumed.variant_name();

					if var_name == "Some" {
						let mut new_map = Map::new();
						for index in 0..enumed.field_len() {
							let ref_value = enumed.field_at(index).unwrap();
							new_map.insert(index.to_string(), Value::new_serde(ref_value));
						}

						self = Value::Object(new_map);
					} else if var_name == "None" {
						self = Value::Null;
					} else {
						match enumed.variant_type() {
							VariantType::Struct => todo!(),
							VariantType::Tuple => {
								let mut new_map = Map::new();
								for index in 0..enumed.field_len() {
									let ref_value = enumed.field_at(index).unwrap();
									new_map.insert(index.to_string(), Value::new_serde(ref_value));
								}

								self = Value::Object(new_map);
							}
							VariantType::Unit => self = var_name.into(),
						}
					}
				}
				ReflectRef::Opaque(partial_reflect) => {
					let output = format!("{:?}", partial_reflect);
					if let Ok(uinted) = output.parse::<u64>() {
						self = uinted.into();
					} else if let Ok(inted) = output.parse::<i64>() {
						self = inted.into();
					} else if let Ok(floated) = output.parse::<f64>() {
						self = floated.into();
					} else {
						self = output.trim_matches('\"').into();
					}
				}
				_ => unimplemented!("Ignore"),
			}

			self.take()
		}

		fn new_deserde(self) -> Box<dyn PartialReflect> {
			todo!()
		}

		fn decursive(self, reflect: &mut dyn PartialReflect) -> Box<dyn PartialReflect> {
			todo!()
		}
	}
}
