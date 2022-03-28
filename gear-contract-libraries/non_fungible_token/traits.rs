use gstd::{ActorId};
use primitive_types::U256;

pub trait NonFungibleTokenBase {

    fn mint(&mut self, to:&ActorId, token_id: U256);

    fn burn(&mut self, token_id: U256);

    fn transfer(&mut self, to: &ActorId, token_id: U256);
    
    fn approve(&mut self,  to: &ActorId, token_id: U256);

}

pub trait NonFungibleTokenAssert  {

    fn assert_token_exists(&self, token_id: U256, existed: bool);

    fn assert_can_transfer(&self, token_id: U256);

    fn assert_owner(&self, token_id: U256);

}