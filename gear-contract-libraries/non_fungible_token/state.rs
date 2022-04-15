use crate::non_fungible_token::token::*;
use gstd::{prelude::*, ActorId};

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
    };
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum NFTQuery {
    Token { token_id: TokenId },
    TokensForOwner { owner: ActorId },
    TotalSupply,
    SupplyForOwner { owner: ActorId },
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum NFTQueryReply {
    Token { token: Token },
    TokensForOwner { tokens: Vec<Token> },
    TotalSupply { total_supply: u128 },
    SupplyForOwner { supply: u128 },
}

pub trait NFTMetaState: StateKeeper {
    fn token(&self, token_id: TokenId) -> Token {
        if let Some(owner_id) = self.get().owner_by_id.get(&token_id) {
            Token {
                token_id,
                owner_id: *owner_id,
                metadata: None,
                approved_account_ids: Vec::new(),
            }
        } else {
            Token::default()
        }
    }

    fn tokens_for_owner(&self, owner: &ActorId) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
        if let Some(token_ids) = self.get().tokens_for_owner.get(owner) {
            for token_id in token_ids {
                tokens.push(self.token(*token_id));
            }
        }
        tokens
    }

    fn total_supply(&self) -> u128 {
        self.get().owner_by_id.len() as u128
    }

    fn supply_for_owner(&self, owner: &ActorId) -> u128 {
        self.get()
            .tokens_for_owner
            .get(owner)
            .map(|tokens| tokens.len() as u128)
            .unwrap_or(0)
    }
    // fn all_tokens(&self, owner: &ActorId) -> Vec<Token> {
    //     let tokens: Vec<Token> = Vec::new();
    //     if let Some(token_ids) = self.get().tokens_for_owner.get(owner) {
    //         for token_id in token_ids {
    //             tokens.push(self.token(*token_id));
    //         };
    //     }
    //     tokens
    // }

    fn proc_state(&mut self, bytes: Vec<u8>) -> Option<Vec<u8>> {
        let query = NFTQuery::decode(&mut &bytes[..]).ok()?;
        let encoded = match query {
            NFTQuery::Token { token_id } => NFTQueryReply::Token {
                token: Self::token(self, token_id),
            }
            .encode(),
            NFTQuery::TokensForOwner { owner } => NFTQueryReply::TokensForOwner {
                tokens: Self::tokens_for_owner(self, &owner),
            }
            .encode(),
            NFTQuery::TotalSupply => NFTQueryReply::TotalSupply {
                total_supply: Self::total_supply(self),
            }
            .encode(),
            NFTQuery::SupplyForOwner { owner } => NFTQueryReply::SupplyForOwner {
                supply: Self::supply_for_owner(self, &owner),
            }
            .encode(),
        };
        Some(encoded)
    }
}
