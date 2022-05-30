#![no_std]

use codec::Encode;
use gstd::{debug, exec, msg, prelude::*, ActorId};
use primitive_types::U256;
use rmrk_io::*;
pub mod burn;
pub mod checks;
pub mod children;
pub mod messages;
pub mod transfer;
use messages::*;
pub mod mint;

#[derive(Debug)]
pub struct RMRKOwner {
    pub token_id: Option<TokenId>,
    pub owner_id: ActorId,
}

#[derive(Debug, Default)]
pub struct RMRKToken {
    pub name: String,
    pub symbol: String,
    pub token_approvals: BTreeMap<TokenId, Vec<ActorId>>,
    pub rmrk_owners: BTreeMap<TokenId, RMRKOwner>,
    pub pending_children: BTreeMap<TokenId, BTreeSet<Vec<u8>>>,
    pub accepted_children: BTreeMap<TokenId, BTreeSet<Vec<u8>>>,
    pub children_status: BTreeMap<Vec<u8>, ChildStatus>,
    pub balances: BTreeMap<ActorId, u128>,
}

static mut RMRK: Option<RMRKToken> = None;
pub const ZERO_ID: ActorId = ActorId::new([0u8; 32]);

impl RMRKToken {

    // reply about root_owner
    async fn root_owner(&self, token_id: TokenId) {
        let root_owner = self.find_root_owner(token_id).await;
        msg::reply(RMRKEvent::RootOwner { root_owner }, 0).unwrap();
    }

    // internal search for root owner
    async fn find_root_owner(&self, token_id: TokenId) -> ActorId {
        let rmrk_owner = self
            .rmrk_owners
            .get(&token_id)
            .expect("RMRK: Token does not exist");
        let root_owner = if rmrk_owner.token_id.is_some() {
            get_root_owner(&rmrk_owner.owner_id, rmrk_owner.token_id.unwrap()).await
        } else {
            rmrk_owner.owner_id
        };
        root_owner
    }

    fn get_pending_children(&self, token_id: TokenId) -> BTreeMap<ActorId, Vec<TokenId>> {
        let mut pending_children: BTreeMap<ActorId, Vec<TokenId>> = BTreeMap::new();
        if let Some(children) = self.pending_children.get(&token_id) {
            for child_vec in children.iter() {
                let child_contract_id = ActorId::new(child_vec[0..32].try_into().unwrap());
                let child_token_id = U256::from(&child_vec[32..64]);
                pending_children
                    .entry(child_contract_id)
                    .and_modify(|c| c.push(child_token_id))
                    .or_insert_with(|| vec![child_token_id]);
            }
        }
        pending_children
    }

    fn get_accepted_children(&self, token_id: TokenId) -> BTreeMap<ActorId, Vec<TokenId>> {
        let mut accepted_children: BTreeMap<ActorId, Vec<TokenId>> = BTreeMap::new();
        if let Some(children) = self.accepted_children.get(&token_id) {
            for child_vec in children.iter() {
                let child_contract_id = ActorId::new(child_vec[0..32].try_into().unwrap());
                let child_token_id = U256::from(&child_vec[32..64]);
                accepted_children
                    .entry(child_contract_id)
                    .and_modify(|c| c.push(child_token_id))
                    .or_insert_with(|| vec![child_token_id]);
            }
        }
        accepted_children
    }
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitRMRK = msg::load().expect("Unable to decode InitRMRK");
    let rmrk = RMRKToken {
        name: config.name,
        symbol: config.symbol,
        ..RMRKToken::default()
    };
    RMRK = Some(rmrk);
}

#[gstd::async_main]
async unsafe fn main() {
    let action: RMRKAction = msg::load().expect("Could not load msg");
    let rmrk = unsafe { RMRK.get_or_insert(RMRKToken::default()) };
    match action {
        RMRKAction::MintToNft {
            to,
            token_id,
            destination_id,
        } => rmrk.mint_to_nft(&to, token_id, destination_id).await,
        RMRKAction::MintToRootOwner { to, token_id } => rmrk.mint_to_root_owner(&to, token_id),
        RMRKAction::Transfer { to, token_id } => rmrk.transfer(&to, token_id).await,
        RMRKAction::TransferToNft {
            to,
            destination_id,
            token_id,
        } => rmrk.transfer_to_nft(&to, destination_id, token_id).await,
        RMRKAction::Approve { to, token_id } => rmrk.approve(&to, token_id).await,
        RMRKAction::AddChild {
            parent_token_id,
            child_token_id,
        } => rmrk.add_child(parent_token_id, child_token_id).await,
        RMRKAction::AcceptChild {
            parent_token_id,
            child_contract_id,
            child_token_id,
        } => {
            rmrk.accept_child(parent_token_id, &child_contract_id, child_token_id)
                .await
        }
        RMRKAction::AddAcceptedChild {
            parent_token_id,
            child_token_id,
        } => {
            rmrk.add_accepted_child(parent_token_id, child_token_id)
                .await
        }
        RMRKAction::TransferChild {
            from,
            to,
            child_token_id,
        } => rmrk.transfer_child(from, to, child_token_id).await,
        RMRKAction::RejectChild {
            parent_token_id,
            child_contract_id,
            child_token_id,
        } => {
            rmrk.reject_child(parent_token_id, &child_contract_id, child_token_id)
                .await
        }
        RMRKAction::RemoveChild {
            parent_token_id,
            child_contract_id,
            child_token_id,
        } => {
            rmrk.remove_child(parent_token_id, &child_contract_id, child_token_id)
                .await
        }
        RMRKAction::BurnChild {
            parent_token_id,
            child_token_id,
        } => rmrk.burn_child(parent_token_id, child_token_id),
        RMRKAction::BurnFromParent {
            child_token_ids,
            root_owner,
        } => rmrk.burn_from_parent(child_token_ids, &root_owner).await,
        RMRKAction::Burn { token_id } => rmrk.burn(token_id).await,
        RMRKAction::RootOwner { token_id } => rmrk.root_owner(token_id).await,
        RMRKAction::Owner { token_id } => {
            let rmrk_owner = rmrk
                .rmrk_owners
                .get(&token_id)
                .expect("RMRK: Token does not exist");
            msg::reply(
                RMRKEvent::Owner {
                    token_id: rmrk_owner.token_id,
                    owner_id: rmrk_owner.owner_id,
                },
                0,
            )
            .unwrap();
        }
        RMRKAction::PendingChildren { token_id } => {
            let children = rmrk.get_pending_children(token_id);
            msg::reply(RMRKEvent::PendingChildren { children }, 0).unwrap();
        }
        RMRKAction::AcceptedChildren { token_id } => {
            let children = rmrk.get_accepted_children(token_id);
            msg::reply(RMRKEvent::AcceptedChildren { children }, 0).unwrap();
        }
    }
}
