use crate::*;
use gstd::{debug, msg, ActorId};

impl RMRKToken {
    /// That function is designed to be from another RMRK contracts
    /// when transfering tokens to a non-nft address
    /// It checks for the ownership and whether the msg::source is an NFT contract
    /// if it is it burns itself from root_owner
    /// then it just swaps the balances
    /// Requirements:
    ///
    /// * The `msg::source()` must be a deployed RMRK contract
    /// * The ``token_id` should exist
    /// * The `to` address should be a non-zero address
    /// * The `msg::source()` must be approved or owner of the token
    /// Arguments:
    /// * `to`: is the receiving address
    /// * `token_id`: is the tokenId of the transfered token
    pub async fn transfer(&mut self, to: &ActorId, token_id: TokenId) {
        let rmrk_owner = self
            .rmrk_owners
            .get(&token_id)
            .expect("Token does not exist");

        self.assert_zero_address(to);
        self.assert_approved_or_owner(token_id).await;
        let previous_root_owner = if rmrk_owner.token_id.is_some() {
            burn_child(&rmrk_owner.owner_id, rmrk_owner.token_id.unwrap(), token_id).await;
            get_root_owner(&rmrk_owner.owner_id, rmrk_owner.token_id.unwrap()).await
        } else {
            rmrk_owner.owner_id
        };
        msg::reply(RMRKEvent::Transfer { to: *to, token_id }, 0).unwrap();
    }


    /// That function is designed to be from another RMRK contracts
    /// when transfering tokens to a non-nft address
    /// It checks for the ownership and whether the msg::source is an NFT contract
    /// if it is it burns itself from root_owner
    /// then it just swaps the balances
    /// If to is also an NFT contract it adds a child to destination_id token
    /// Requirements:
    ///
    /// * The `msg::source()` must be a deployed RMRK contract
    /// * The ``token_id` should exist
    /// *
    /// * The `to` address should be a non-zero address
    /// * The `msg::source()` must be approved or owner of the token
    /// Arguments:
    /// * `to`: is the receiving address
    /// * `token_id`: is the tokenId of the transfered token
    pub async fn transfer_to_nft(
        &mut self,
        to: &ActorId,
        destination_id: TokenId,
        token_id: TokenId,
    ) {
        let rmrk_owner = self
            .rmrk_owners
            .get(&token_id)
            .expect("Token does not exist");

        self.assert_zero_address(to);
        self.assert_approved_or_owner(token_id).await;
        // if that NFT has parent NFT contract
        let previous_root_owner = if rmrk_owner.token_id.is_some() {
            // burn that child from previous parent NFT contract
            burn_child(&rmrk_owner.owner_id, rmrk_owner.token_id.unwrap(), token_id).await;
            get_root_owner(&rmrk_owner.owner_id, rmrk_owner.token_id.unwrap()).await
        } else {
            rmrk_owner.owner_id
        };

        let root_owner = add_child(to, destination_id, token_id).await;
        // if new root owner differs from the previous one
        if root_owner != previous_root_owner {
            self.balances
                .entry(root_owner)
                .and_modify(|balance| *balance += 1)
                .or_insert(1);
            self.balances
                .entry(previous_root_owner)
                .and_modify(|balance| *balance -= 1);
        }
        self.rmrk_owners.insert(
            token_id,
            RMRKOwner {
                token_id: Some(destination_id),
                owner_id: *to,
            },
        );
        msg::reply(RMRKEvent::Transfer { to: *to, token_id }, 0).unwrap();
    }

    pub async fn approve(&mut self, to: &ActorId, token_id: TokenId) {
        let owner = self.assert_owner(token_id).await;
        self.assert_zero_address(to);
        self.token_approvals
            .entry(token_id)
            .and_modify(|approvals| approvals.push(*to))
            .or_insert_with(|| vec![*to]);
        msg::reply(
            RMRKEvent::Approval {
                owner,
                approved_account: *to,
                token_id,
            },
            0,
        )
        .unwrap();
    }
}
