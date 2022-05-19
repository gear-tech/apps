use crate::*;
use gstd::{msg, ActorId};

impl RMRKToken {
    /// Mints token that will belong to another token in another RMRK contract
    /// Requirements:
    /// * The `to`  must be a deployed RMRK contract
    /// * The `token_id` must not exist
    /// Arguments:
    /// * `to`: is the address of RMRK parent contract
    /// * `destination_id`: is the parent RMRK token
    /// * `token_id`: is the tokenId of new RMRK token
    pub async fn mint_to_nft(&mut self, to: &ActorId, token_id: TokenId, destination_id: TokenId) {
        self.assert_token_exists(token_id);
        //check that `to` is a deployed program

        let root_owner = add_child(to, destination_id, token_id).await;

        self.balances
            .entry(root_owner)
            .and_modify(|balance| *balance += 1)
            .or_insert(1);

        self.rmrk_owners.insert(
            token_id,
            RMRKOwner {
                token_id: Some(destination_id),
                owner_id: *to,
            },
        );

        msg::reply(
            RMRKEvent::MintToNft {
                to: *to,
                token_id,
                destination_id,
            },
            0,
        )
        .unwrap();
    }

    /// Mints token to the user
    /// Requirements:
    /// * The ``token_id` must not exist
    /// * The `to` address should be a non-zero address
    /// Arguments:
    /// * `to`: is the address who will own the token
    /// * `token_id`: is the tokenId of new RMRK token
    pub fn mint_to_root_owner(&mut self, to: &ActorId, token_id: TokenId) {
        self.assert_zero_address(to);
        // check that token does not exist
        self.assert_token_exists(token_id);
        self.balances
            .entry(*to)
            .and_modify(|balance| *balance += 1)
            .or_insert(1);

        self.rmrk_owners.insert(
            token_id,
            RMRKOwner {
                token_id: None,
                owner_id: *to,
            },
        );

        msg::reply(RMRKEvent::MintToRootOwner { to: *to, token_id }, 0).unwrap();
    }
}
