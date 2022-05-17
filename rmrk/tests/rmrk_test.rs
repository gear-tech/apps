use codec::Encode;
use gtest::{Program, System};
use rmrk_io::*;
mod utils;
use utils::*;

#[test]
fn mint_to_root_owner_success() {
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
}

#[test]
fn mint_to_nft_success() {
    let sys = System::new();
    init_rmrk(&sys);
    init_rmrk(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);
    assert!(!mint_to_root_owner(&rmrk_parent, USERS[0], USERS[1], 0.into()).main_failed());
    let res = mint_to_nft(&rmrk_child, USERS[1], 2, 0.into(), 0.into());
    println!("{:?}", res.decoded_log::<RMRKEvent>());
    assert!(res.contains(&(
        USERS[1],
        RMRKEvent::MintToNft {
            to: 2.into(),
            token_id: 0.into(),
            destination_id: 0.into(),
        }
        .encode()
    )));
}

#[test]
fn mint_to_nft_faiures() {
    let sys = System::new();
    init_rmrk(&sys);
    init_rmrk(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);
    assert!(!mint_to_root_owner(&rmrk_parent, USERS[0], USERS[1], 0.into()).main_failed());
    assert!(!mint_to_nft(&rmrk_child, USERS[1], 2, 0.into(), 0.into()).main_failed());
    // mints already minted token
    assert!(mint_to_nft(&rmrk_child, USERS[1], 2, 0.into(), 0.into()).main_failed());
    // mints to non rmrk core implementer
    // assert!(mint_to_nft(&rmrk_child, USERS[1], 3, 1.into(), 0.into()).main_failed());
    // mints to a non-existent token
    assert!(mint_to_nft(&rmrk_child, USERS[1], 2, 1.into(), 1.into()).main_failed());
}

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
        .encode()
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
        RMRKEvent::Transfer{
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

    // Approval { owner: ActorId([4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), approved_account: ActorId([5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), token_id: 0 }
    // pub const USERS: &'static [u64] = &[3, 4, 5, 6];
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
        RMRKEvent::Transfer{
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
        USERS[2],
        RMRKEvent::Transfer{
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
}

