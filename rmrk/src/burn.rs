use crate::*;
use gstd::{exec, msg, prelude::*, ActorId};

impl RMRKToken {
    pub async fn burn(&mut self, token_id: TokenId) {
        self.assert_approved_or_owner(token_id);
        let owner = msg::source();
        self.balances
            .entry(owner)
            .and_modify(|balance| *balance -= 1);
        self.approve(0.into(), token_id);
        if let Some(ch_map) = self.children.get(&token_id) {
            let children: Vec<Child> = ch_map.clone().into_values().collect();
            for i in 0..children.len() {
                self.burn_child(token_id, children[0].actual_id);
                let _status = burn_child(&children[i].token_id, token_id, children[i].actual_id).await;
            }
        }

        self.rmrk_owners.remove(&token_id);
        msg::reply(
            RMRKEvent::Transfer {
                to: 0.into(),
                token_id: token_id,
            },
            0,
        )
        .unwrap();
    }
}