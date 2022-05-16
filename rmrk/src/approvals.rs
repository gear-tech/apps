use crate::*;
use gstd::{msg, ActorId};

impl RMRKToken {
    pub async fn transfer(&mut self, to: &ActorId, token_id: TokenId) {
        self._transfer(to, &msg::source(), token_id, primitive_types::U256::from(0))
            .await;
    }

    pub async fn _transfer(
        &mut self,
        to: &ActorId,
        from: &ActorId,
        token_id: TokenId,
        to_token_id: TokenId,
    ) {
        self.assert_zero_address(to);
        self.assert_approved_or_owner(token_id);
        let rmrk_owner = self
            .rmrk_owners
            .get(&token_id)
            .expect("Token does not exist");

        let rmk_dest = self
            .rmrk_owners
            .get(&token_id)
            .expect("Token does not exist");

        // // our owner is an nft also
        if rmrk_owner.token_id.is_some() {
            let parent_contract = rmrk_owner.owner_id;
            // transfer all the children
            if let Some(ch_map) = self.children.get(&token_id) {
                for (child_token_id, child) in ch_map.clone().iter() {
                    match child.status {
                        ChildStatus::Pending => {
                            reject_child(&parent_contract, token_id, *child_token_id).await
                        }
                        ChildStatus::Accepted => {
                            remove_child(&parent_contract, token_id, *child_token_id).await
                        }
                        ChildStatus::Unknown => {
                            // find status
                            let un_status = ChildStatus::Accepted;
                            match un_status {
                                ChildStatus::Pending => {
                                    reject_child(&parent_contract, token_id, *child_token_id).await
                                }
                                ChildStatus::Accepted => {
                                    remove_child(&parent_contract, token_id, *child_token_id).await
                                }
                                ChildStatus::Unknown => panic!("RMRKCore: Invalid child status"),
                            }
                        }
                    }
                }
            }
        }
        // the destination is an nft
        if rmk_dest.token_id.is_some() {
            // get nextOwner
            let next_owner = get_root_owner(&rmk_dest.owner_id, to_token_id).await;
            self.balances
                .entry(next_owner)
                .and_modify(|balance| *balance += 1);
            if let Some(ch_map) = self.children.get(&token_id) {
                for (child_token_id, child) in ch_map.clone().iter() {
                    if next_owner == *from && child.status == ChildStatus::Accepted {
                        let _response = add_accepted_child(&next_owner, token_id, *child_token_id);
                    } else {
                        let _response = add_child(&next_owner, token_id, *child_token_id);
                    }
                }
            }
        } else {
            self.balances.entry(*to).and_modify(|balance| *balance += 1);
        }

        msg::reply(RMRKEvent::Transfer { to: *to, token_id }.encode(), 0).unwrap();
    }

    pub fn transfer_to_nft(&mut self, _to: &ActorId, _destination_id: TokenId, _token_id: TokenId) {
    }

    pub async fn approve(&mut self, to: &ActorId, token_id: TokenId) {
        let owner = self.assert_owner(token_id);
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
            }
            .encode(),
            0,
        )
        .unwrap();
    }
}
