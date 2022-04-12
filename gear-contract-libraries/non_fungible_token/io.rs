use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId};
use scale_info::TypeInfo;
use crate::non_fungible_token::token::*;

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum NFTAction {
    Mint { to: ActorId, token_id: TokenId, token_metadata: Option<TokenMetadata>},
    Burn { token_id: TokenId },
    Transfer { to: ActorId, token_id: TokenId },
    Approve { to: ActorId, token_id: TokenId },
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum NFTEvent {
    Transfer {
        from: ActorId,
        to: ActorId,
        token_id: TokenId,
    },
    Approval {
        owner: ActorId,
        approved_account: ActorId,
        token_id: TokenId,
    },
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct InitConfig {
    pub name: String,
    pub symbol: String,
    pub base_uri: String,
}
