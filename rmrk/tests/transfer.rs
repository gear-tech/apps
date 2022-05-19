use codec::Encode;
use gtest::{Program, System};
use rmrk_io::*;
mod utils;
use utils::*;

#[test]
fn transfer() {
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

    let res = transfer_utils(&rmrk, USERS[1], USERS[2], 0.into());
    assert!(res.contains(&(
        USERS[1],
        RMRKEvent::Transfer {
            to: USERS[2].into(),
            token_id: 0.into(),
        }
        .encode()
    )));

    let res = owner(&rmrk, 0.into());
    assert!(res.contains(&(
        10,
        RMRKEvent::Owner {
            token_id: None,
            owner_id: USERS[2].into(),
        }
        .encode()
    )));
}

#[test]
fn transfer_with_child() {
    let sys = System::new();
    init_rmrk(&sys);
    init_rmrk(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);
    // mint parent NFT
    assert!(
        !mint_to_root_owner(&rmrk_parent, USERS[0], USERS[1], PARENT_TOKEN_ID.into()).main_failed()
    );
    // mint child NFT
    let res = mint_to_nft(
        &rmrk_child,
        USERS[1],
        PARENT_NFT_CONTRACT,
        0.into(),
        PARENT_TOKEN_ID.into(),
    );
    println!("{:?}", res.decoded_log::<RMRKEvent>());
    assert!(res.contains(&(
        USERS[1],
        RMRKEvent::MintToNft {
            to: PARENT_NFT_CONTRACT.into(),
            token_id: 0.into(),
            destination_id: PARENT_TOKEN_ID.into(),
        }
        .encode()
    )));

    let res = get_root_owner(&rmrk_child, 0.into());
    assert!(res.contains(&(
        10,
        RMRKEvent::RootOwner {
            root_owner: USERS[1].into(),
        }
        .encode()
    )));

    // from user1 to user2
    let res = transfer_utils(&rmrk_parent, USERS[1], USERS[2], 0.into());
    assert!(res.contains(&(
        USERS[1],
        RMRKEvent::Transfer {
            to: USERS[2].into(),
            token_id: 0.into(),
        }
        .encode()
    )));

    let res = owner(&rmrk_parent, 0.into());
    assert!(res.contains(&(
        10,
        RMRKEvent::Owner {
            token_id: None,
            owner_id: USERS[2].into(),
        }
        .encode()
    )));

    let res = get_root_owner(&rmrk_child, 0.into());
    assert!(res.contains(&(
        10,
        RMRKEvent::RootOwner {
            root_owner: USERS[2].into(),
        }
        .encode()
    )));
}

#[test]
fn transfer_to_nft() {
    let sys = System::new();
    init_rmrk(&sys);
    init_rmrk(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);
    // mint parent NFT contract to root owner
    assert!(!mint_to_root_owner(&rmrk_parent, USERS[0], USERS[1], 0.into()).main_failed());

    // mint child NFT contract to root owner at first
    assert!(!mint_to_root_owner(&rmrk_child, USERS[0], USERS[2], 0.into()).main_failed());

    // transfer NFT, that is now NFT becomes a child
    let res = transfer_to_nft_utils(
        &rmrk_child,
        USERS[2],
        PARENT_NFT_CONTRACT,
        0.into(),
        0.into(),
    );
    println!("{:?}", res.decoded_log::<RMRKEvent>());
    assert!(res.contains(&(
        USERS[2],
        RMRKEvent::Transfer {
            to: PARENT_NFT_CONTRACT.into(),
            token_id: 0.into(),
        }
        .encode()
    )));
    let res = owner(&rmrk_child, 0.into());
    assert!(res.contains(&(
        10,
        RMRKEvent::Owner {
            token_id: Some(0.into()),
            owner_id: PARENT_NFT_CONTRACT.into(),
        }
        .encode()
    )));
}

#[test]
fn transfer_to_nft_with_parent() {
    let sys = System::new();
    init_rmrk(&sys);
    init_rmrk(&sys);
    init_rmrk(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);
    let rmrk_future_parent = sys.get_program(3);
    // mint parent NFT
    assert!(
        !mint_to_root_owner(&rmrk_parent, USERS[1], USERS[2], PARENT_TOKEN_ID.into()).main_failed()
    );
    // mint child NFT
    let res = mint_to_nft(
        &rmrk_child,
        USERS[1],
        PARENT_NFT_CONTRACT,
        CHILD_TOKEN_ID.into(),
        PARENT_TOKEN_ID.into(),
    );
    assert!(res.contains(&(
        USERS[1],
        RMRKEvent::MintToNft {
            to: PARENT_NFT_CONTRACT.into(),
            token_id: CHILD_TOKEN_ID.into(),
            destination_id: PARENT_TOKEN_ID.into(),
        }
        .encode()
    )));

    assert!(!mint_to_root_owner(&rmrk_future_parent, USERS[0], USERS[3], 0.into()).main_failed());

    let res = transfer_to_nft_utils(
        &rmrk_child,
        USERS[2],
        NEW_PARENT_NFT_CONTRACT,
        PARENT_TOKEN_ID.into(),
        CHILD_TOKEN_ID.into(),
    );
    assert!(res.contains(&(
        USERS[2],
        RMRKEvent::Transfer {
            to: NEW_PARENT_NFT_CONTRACT.into(),
            token_id: 0.into(),
        }
        .encode()
    )));

    let res = owner(&rmrk_child, 0.into());
    assert!(res.contains(&(
        10,
        RMRKEvent::Owner {
            token_id: Some(0.into()),
            owner_id: NEW_PARENT_NFT_CONTRACT.into(),
        }
        .encode()
    )));
}

#[test]
fn transfer_failures() {}
