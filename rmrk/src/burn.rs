use crate::*;
use gstd::msg;
use primitive_types::U256;

impl RMRKToken {
    /// Burns RMRK token. It must be called from the root owner
    /// It recursively burns tokens starting from the contract it is called within
    /// then it looks if the token has children and call `burn_from_parent` function on these children
    /// Requirements:
    /// * The `msg::source()` must be owner of the token
    /// Arguments:
    /// * `token_id`: is the tokenId of the burnt token
    pub async fn burn(&mut self, token_id: TokenId) {
        let root_owner = self.find_root_owner(token_id).await;
        self.assert_owner(&root_owner);
        self.balances
            .entry(root_owner)
            .and_modify(|balance| *balance -= 1);

        self.token_approvals.remove(&token_id);

        self.internal_burn_children(token_id, &root_owner).await;
        let rmrk_owner = self.rmrk_owners.remove(&token_id).unwrap();
        if rmrk_owner.token_id.is_some() {
            burn_child(&rmrk_owner.owner_id, rmrk_owner.token_id.unwrap(), token_id).await;
        }

        msg::reply(
            RMRKEvent::Transfer {
                to: ZERO_ID,
                token_id,
            },
            0,
        )
        .unwrap();
    }

    /// Burns RMRK token. It must be called from the RMRK parent contract
    /// RMRK parent contract passes `root_owner` as an argument
    /// So that the contract itself does not search recursively for the root owner
    /// It also recursively calls `burn_from_parent` function in case the burnt RMRK token has children
    /// Requirements:
    /// * The `msg::source()` must be RMRK parent contract
    /// Arguments:
    /// * `token_id`: is the tokenId of the burnt token
    pub async fn burn_from_parent(&mut self, token_ids: Vec<TokenId>, root_owner: &ActorId) {
        for token_id in &token_ids {
            let rmrk_owner = self
                .rmrk_owners
                .get(&token_id)
                .expect("Token does not exist");
            if msg::source() != rmrk_owner.owner_id {
                panic!("Caller must be parent RMRK contract")
            }
            self.token_approvals.remove(&token_id);
            self.balances
                .entry(*root_owner)
                .and_modify(|balance| *balance -= 1);
            self.internal_burn_children(*token_id, root_owner).await;
            self.rmrk_owners.remove(&token_id);
        }

        msg::reply(RMRKEvent::TokensBurnt { token_ids }, 0).unwrap();
    }

    async fn internal_burn_children(&mut self, token_id: TokenId, root_owner: &ActorId) {
        let pending_children = self.get_pending_children(token_id);
        for (child_contract_id, children) in &pending_children {
            burn_from_parent(&child_contract_id, children.clone(), &root_owner).await;
        }
        let accepted_children = self.get_accepted_children(token_id);
        for (child_contract_id, children) in &accepted_children {
            burn_from_parent(&child_contract_id, children.clone(), &root_owner).await;
        }
    }
}
