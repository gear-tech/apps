use codec::Encode;
use gtest::{Program, System};
use rmrk_io::*;
mod utils;
use utils::*;

#[test]
fn transfer_simple() {
    let sys = System::new();
    before_test(&sys);
    let rmrk = sys.get_program(2);
    let token_id: u64 = 9;

    let res = transfer(&rmrk, USERS[0], USERS[3], token_id.into());
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::Transfer {
            to: USERS[3].into(),
            token_id: token_id.into(),
        }
        .encode()
    )));

    let res = owner(&rmrk, token_id.into());
    assert!(res.contains(&(
        10,
        RMRKEvent::Owner {
            token_id: None,
            owner_id: USERS[3].into(),
        }
        .encode()
    )));
}

#[test]
fn transfer_parent_with_child() {
    let sys = System::new();
    before_test(&sys);
    init_rmrk(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);
    let rmrk_grand = sys.get_program(3);
    let child_token_id: u64 = 9;
    let parent_token_id: u64 = 10;
    let grand_token_id: u64 = 11;
    let grand_grand_token_id: u64 = 12;

    // ownership chain is  USERS[0] > parent_token_id > child_token_id > grand_token_id
    rmrk_chain(
        &rmrk_grand,
        &rmrk_child,
        &rmrk_parent,
        grand_token_id,
        child_token_id,
        parent_token_id,
    );

    assert!(!transfer(&rmrk_parent, USERS[0], USERS[3], parent_token_id.into()).main_failed());

    // check root_owner of child_token_id
    let res = get_root_owner(&rmrk_child, child_token_id.into());
    assert!(res.contains(&(
        10,
        RMRKEvent::RootOwner {
            root_owner: USERS[3].into(),
        }
        .encode()
    )));

    // check root_owner of grand_token_id
    let res = get_root_owner(&rmrk_grand, grand_token_id.into());
    assert!(res.contains(&(
        10,
        RMRKEvent::RootOwner {
            root_owner: USERS[3].into(),
        }
        .encode()
    )));
}

// #[test]
// fn transfer_to_nft() {
//     let sys = System::new();
//     init_rmrk(&sys);
//     init_rmrk(&sys);
//     let rmrk_child = sys.get_program(1);
//     let rmrk_parent = sys.get_program(2);
//     // mint parent NFT contract to root owner
//     assert!(!mint_to_root_owner(&rmrk_parent, USERS[0], USERS[1], 0.into()).main_failed());

//     // mint child NFT contract to root owner at first
//     assert!(!mint_to_root_owner(&rmrk_child, USERS[0], USERS[2], 0.into()).main_failed());

//     // transfer NFT, that is now NFT becomes a child
//     let res = transfer_to_nft_utils(
//         &rmrk_child,
//         USERS[2],
//         PARENT_NFT_CONTRACT,
//         0.into(),
//         0.into(),
//     );
//     println!("{:?}", res.decoded_log::<RMRKEvent>());
//     assert!(res.contains(&(
//         USERS[2],
//         RMRKEvent::Transfer {
//             to: PARENT_NFT_CONTRACT.into(),
//             token_id: 0.into(),
//         }
//         .encode()
//     )));
//     let res = owner(&rmrk_child, 0.into());
//     assert!(res.contains(&(
//         10,
//         RMRKEvent::Owner {
//             token_id: Some(0.into()),
//             owner_id: PARENT_NFT_CONTRACT.into(),
//         }
//         .encode()
//     )));
// }

// #[test]
// fn transfer_to_nft_with_parent() {
//     let sys = System::new();
//     init_rmrk(&sys);
//     init_rmrk(&sys);
//     init_rmrk(&sys);
//     let rmrk_child = sys.get_program(1);
//     let rmrk_parent = sys.get_program(2);
//     let rmrk_future_parent = sys.get_program(3);
//     // mint parent NFT
//     assert!(
//         !mint_to_root_owner(&rmrk_parent, USERS[1], USERS[2], PARENT_TOKEN_ID.into()).main_failed()
//     );
//     // mint child NFT
//     let res = mint_to_nft(
//         &rmrk_child,
//         USERS[1],
//         PARENT_NFT_CONTRACT,
//         CHILD_TOKEN_ID.into(),
//         PARENT_TOKEN_ID.into(),
//     );
//     assert!(res.contains(&(
//         USERS[1],
//         RMRKEvent::MintToNft {
//             to: PARENT_NFT_CONTRACT.into(),
//             token_id: CHILD_TOKEN_ID.into(),
//             destination_id: PARENT_TOKEN_ID.into(),
//         }
//         .encode()
//     )));

//     assert!(!mint_to_root_owner(&rmrk_future_parent, USERS[0], USERS[3], 0.into()).main_failed());

//     let res = transfer_to_nft_utils(
//         &rmrk_child,
//         USERS[2],
//         NEW_PARENT_NFT_CONTRACT,
//         PARENT_TOKEN_ID.into(),
//         CHILD_TOKEN_ID.into(),
//     );
//     assert!(res.contains(&(
//         USERS[2],
//         RMRKEvent::Transfer {
//             to: NEW_PARENT_NFT_CONTRACT.into(),
//             token_id: 0.into(),
//         }
//         .encode()
//     )));

//     let res = owner(&rmrk_child, 0.into());
//     assert!(res.contains(&(
//         10,
//         RMRKEvent::Owner {
//             token_id: Some(0.into()),
//             owner_id: NEW_PARENT_NFT_CONTRACT.into(),
//         }
//         .encode()
//     )));
// }

// #[test]
// fn transfer_failures() {}
