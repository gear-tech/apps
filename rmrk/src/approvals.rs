use crate::*;
use gstd::{exec, msg, prelude::*, ActorId};

impl RMRKToken {
    pub async fn transfer(&mut self, to: &ActorId, token_id: TokenId) {
        self.assert_zero_address(to);
        self.assert_approved_or_owner(token_id);
        let rmrk_owner = self
            .rmrk_owners
            .get(&token_id)
            .expect("Token does not exist");
        if rmrk_owner.token_id.is_some() {
            let child_status = burn_child(&rmrk_owner.owner_id, rmrk_owner.token_id.unwrap(), token_id).await;

        }

    }

    pub fn transfer_to_nft(&mut self, to: &ActorId, destination_id: TokenId, token_id: TokenId) {}
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
