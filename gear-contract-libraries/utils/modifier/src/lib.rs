#![no_std]
extern crate proc_macro;
use proc_macro::TokenStream;

mod modifier;

/// Macro calls modifier function by passing self and the code of function's body.
/// # Explanation:
///
/// Let's define modifier.
/// ```
/// pub fn only_owner<T, F>(instance: &mut T, body: F) 
/// where
///     T: OwnableStorage,
///     F: FnOnce(&mut T),
/// {
///     if instance.get().owner != msg::source() {
///         panic!();
///     }
///     body(instance)
/// }
///
/// struct NFT {}
///
/// impl NFT {
///     #[modifier(only_owner)]
///     fn mint_token(&mut self) {
///         self.mint(&msg::source(), self.token_id);
///         self.token_id = self.token_id.saturating_add(U256::one());
///     }
/// ```
/// The code above will be expanded into:
/// ```
/// pub fn only_owner<T, F>(instance: &mut T, body: F) 
/// where
///     T: OwnableStorage,
///     F: FnOnce(&mut T),
/// {
///     if instance.get().owner != msg::source() {
///         panic!();
///     }
///     body(instance)
/// }
///
/// struct NFT {}
/// impl NFT {
///     fn mint_token(&mut self) {
///     let mut body_function = |modifier: &mut Self| {
///         modifier.mint(&msg::source(), modifier.token_id);
///         modifier.token_id = modifier.token_id.saturating_add(U256::one());
///     };
///     only_owner(self, body_function)
/// }

#[proc_macro_attribute]
pub fn modifier(_attrs: TokenStream, method: TokenStream) -> TokenStream {
    modifier::generate(_attrs, method)
}
