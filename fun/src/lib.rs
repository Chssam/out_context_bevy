extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro]
pub fn ab(_item: TokenStream) -> TokenStream {
	"fn answer() -> u32 { 42 }".parse().unwrap()
}

#[proc_macro_derive(AnswerFn)]
pub fn abaa(_item: TokenStream) -> TokenStream {
	"fn answer() -> u32 { 42 }".parse().unwrap()
}

#[proc_macro_derive(WithHelperAttr, attributes(helper))]
pub fn derive_with_helper_attr(_item: TokenStream) -> TokenStream {
	TokenStream::new()
}
