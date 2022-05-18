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
        RMRKEvent::Transfer{
            to: USERS[2].into(),
            token_id: 0.into(),
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

    // from user1 to user2
    let res = transfer_utils(&rmrk_parent, USERS[1], USERS[2], 0.into());
    assert!(res.contains(&(
        USERS[1],
        RMRKEvent::Transfer{
            to: USERS[2].into(),
            token_id: 0.into(),
        }
        .encode()
    )));
}

#[test]
fn transfer_to_nft() {
    let sys = System::new();
    init_rmrk(&sys);
    init_rmrk(&sys);
    let rmrk_from = sys.get_program(FROM_NFT_CONTRACT);
    let rmrk_to = sys.get_program(TO_NFT_CONTRACT);
    // mint parent NFT
    assert!(
        !mint_to_root_owner(&rmrk_from, USERS[0], USERS[1], FROM_TOKEN_ID.into()).main_failed()
    );
    assert!(
        !mint_to_root_owner(&rmrk_to, USERS[0], USERS[2], TO_TOKEN_ID.into()).main_failed()
    );

    let res = transfer_to_nft_utils(
        &rmrk_from,
        USERS[1],
        TO_NFT_CONTRACT,
        FROM_TOKEN_ID.into(),
        TO_TOKEN_ID.into(),
    );
    assert!(res.contains(&(
        USERS[1],
        RMRKEvent::Transfer {
            to: TO_NFT_CONTRACT.into(),
            token_id: 0.into(),
        }
        .encode()
    )));
}

#[test]
fn transfer_to_nft_with_child() {
    let sys = System::new();
    init_rmrk(&sys);
    init_rmrk(&sys);
    init_rmrk(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);
    // mint parent NFT
    assert!(
        !mint_to_root_owner(&rmrk_parent, USERS[1], USERS[2], PARENT_TOKEN_ID.into()).main_failed()
    );
    // mint child NFT
    // let res = mint_to_nft(
    //     &rmrk_child,
    //     USERS[1],
    //     PARENT_NFT_CONTRACT,
    //     0.into(),
    //     PARENT_TOKEN_ID.into(),
    // );
    // assert!(res.contains(&(
    //     USERS[1],
    //     RMRKEvent::MintToNft {
    //         to: PARENT_NFT_CONTRACT.into(),
    //         token_id: 0.into(),
    //         destination_id: PARENT_TOKEN_ID.into(),
    //     }
    //     .encode()
    // )));

    // +1 since we added :) TODO: move to constants
    // let rmrk_to = sys.get_program(TO_NFT_CONTRACT + 1);
    // assert!(
    //     !mint_to_root_owner(&rmrk_to, USERS[0], USERS[2], TO_TOKEN_ID.into()).main_failed()
    // );

    // let res = transfer_to_nft_utils(
    //     &rmrk_parent,
    //     USERS[1],
    //     TO_NFT_CONTRACT,
    //     PARENT_TOKEN_ID.into(),
    //     TO_TOKEN_ID.into(),
    // );
    // assert!(res.contains(&(
    //     USERS[1],
    //     RMRKEvent::Transfer {
    //         to: TO_NFT_CONTRACT.into(),
    //         token_id: 0.into(),
    //     }
    //     .encode()
    // )));
}

#[test]
fn transfer_failures() {

}