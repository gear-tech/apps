use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId};
use primitive_types::U256;
use scale_info::TypeInfo;

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum NFTAction {
    Mint { to: ActorId, token_id: U256 },
    Burn(U256),
    Transfer { to: ActorId, token_id: U256 },
    Approve { to: ActorId, token_id: U256 },
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum NFTEvent {
    Transfer {
        from: ActorId,
        to: ActorId,
        token_id: U256,
    },
    Approval {
        owner: ActorId,
        approved_account: ActorId,
        token_id: U256,
    },
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct InitConfig {
    pub name: String,
    pub symbol: String,
    pub base_uri: String,
}
