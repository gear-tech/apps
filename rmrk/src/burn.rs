use crate::*;
use gstd::msg;
use primitive_types::U256;

impl RMRKToken {
    /// Burns RMRK token. It must be called from the root owner
    /// It recursively burns tokens starting from the contract it is called within
    /// then it looks if the token has children and call `burn_from_parent` function on this children
    /// Requirements:
    /// * The `msg::source()` must be owner of the token
    /// Arguments:
    /// * `token_id`: is the tokenId of the burnt token
    pub async fn burn(&mut self, token_id: TokenId) {

        let root_owner = self.assert_owner(token_id).await;
        self.balances
            .entry(root_owner)
            .and_modify(|balance| *balance -= 1);

        self.token_approvals.remove(&token_id);
        if let Some(children) = self.parent_to_children.get(&token_id) {
            for child_vec in children {
                let child_contract_id = ActorId::new(child_vec[1..32].try_into().unwrap());
                let child_token_id = U256::from(&child_vec[33..64]);
                burn_from_parent(&child_contract_id, child_token_id).await;
            }
        }

        self.parent_to_children.remove(&token_id);
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

    /// Burns RMRK token. It must be called from the RMRK parent contract
    /// RMRK parent contract passes `root_owner` as an argument
    /// So that the contract itself does not search recursively for the root owner
    /// It also recursively calls `burn_from_parent` function in case the burnt RMRK token has children
    /// Requirements:
    /// * The `msg::source()` must be RMRK parent contract
    /// Arguments:
    /// * `token_id`: is the tokenId of the burnt token
    pub async fn burn_from_parent(&mut self, token_id: TokenId, root_owner: &ActorId) {
        let rmrk_owner = self.rmrk_owners.get(&token_id).expect("Token does not exist");
        if msg::source() != rmrk_owner.owner_id || rmrk_owner.token_id.is_none() {
            panic!("Caller must be parent RMRK contract")
        }

        self.balances
            .entry(*root_owner)
            .and_modify(|balance| *balance -= 1);

        self.token_approvals.remove(&token_id);
        if let Some(children) = self.parent_to_children.get(&token_id) {
            for child_vec in children {
                let child_contract_id = ActorId::new(child_vec[1..32].try_into().unwrap());
                let child_token_id = U256::from(&child_vec[33..64]);
                burn_from_parent(&child_contract_id, child_token_id).await;
            }
        }

        self.parent_to_children.remove(&token_id);
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
