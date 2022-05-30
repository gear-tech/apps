use crate::*;
use gstd::{exec, msg, ActorId};

impl RMRKToken {
    pub fn assert_zero_address(&self, account: &ActorId) {
        if account == &ZERO_ID {
            panic!("RMRK: Zero address");
        }
    }

    /// Checks that NFT with indicated ID already exists
    pub fn assert_token_exists(&self, token_id: TokenId) {
        if self.rmrk_owners.contains_key(&token_id) {
            panic!("RMRK: Token already exists");
        }
    }

    /// Checks that NFT with indicated ID already does not exist
    pub fn assert_token_does_not_exist(&self, token_id: TokenId) {
        if !self.rmrk_owners.contains_key(&token_id) {
            panic!("RMRK: Token does not exist");
        }
    }

    /// Checks that `msg::source()` is the owner of the token with indicated `token_id`
    pub fn assert_owner(&self, root_owner: &ActorId) {
        debug!("OWNER {:?}", root_owner);
        if msg::source() != *root_owner {
            panic!("Wrong owner");
        }
    }

    /// Checks that `exec::origin()` is the owner of the token with indicated `token_id`
    pub fn assert_exec_origin(&self, root_owner: &ActorId) {
        debug!("EXEC OWNER {:?}", root_owner);
        if exec::origin() != *root_owner {
            panic!("Wrong owner");
        }
    }

    pub fn assert_approved_or_owner(&self, token_id: TokenId, root_owner: &ActorId) {
        if let Some(approved_accounts) = self.token_approvals.get(&token_id) {
            if approved_accounts.contains(&msg::source()) {
                return;
            }
        }
        self.assert_owner(root_owner);
    }
}
