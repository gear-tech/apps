use crate::*;
use gstd::{msg, ActorId};

impl RMRKToken {
    pub async fn transfer(&mut self, to: &ActorId, token_id: TokenId) {
        self._transfer(
            to,
            &msg::source(),
            token_id,
            primitive_types::U256::from(0),
            ChildStatus::Unknown,
            0,
        )
        .await;
    }

    pub async fn _transfer(
        &mut self,
        _to: &ActorId,
        _from: &ActorId,
        _token_id: TokenId,
        _to_token_id: TokenId,
        _status: ChildStatus,
        _child_index: u128,
    ) {
        // self.assert_zero_address(to);
        // self.assert_approved_or_owner(token_id);
        // let rmrk_owner = self
        //     .rmrk_owners
        //     .get(&token_id)
        //     .expect("Token does not exist");

        // // our owner is an nft also
        // if rmrk_owner.token_id.is_some() {
        //     // let child_status = burn_child(&rmrk_owner.owner_id, rmrk_owner.token_id.unwrap(), token_id).await;
        //     let parent_contract = rmrk_owner.owner_id;
        //     match status {
        //         ChildStatus::Pending => self.reject_child(),
        //         ChildStatus::Accepted => self.remove_child(),
        //         ChildStatus::Unknown => self.remove_or_reject(),
        //         _ => panic!("RMRKCore: Invalid child status"),
        //     }
        // }

        // // if destination if nft
        // if !dest_is_nft {
        //     self.balances
        //         .entry(to)
        //         .and_modify(|balance| *balance += 1);
        // } else {
        //     let nextOwner = destContract.owner_of(token_id);
        //     self.balances
        //         .entry(nextOwner)
        //         .and_modify(|balance| *balance += 1);

        //     if from == nextOwner && status == ChildStatus::Accepted {
        //         destContract.add_accepted_child()
        //     } else {
        //         destContract.add_child();
        //     }
        // }
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
