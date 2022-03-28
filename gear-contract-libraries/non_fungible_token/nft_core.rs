use gstd::{debug, exec, msg, prelude::*, ActorId};
use primitive_types::U256;
use crate::non_fungible_token::io::{NFTEvent};
use crate::non_fungible_token::traits::{NonFungibleTokenAssert, NonFungibleTokenBase};

const ZERO_ID: ActorId = ActorId::new([0u8; 32]);
pub use derive_trait::{NFTStorage,  ActionParser};

pub trait NFTStorage {
    fn get(&self) -> &NFTData;
    fn get_mut(&mut self) -> &mut NFTData;
}

#[derive(Debug, Default)]
pub struct NFTData {
    pub name: String,
    pub symbol: String,
    pub base_uri: String,
    pub owner_by_id: BTreeMap<U256, ActorId>,
    pub token_approvals: BTreeMap<U256, ActorId>,
    pub tokens_for_owner: BTreeMap<ActorId, Vec<U256>>,
    pub operator_approval: BTreeMap<ActorId, ActorId>,
}

impl<T: NFTStorage> NonFungibleTokenBase for T  {

    fn mint(&mut self, to:&ActorId, token_id: U256) {
        self.assert_token_exists(token_id, false);
        self.get_mut().owner_by_id.insert(token_id, *to);
        self.get_mut().tokens_for_owner.entry(*to)
            .and_modify(|tokens| tokens.push(token_id));
        msg::reply(
            NFTEvent::Transfer {
                from: ZERO_ID,
                to: *to,
                token_id,
            },
            0,
        );
    }

    fn burn(&mut self, token_id: U256) {
        self.assert_owner(token_id);
        let owner = *self.get().owner_by_id.get(&token_id).expect("NonFungibleToken: token does not exist");
        self.get_mut().owner_by_id.remove(&token_id);
        self.get_mut().tokens_for_owner.entry(owner)
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

    fn transfer(&mut self, to: &ActorId, token_id: U256) {
       self.assert_can_transfer(token_id);
       let owner = *self.get().owner_by_id.get(&token_id).expect("NonFungibleToken: token does not exist");

       // assign new owner
       self.get_mut().owner_by_id.entry(token_id)
            .and_modify(|owner| *owner = *to);
        // push token to new owner
        self.get_mut().tokens_for_owner.entry(*to)
            .and_modify(|tokens| tokens.push(token_id));
        // remove token from old owner
        self.get_mut().tokens_for_owner.entry(owner)
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

    fn approve(&mut self, to: &ActorId, token_id: U256) {
        self.assert_owner(token_id);
        let owner = *self.get().owner_by_id.get(&token_id).expect("NonFungibleToken: token does not exist");
        self.get_mut().token_approvals.insert(token_id, *to);
        msg::reply(
            NFTEvent::Approval {
                owner,
                approved_account: *to,
                token_id,
            },
            0,
        );
    }

}

impl<T: NFTStorage> NonFungibleTokenAssert for T {
    fn assert_token_exists(&self,  token_id: U256, existed: bool) {
        if !(self.get().owner_by_id.contains_key(&token_id) && existed) {
            panic!("NonFungibleToken: Token does not exist");
        } else if self.get().owner_by_id.contains_key(&token_id) && !existed{
            panic!("NonFungibleToken: Token already exists");
        }
     }
 
     fn assert_can_transfer(&self, token_id: U256) {
        let owner = self.get().owner_by_id.get(&token_id).expect("NonFungibleToken: token does not exist");
        let approved_account = self.get().token_approvals.get(&token_id).unwrap_or(&ZERO_ID);
        if !(owner == &msg::source() || owner == &exec::origin() || approved_account == &msg::source()) {
            panic!("NonFungibleToken: Not allowed to transfer");
        }
     }

     fn assert_owner(&self, token_id: U256) {
        let owner = self.get().owner_by_id.get(&token_id).expect("NonFungibleToken: token does not exist");
        if !(owner == &msg::source() || owner == &exec::origin()) {
            panic!("NonFungibleToken: Not allowed to apporve");
        }
     }
}


