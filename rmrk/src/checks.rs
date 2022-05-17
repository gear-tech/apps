use crate::constants::ZERO_ID;
use crate::*;
use gstd::{exec, msg, ActorId};

impl RMRKToken {
    pub fn assert_zero_address(&self, account: &ActorId) {
        if account == &ZERO_ID {
            panic!("RMRK: Zero address");
        }
    }

    pub async fn assert_parent(&self, token_id: TokenId) {
        let response: RMRKEvent =
            msg::send_and_wait_for_reply(msg::source(), RMRKAction::NFTParent { token_id }, 0)
                .unwrap()
                .await
                .expect("Error in NFTParent message");
        if let RMRKEvent::NFTParent { parent } = response {
            if parent != exec::program_id() {
                panic!("RMRCore:: Wrong parent address");
            }
        } else {
            panic!("Wrong received message");
        }
    }

    /// Checks that NFT with indicated ID already exists
    pub fn assert_token_exists(&self, token_id: TokenId) {
        if self.rmrk_owners.contains_key(&token_id) {
            panic!("RMRK: Token already exists");
        }
    }

    /// Checks that `msg::source()` is the owner of the token with indicated `token_id`
    pub fn assert_owner(&self, token_id: TokenId) -> ActorId {
        let rmrk_owner = self
            .rmrk_owners
            .get(&token_id)
            .expect("NonFungibleToken: token does not exist");

        if rmrk_owner.root_owner != msg::source() {
            panic!("RMRK: Not approved or owner");
        }
        rmrk_owner.root_owner
    }

    pub fn assert_approved_or_owner(&self, token_id: TokenId) {
        if self
            .token_approvals
            .get(&token_id)
            .expect("token does not exist")
            .contains(&msg::source())
        {
            return;
        }
        self.assert_owner(token_id);
    }
}
