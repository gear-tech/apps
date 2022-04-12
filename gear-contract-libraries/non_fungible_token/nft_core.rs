use crate::non_fungible_token::{io::*, state::*, token::*};
use gstd::{exec, msg, prelude::*, ActorId};

const ZERO_ID: ActorId = ActorId::new([0u8; 32]);

pub trait NFTCore: StateKeeper + NonFungibleTokenAssert {
    fn mint(&mut self, to: &ActorId, token_id: TokenId, token_metadata: Option<TokenMetadata>) {
        self.assert_token_exists(token_id, false);
        self.get_mut().owner_by_id.insert(token_id, *to);
        self.get_mut()
            .tokens_for_owner
            .entry(*to)
            .and_modify(|tokens| tokens.push(token_id));
        self.get_mut().token_metadata_by_id.insert(token_id, token_metadata);
        msg::reply(
            NFTEvent::Transfer {
                from: ZERO_ID,
                to: *to,
                token_id,
            },
            0,
        );
    }

    fn burn(&mut self, token_id: TokenId) {
        self.assert_owner(token_id);
        let owner = *self
            .get_mut()
            .owner_by_id
            .get(&token_id)
            .expect("NonFungibleToken: token does not exist");
        self.get_mut().owner_by_id.remove(&token_id);
        self.get_mut()
            .tokens_for_owner
            .entry(owner)
            .and_modify(|tokens| tokens.retain(|&token| token != token_id));
        msg::reply(
            NFTEvent::Transfer {
                from: owner,
                to: ZERO_ID,
                token_id,
            },
            0,
        );
    }

    fn transfer(&mut self, to: &ActorId, token_id: TokenId) {
        self.assert_can_transfer(token_id);
        let owner = *self
            .get()
            .owner_by_id
            .get(&token_id)
            .expect("NonFungibleToken: token does not exist");

        // assign new owner
        self.get_mut()
            .owner_by_id
            .entry(token_id)
            .and_modify(|owner| *owner = *to);
        // push token to new owner
        self.get_mut()
            .tokens_for_owner
            .entry(*to)
            .and_modify(|tokens| tokens.push(token_id));
        // remove token from old owner
        self.get_mut()
            .tokens_for_owner
            .entry(owner)
            .and_modify(|tokens| tokens.retain(|&token| token != token_id));
        // remove approvals if any
        self.get_mut().token_approvals.remove(&token_id);

        msg::reply(
            NFTEvent::Transfer {
                from: owner,
                to: *to,
                token_id,
            },
            0,
        );
    }

    fn approve(&mut self, to: &ActorId, token_id: TokenId) {
        self.assert_owner(token_id);
        let owner = *self
            .get()
            .owner_by_id
            .get(&token_id)
            .expect("NonFungibleToken: token does not exist");
        // self.token_approvals.insert(token_id, *to);
        msg::reply(
            NFTEvent::Approval {
                owner,
                approved_account: *to,
                token_id,
            },
            0,
        );
    }

    fn proc(&mut self, bytes: Vec<u8>) -> Option<()> {
        let action = NFTAction::decode(&mut &bytes[..]).ok()?;
        match action {
            NFTAction::Mint { to, token_id, token_metadata } => Self::mint(self, &to, token_id, token_metadata),
            NFTAction::Burn { token_id } => Self::burn(self, token_id),
            NFTAction::Transfer { to, token_id } => Self::transfer(self, &to, token_id),
            NFTAction::Approve { to, token_id } => Self::approve(self, &to, token_id),
        }
        Some(())
    }
}

pub trait NonFungibleTokenAssert: StateKeeper {
    fn assert_token_exists(&self, token_id: TokenId, existed: bool) {
        if !(self.get().owner_by_id.contains_key(&token_id) && existed) {
            panic!("NonFungibleToken: Token does not exist");
        } else if self.get().owner_by_id.contains_key(&token_id) && !existed {
            panic!("NonFungibleToken: Token already exists");
        }
    }

    fn assert_can_transfer(&self, token_id: TokenId) {
        if let Some(approved_accounts) = self.get().token_approvals.get(&token_id) {
            if approved_accounts.contains(&msg::source()) {
                return;
            }
        }
        self.assert_owner(token_id);
    }

    fn assert_owner(&self, token_id: TokenId) {
        let owner = self
            .get()
            .owner_by_id
            .get(&token_id)
            .expect("NonFungibleToken: token does not exist");
        if !(owner == &msg::source() || owner == &exec::origin()) {
            panic!("Not allowed to transfer");
        }
    }
}
