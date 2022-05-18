use crate::*;
use gstd::{debug, msg, ActorId};

impl RMRKToken {
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
        debug!("TO: {:?}", to);
        debug!("TOKEN_ID: {:?}", token_id);
        msg::reply(RMRKEvent::Transfer { to: *to, token_id }, 0).unwrap();
    }

    // pub async fn _transfer(
    //     &mut self,
    //     to: &ActorId,
    //     _from: &ActorId,
    //     token_id: TokenId,
    //     to_token_id: TokenId,
    // ) {
    //     self.assert_zero_address(to);
    //     self.assert_approved_or_owner(token_id);
    //     let rmrk_owner = self
    //         .rmrk_owners
    //         .get(&token_id)
    //         .expect("Token does not exist");

    //     self.balances
    //         .entry(rmrk_owner.owner_id)
    //         .and_modify(|balance| *balance -= 1);
    //     let mut ch_ids: Vec<TokenId> = Vec::new();
    //     let mut ch_token_ids: Vec<ActorId> = Vec::new();
    //     let mut ch_statuses: Vec<ChildStatus> = Vec::new();
    //     // // our owner is an nft also
    //     if rmrk_owner.token_id.is_some() {
    //         let parent_contract = rmrk_owner.owner_id;
    //         // transfer all the children
    //         if let Some(ch_map) = self.children.get(&token_id) {
    //             for (child_token_id, child) in ch_map.clone().iter() {
    //                 ch_ids.push(*child_token_id);
    //                 ch_token_ids.push(child.token_id);
    //                 ch_statuses.push(child.status);
    //             }
    //             // transfer children with add = false, since we want to remove those
    //             transfer_children(
    //                 &parent_contract,
    //                 token_id,
    //                 ch_ids.clone(),
    //                 ch_token_ids.clone(),
    //                 ch_statuses.clone(),
    //                 false,
    //             )
    //             .await;
    //         }
    //     }

    //     let rmk_dest = self
    //         .rmrk_owners
    //         .get(&to_token_id)
    //         .expect("Token does not exist");
    //     // the destination is an nft
    //     if rmk_dest.token_id.is_some() {
    //         // get nextOwner
    //         let next_owner = get_root_owner(&rmk_dest.owner_id, to_token_id).await;
    //         self.balances
    //             .entry(next_owner)
    //             .and_modify(|balance| *balance += 1);

    //         transfer_children(
    //             &next_owner,
    //             to_token_id,
    //             ch_ids.clone(),
    //             ch_token_ids.clone(),
    //             ch_statuses.clone(),
    //             true,
    //         )
    //         .await;
    //     } else {
    //         self.balances.entry(*to).and_modify(|balance| *balance += 1);
    //     }

    //     msg::reply(RMRKEvent::Transfer { to: *to, token_id }.encode(), 0).unwrap();
    // }

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
