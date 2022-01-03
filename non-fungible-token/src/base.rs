use gstd::{prelude::*, ActorId};
use primitive_types::U256;

pub trait NonFungibleTokenBase {
    /// called during the NFT contract deployment
    /// Arguments:
    /// * `name`: A descriptive name for a collection of NFTs in this contract
    /// * `symbol`: An abbreviated name for NFTs in this contract
    /// * `base_uri`: The URI of the NFT. This could be a website link, an API call, something on IPFS, some other unique identifier, etc
    fn init(&mut self, name: String, symbol: String, base_uri: String);

    /// Transfer an NFT item from current owner to the new one
    /// Arguments:
    /// * `token_id`: the ID of the token to transfer
    /// * `from`: the valid ActorId. It can the the token owner or the actor with the right to transfer the token
    /// * `to`: the valid ActorId, the account to which the token will be sent
    /// Contract must panic if `from` is neither the token owner nor the approved actor for the token. It also must panic if `to` is a zero ID
    fn transfer(&mut self, rom: &ActorId, to: &ActorId, token_id: U256);

    /// Gives a right to the actor to manage the specific token
    /// Arguments:
    /// * `token_id`: the token ID
    /// * `owner`: the valid ActorId that must be the token owner
    /// * `spender`: the valid ActorId that will be approved to manage the token
    /// Contract must panic if `owner` is not the token owner or `spender` is a zero ID
    fn approve(&mut self, owner: &ActorId, spender: &ActorId, token_id: U256);

    /// Enables or disables the actor to manage all the tokens the owner has
    /// Arguments:
    /// * `owner`: the valid ActorId that must be the token owner
    /// * `operator`: the valid ActorId that will be approved to manage the tokens
    /// * `approved`: True if the operator is approved, false to revoke approval
    /// Contract must panic if `owner` is not the token owner or `operator` is a zero ID
    fn approve_for_all(&mut self, owner: &ActorId, operator: &ActorId, approved: bool);
}
