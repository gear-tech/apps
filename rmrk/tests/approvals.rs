use codec::Encode;
use gtest::{Program, System};
use rmrk_io::*;
mod utils;
use utils::*;

#[test]
fn nft_parent() {
    let sys = System::new();
    init_rmrk(&sys);
    let rmrk = sys.get_program(1);
    let res = mint_to_root_owner(&rmrk, USERS[0], USERS[1], 0.into());
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::MintToRootOwner {
            to: USERS[1].into(),
            token_id: 0.into(),
        }
        .encode()
    )));

    let res = nft_parent_utils(&rmrk, USERS[1], 0.into());
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::NFTParent {
            parent: USERS[1].into(),
        }
        .encode(),
    )));
}

#[test]
fn approve() {
    let sys = System::new();
    init_rmrk(&sys);
    let rmrk = sys.get_program(1);
    let res = mint_to_root_owner(&rmrk, USERS[0], USERS[1], 0.into());
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::MintToRootOwner {
            to: USERS[1].into(),
            token_id: 0.into(),
        }
        .encode()
    )));
    let res = approve_utils(&rmrk, USERS[1], USERS[2], 0.into());
    assert!(res.contains(&(
        USERS[1],
        RMRKEvent::Approval {
            owner: USERS[1].into(),
            approved_account: USERS[2].into(),
            token_id: 0.into(),
        }
        .encode()
    )));
}

#[test]
fn root_owner() {
    let sys = System::new();
    init_rmrk(&sys);
    let rmrk = sys.get_program(1);
    let res = mint_to_root_owner(&rmrk, USERS[0], USERS[1], 0.into());
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::MintToRootOwner {
            to: USERS[1].into(),
            token_id: 0.into(),
        }
        .encode()
    )));

    let res = root_owner_utils(&rmrk, USERS[1], 0.into());
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::RootOwner {
            root_owner: USERS[1].into(),
        }
        .encode()
    )));
}
