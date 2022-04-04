use gstd::{prelude::*, ActorId};

use crate::common::BalanceOfBatchReply;
use crate::common::TokenId;
use crate::common::TokenMetadata;

pub trait ERC1155TokenBase {
    fn balance_of(&self, account: &ActorId, id: &TokenId) -> u128;
    fn balance_of_batch(&self, accounts: &[ActorId], ids: &[TokenId]) -> Vec<BalanceOfBatchReply>;
    fn set_approval_for_all(&mut self, operator: &ActorId, approved: bool);
    fn is_approved_for_all(&self, account: &ActorId, operator: &ActorId) -> bool;
    fn safe_transfer_from(&mut self, from: &ActorId, to: &ActorId, id: &TokenId, amount: u128);
    fn safe_batch_transfer_from(
        &mut self,
        from: &ActorId,
        to: &ActorId,
        ids: &[TokenId],
        amounts: &[u128],
    );
    fn can_transfer(&self, from: &ActorId, id: &TokenId, amount: u128) -> bool;
}

pub trait ExtendERC1155TokenBase {
    fn mint(&mut self, account: &ActorId, id: &TokenId, amount: u128, meta: Option<TokenMetadata>);
    fn mint_batch(
        &mut self,
        account: &ActorId,
        ids: &[TokenId],
        amounts: &[u128],
        meta: Vec<Option<TokenMetadata>>,
    );
    fn burn(&mut self, id: &TokenId, amount: u128);
    fn burn_batch(&mut self, ids: &[TokenId], amounts: &[u128]);
    fn is_owner_of(&self, id: &TokenId) -> bool;
    fn is_owner_of_batch(&self, ids: &[TokenId]) -> bool;
    fn uri(&self, id: TokenId) -> String;
    fn can_burn(&mut self, owner: &ActorId, id: &TokenId, amount: u128) -> bool;
    fn get_metadata(&self, id: TokenId) -> TokenMetadata;
}
