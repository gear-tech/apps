use gstd::{prelude::*, ActorId};
use crate::non_fungible_token::token::*;

#[derive(Debug, Default)]
pub struct NFTState {
    pub name: String,
    pub symbol: String,
    pub base_uri: String,
    pub owner_by_id: BTreeMap<TokenId, ActorId>,
    pub token_approvals: BTreeMap<TokenId, Vec<ActorId>>,
    pub token_metadata_by_id: BTreeMap<TokenId, Option<TokenMetadata>>,
    pub tokens_for_owner: BTreeMap<ActorId, Vec<TokenId>>,
}

pub trait StateKeeper {
    fn get(&self) -> &NFTState;
    fn get_mut(&mut self) -> &mut NFTState;
}

#[macro_export]
macro_rules! impl_state_keeper {
    ($struct_name:ty, $field_name:ident) => {
        impl StateKeeper for $struct_name {
            fn get(&self) -> &NFTState {
                &self.$field_name
            }

            fn get_mut(&mut self) -> &mut NFTState {
                &mut self.$field_name
            }
        }
    }
}