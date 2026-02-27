#[macro_export]
macro_rules! common {
	() => {
		println!("Hello");
	};
}

#[macro_export]
macro_rules! ambiguity {
	($func_name:ident) => {
		#[derive(Clone)]
		struct $func_name;

		impl $func_name {
			pub fn new() -> String {
				"Yes".to_owned()
			}
		}
		// fn $func_name() {
		// 	// The `stringify!` macro converts an `ident` into a string.
		// 	println!("You called {:?}()", stringify!($func_name));
		// }
	};
}
