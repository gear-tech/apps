use crate::constants::ZERO_ID;
use crate::*;
use gstd::msg;

impl RMRKToken {
    /// That function is designed to be from another RMRK contracts
    /// when burning tokens
    ///  It recursively burns tokens starting from the contract it is called within
    /// then it looks if the token has children and call burn function on this children
    /// Requirements:
    ///
    /// * The `msg::source()` must be a deployed RMRK contract
    /// * The `msg::source()` must be approved or owner of the token
    /// Arguments:
    /// * `token_id`: is the tokenId of the burnt token
    pub async fn burn(&mut self, token_id: TokenId) {
        let _ = self.assert_owner(token_id);
        self.balances
            .entry(msg::source())
            .and_modify(|balance| *balance -= 1);

        self.token_approvals.remove(&token_id);
        if let Some(ch_map) = self.children.get(&token_id) {
            for (child_token_id, child) in ch_map.clone().iter() {
                // Remove children from a parent_id in this contract, if there are any
                self.children.entry(token_id).and_modify(|children| {
                    children.remove(child_token_id);
                });
                // Pass recursively burn to a child
                burn(&child.token_id, *child_token_id).await;
            }
        }

        self.rmrk_owners.remove(&token_id);

        msg::reply(
            RMRKEvent::Transfer {
                to: ZERO_ID,
                token_id,
            },
            0,
        )
        .unwrap();
    }
}
