use gstd::{prelude::*, ActorId};
use primitive_types::U256;

pub trait NonFungibleTokenBase {
    
    /// Transfer an NFT item from current owner to the new one
    /// Arguments:
    /// * `token_id`: the ID of the token to transfer
    /// * `from`: the valid ActorId. It can the the token owner or the actor with the right to transfer the token
    /// * `to`: the valid ActorId, the account to which the token will be sent
    /// Contract must panic if `from` is neither the token owner nor the approved actor for the token. It also must panic if `to` is a zero ID
    fn transfer(&mut self, to: &ActorId, token_id: U256);

    /// Gives a right to the actor to manage the specific token
    /// Arguments:
    /// * `token_id`: the token ID
    /// * `owner`: the valid ActorId that must be the token owner
    /// * `spender`: the valid ActorId that will be approved to manage the token
    /// Contract must panic if `owner` is not the token owner or `spender` is a zero ID
    fn approve(&mut self, owner: &ActorId, spender: &ActorId, token_id: U256);

    /// Sends a message including the information about the balance of `account`
    /// Arguments:
    /// * `account`: the valid ActorId
    fn balance_of(&self, account: &ActorId);

    /// Sends a message including the information about the owner of `token_id`
    /// If token does not exist, it sends the zero address
    /// Arguments:
    /// * `token_id`: the token ID
    fn owner_of(&self, token_id: U256);
}
