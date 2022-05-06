#![no_std]

use codec::Encode;
use gstd::{debug, exec, msg, prelude::*, ActorId};
use primitive_types::U256;
use rmrk_io::*;
const ZERO_ID: ActorId = ActorId::new([0u8; 32]);

#[derive(Debug)]
pub struct RMRKOwner {
    pub token_id: Option<TokenId>,
    pub owner_id: ActorId,
}

#[derive(Debug, Clone)]
pub struct Child {
    child_token_address: ActorId,
    token_id: TokenId,
    status: ChildStatus,
}

#[derive(Debug, Clone)]
pub struct ChildStatus{
    Pending,
    Accepted,
}

#[derive(Debug, Default)]
pub struct RMRKToken {
    pub name: String,
    pub symbol: String,
    pub token_approvals: BTreeMap<TokenId, ActorId>,
    pub rmrk_owners: BTreeMap<TokenId, RMRKOwner>,
    pub children: BTreeMap<TokenId, Vec<Child>>,
    pub balances: BTreeMap<ActorId, u128>,
}

static mut RMRK: Option<RMRKToken> = None;

impl RMRKToken {
    async fn mint_to_nft(
        &mut self,
        to: &ActorId,
        token_id: TokenId,
        destination_id: TokenId,
    ) {
        self.assert_token_exists(token_id);
        //check that `to` is a deployed program
        self.assert_check_rmrk_implementation(to).await;

        let root_owner: RMRKEvent = msg::send_and_wait_for_reply(
            *to,
            RMRKAction::RootOwner {
                token_id: destination_id,
            },
            0,
        )
        .unwrap()
        .await
        .expect("Error in message to nft contract");

        self.rmrk_owners.insert(
            token_id,
            RMRKOwner {
                token_id: Some(destination_id),
                owner_id: *to,
            },
        );
       
        let _response: RMRKEvent = msg::send_and_wait_for_reply(
            *to,
            RMRKAction::AddChild {
                parent_token_id: destination_id,
                child_token_id: token_id,
                child_token_address: exec::program_id(),
            },
            0,
        )
        .unwrap()
        .await
        .expect("Error in adding child to nft contract");

        self.balances
            .entry(*to)
            .and_modify(|balance| *balance += 1)
            .or_insert(1);
        msg::reply(
            RMRKEvent::MintToNft {
                to: *to,
                token_id,
                destination_id,
            },
            0,
        )
        .unwrap();
    }

    fn mint_to_root_owner(&mut self, to: &ActorId, token_id: TokenId) {
        self.assert_zero_address(to);
        self.assert_token_exists(token_id);

        self.balances
            .entry(*to)
            .and_modify(|balance| *balance += 1)
            .or_insert(1);
            
        self.rmrk_owners.insert(
            token_id,
            RMRKOwner {
                token_id: None,
                owner_id: *to,
            },
        );

        msg::reply(
            RMRKEvent::MintToRootOwner {
                to: *to,
                token_id,
            },
            0,
        )
        .unwrap();

    }

    async fn add_child(
        &mut self,
        parent_token_id: TokenId,
        child_token_id: TokenId,
        child_token_address: ActorId,
    ) {
        //check that `msg::source()` is a deployed program
        self.assert_parent(child_token_id).await;
        let child = Child {
            child_token_address: msg::source(),
            token_id: parent_token_id,
            status: ChildStatus::Pending,
        };
        self.children
            .entry(parent_token_id)
            .and_modify(|children| children.push(child.clone()))
            .or_insert_with(|| vec![child]);
        
        msg::reply(
            RMRKEvent::PendingChild {
                child_token_address:  msg::source(),
                child_token_id,
                parent_token_id,
            },
            0,
        )
        .unwrap();

    }

    /// Checks that NFT with indicated ID already exists
    fn assert_token_exists(&self, token_id: TokenId) {
        if self.rmrk_owners.contains_key(&token_id) {
            panic!("RMRK: Token already exists");
        }
    }

    fn nft_parent(&self, token_id: TokenId) {
        let rmrk_owner = self
            .rmrk_owners
            .get(&token_id)
            .expect("Token does not exist");
        debug!("here {:?}", rmrk_owner);
        msg::reply(
            RMRKEvent::NFTParent {
                parent: rmrk_owner.owner_id,
            },
            0,
        )
        .unwrap();
    }

    fn check_rmrk_implementation(&self) {
        msg::reply(
            RMRKEvent::CheckRMRKImplementation,
            0,
        )
        .unwrap();
    }

    fn root_owner(&self, token_id: TokenId) {
        let root_owner = self.rmrk_owners.get(&token_id).expect("RMRK: Token does not exist");
        msg::reply(
            RMRKEvent::RootOwner {root_owner: root_owner.owner_id},
            0,
        )
        .unwrap();
    }

    fn assert_zero_address(&self, account: &ActorId) {
        if account == &ZERO_ID {
            panic!("RMRK: Zero address");
        }
    }

    async fn assert_check_rmrk_implementation(&self, to: &ActorId) {
        let response: RMRKEvent =
            msg::send_and_wait_for_reply(*to, RMRKAction::CheckRMRKImplementation, 0)
                .unwrap()
                .await
                .expect("Error in checking of RMRK implementation");
        match response {
            RMRKEvent::CheckRMRKImplementation => (),
            _ => panic!("RMRKCore: Mint to non-RMRKCore implementer"),
        }
    }

    async fn assert_parent(&self, token_id: TokenId) {
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
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitRMRK = msg::load().expect("Unable to decode InitRMRK");
    let mut rmrk = RMRKToken::default();
    rmrk.name = config.name;
    rmrk.symbol = config.symbol;
    RMRK = Some(rmrk);
}

#[gstd::async_main]
async unsafe fn main() {
    let action: RMRKAction = msg::load().expect("Could not load msg");
    let rmrk = unsafe { RMRK.get_or_insert(RMRKToken::default()) };
    match action {
        RMRKAction::MintToNft { to, token_id, destination_id } => rmrk.mint_to_nft(&to, token_id, destination_id).await,
        RMRKAction::MintToRootOwner { to, token_id } => rmrk.mint_to_root_owner(&to, token_id),
        RMRKAction::AddChild { parent_token_id, child_token_id, child_token_address } => rmrk.add_child(parent_token_id, child_token_id, child_token_address).await,
        RMRKAction::NFTParent { token_id } => rmrk.nft_parent(token_id),
        RMRKAction::CheckRMRKImplementation => rmrk.check_rmrk_implementation(),
        RMRKAction::RootOwner { token_id } => rmrk.root_owner(token_id),
    }
}