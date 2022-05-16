#![no_std]

use codec::Encode;
use gstd::{msg, prelude::*, ActorId};
use rmrk_io::*;
pub mod approvals;
pub mod burn;
pub mod checks;
pub mod children;
pub mod messages;
use messages::*;
pub mod mint;
const ZERO_ID: ActorId = ActorId::new([0u8; 32]);

#[derive(Debug)]
pub struct RMRKOwner {
    pub token_id: Option<TokenId>,
    pub owner_id: ActorId,
    pub root_owner: ActorId,
}

#[derive(Debug, Clone)]
pub struct Child {
    token_id: ActorId,
    status: ChildStatus,
}

#[derive(Debug, Default)]
pub struct RMRKToken {
    pub name: String,
    pub symbol: String,
    pub token_approvals: BTreeMap<TokenId, Vec<ActorId>>,
    pub rmrk_owners: BTreeMap<TokenId, RMRKOwner>,
    pub children: BTreeMap<TokenId, BTreeMap<TokenId, Child>>,
    pub balances: BTreeMap<ActorId, u128>,
}

static mut RMRK: Option<RMRKToken> = None;

impl RMRKToken {
    fn nft_parent(&self, token_id: TokenId) {
        let rmrk_owner = self
            .rmrk_owners
            .get(&token_id)
            .expect("Token does not exist");
        msg::reply(
            RMRKEvent::NFTParent {
                parent: rmrk_owner.owner_id,
            },
            0,
        )
        .unwrap();
    }

    fn root_owner(&self, token_id: TokenId) {
        let rmrk_owner = self
            .rmrk_owners
            .get(&token_id)
            .expect("RMRK: Token does not exist");
        msg::reply(
            RMRKEvent::RootOwner {
                root_owner: rmrk_owner.root_owner,
            },
            0,
        )
        .unwrap();
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
        } => rmrk.transfer_to_nft(&to, destination_id, token_id),
        RMRKAction::Approve { to, token_id } => rmrk.approve(&to, token_id).await,
        RMRKAction::AddChild {
            parent_token_id,
            child_token_id,
        } => rmrk.add_child(parent_token_id, child_token_id).await,
        RMRKAction::AcceptChild {
            parent_token_id,
            child_token_id,
        } => rmrk.accept_child(parent_token_id, child_token_id),
        // my implementation
        RMRKAction::BurnChild {
            parent_token_id,
            child_token_id,
        } => rmrk.burn_child(parent_token_id, child_token_id),
        RMRKAction::Burn { token_id } => rmrk.burn(token_id).await,
        RMRKAction::RejectChild {
            parent_token_id,
            child_token_id,
        } => rmrk.reject_child(parent_token_id, child_token_id),
        RMRKAction::RemoveChild {
            parent_token_id,
            child_token_id,
        } => rmrk.remove_child(parent_token_id, child_token_id),
        RMRKAction::AddChildAccepted {
            parent_token_id,
            child_token_id,
        } => {
            rmrk.add_accepted_child(parent_token_id, child_token_id)
                .await
        }
        RMRKAction::NFTParent { token_id } => rmrk.nft_parent(token_id),
        RMRKAction::RootOwner { token_id } => rmrk.root_owner(token_id),
    }
}
