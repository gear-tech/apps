use codec::Encode;
use gtest::{Program, System};
use rmrk_io::*;
mod utils;
use utils::*;

#[test]
fn burn() {
    // mint first
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

    let res = burn_utils(&rmrk, USERS[1], 0.into());
    assert!(res.contains(&(
        USERS[1],
        RMRKEvent::Transfer {
            to: ZERO_ID.into(),
            token_id: 0.into(),
        }
        .encode()
    )));
}

#[test]
fn burn_with_approval() {
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

    let res = burn_utils(&rmrk, USERS[2], 0.into());
    assert!(res.contains(&(
        USERS[2],
        RMRKEvent::Transfer {
            to: ZERO_ID.into(),
            token_id: 0.into(),
        }
        .encode()
    )));
}

#[test]
fn burn_with_children() {
    let sys = System::new();
    init_rmrk(&sys);
    init_rmrk(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);
    assert!(!mint_to_root_owner(&rmrk_parent, USERS[0], USERS[1], 0.into()).main_failed());
    let res = mint_to_nft(&rmrk_child, USERS[1], 2, 0.into(), 0.into());
    assert!(res.contains(&(
        USERS[1],
        RMRKEvent::MintToNft {
            to: 2.into(),
            token_id: 0.into(),
            destination_id: 0.into(),
        }
        .encode()
    )));

    let res = burn_utils(&rmrk_parent, USERS[1], 0.into());
    assert!(res.contains(&(
        USERS[1],
        RMRKEvent::Transfer {
            to: ZERO_ID.into(),
            token_id: 0.into(),
        }
        .encode()
    )));
}

#[test]
fn burn_failures() {
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

    // no ownership
    assert!(!burn_utils(&rmrk, USERS[3], 0.into()).main_failed());

    // no such token
}
