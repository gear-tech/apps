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

    let res = owner(&rmrk, 0.into());
    assert!(res.contains(&(
        10,
        RMRKEvent::Owner {
            token_id: None,
            owner_id: USERS[1].into(),
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
fn mint_to_nft_faiures() {
    let sys = System::new();
    init_rmrk(&sys);
    init_rmrk(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);
    assert!(
        !mint_to_root_owner(&rmrk_parent, USERS[0], USERS[1], PARENT_TOKEN_ID.into()).main_failed()
    );
    assert!(!mint_to_nft(
        &rmrk_child,
        USERS[1],
        PARENT_NFT_CONTRACT,
        0.into(),
        PARENT_TOKEN_ID.into()
    )
    .main_failed());
    // mints already minted token
    assert!(mint_to_nft(
        &rmrk_child,
        USERS[1],
        PARENT_NFT_CONTRACT,
        0.into(),
        PARENT_TOKEN_ID.into()
    )
    .main_failed());
    // mints to a non-existent token
    assert!(mint_to_nft(
        &rmrk_child,
        USERS[1],
        PARENT_NFT_CONTRACT,
        1.into(),
        1.into()
    )
    .main_failed());
}
