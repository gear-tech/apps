use crate::*;
use gstd::{msg, ActorId};

impl RMRKToken {
    pub async fn burn(&mut self, token_id: TokenId) {
        let zero_id = &ActorId::new([0u8; 32]);
        self.assert_approved_or_owner(token_id);
        let owner = msg::source();
        self.balances
            .entry(owner)
            .and_modify(|balance| *balance -= 1);
        self.approve(zero_id, token_id).await;
        if let Some(ch_map) = self.children.get(&token_id) {
            for (child_token_id, child) in ch_map.clone().iter() {
                // TODO: initial solidity contract has some weird nested struct
                self.burn_child(token_id, *child_token_id);
                let _status = burn_child(&child.token_id, token_id, *child_token_id);
            }
        }
        self.rmrk_owners.remove(&token_id);
        msg::reply(
            RMRKEvent::Transfer {
                to: *zero_id,
                token_id,
            },
            0,
        )
        .unwrap();
    }
}
