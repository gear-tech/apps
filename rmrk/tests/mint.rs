use codec::Encode;
use gstd::{ActorId, BTreeMap};
use gtest::{Program, System};
use rmrk_io::*;
mod utils;
use utils::*;

#[test]
fn mint_to_root_owner_success() {
    let sys = System::new();
    before_test(&sys);
    let rmrk = sys.get_program(1);
    let res = mint_to_root_owner(&rmrk, USERS[0], USERS[0], 0.into());
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::MintToRootOwner {
            to: USERS[0].into(),
            token_id: 0.into(),
        }
        .encode()
    )));

    let res = owner(&rmrk, 0.into());
    assert!(res.contains(&(
        10,
        RMRKEvent::Owner {
            token_id: None,
            owner_id: USERS[0].into(),
        }
        .encode()
    )));
}

#[test]
fn mint_to_root_owner_failures() {
    let sys = System::new();
    before_test(&sys);
    let rmrk_parent = sys.get_program(2);
    // mints already minted token
    assert!(mint_to_root_owner(&rmrk_parent, USERS[1], USERS[1], 1.into(),).main_failed());
    // mints to zero address
    assert!(mint_to_root_owner(&rmrk_parent, USERS[1], ZERO_ID, 1.into(),).main_failed());
}

#[test]
fn mint_to_nft_failures() {
    let sys = System::new();
    before_test(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);
    // mint to a non-contract destination
    // TO DO

    // nest mint to a non-existent token
    assert!(mint_to_nft(
        &rmrk_child,
        USERS[1],
        PARENT_NFT_CONTRACT,
        1.into(),
        100.into()
    )
    .main_failed());

    let child_token_id: u64 = 1;
    let parent_token_id: u64 = 10;
    // mint RMRK child token to RMRK parent token
    assert!(!mint_to_nft(
        &rmrk_child,
        USERS[1],
        PARENT_NFT_CONTRACT,
        child_token_id.into(),
        parent_token_id.into()
    )
    .main_failed());
    // nest mint already minted token
    assert!(mint_to_nft(
        &rmrk_child,
        USERS[1],
        PARENT_NFT_CONTRACT,
        child_token_id.into(),
        parent_token_id.into()
    )
    .main_failed());
    // nest mint already minted token to a different parent
    assert!(mint_to_nft(
        &rmrk_child,
        USERS[1],
        PARENT_NFT_CONTRACT,
        child_token_id.into(),
        12.into()
    )
    .main_failed());
    // nest mint to zero address (TO DO)
    // assert!(mint_to_nft(&rmrk_child, USERS[1], ZERO_ID, 2.into(), 12.into()).main_failed());
}

#[test]
fn mint_to_nft_success() {
    let sys = System::new();
    before_test(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);
    let parent_token_id: u64 = 10;

    let mut pending_children: BTreeMap<ActorId, Vec<TokenId>> = BTreeMap::new();
    // mint  RMRK children
    for child_token_id in 0..10 as u64 {
        let res = mint_to_nft(
            &rmrk_child,
            USERS[1],
            PARENT_NFT_CONTRACT,
            child_token_id.into(),
            parent_token_id.into(),
        );
        assert!(res.contains(&(
            USERS[1],
            RMRKEvent::MintToNft {
                to: PARENT_NFT_CONTRACT.into(),
                token_id: child_token_id.into(),
                destination_id: parent_token_id.into(),
            }
            .encode()
        )));

        // check owner
        let res = owner(&rmrk_child, child_token_id.into());
        assert!(res.contains(&(
            10,
            RMRKEvent::Owner {
                token_id: Some(parent_token_id.into()),
                owner_id: PARENT_NFT_CONTRACT.into(),
            }
            .encode()
        )));

        pending_children
            .entry(1.into())
            .and_modify(|c| c.push(child_token_id.into()))
            .or_insert_with(|| vec![child_token_id.into()]);
    }

    // another RMRK child contract
    init_rmrk(&sys);
    let rmrk_child_2 = sys.get_program(3);
    for child_token_id in 0..20 as u64 {
        let res = mint_to_nft(
            &rmrk_child_2,
            USERS[1],
            PARENT_NFT_CONTRACT,
            child_token_id.into(),
            parent_token_id.into(),
        );
        assert!(res.contains(&(
            USERS[1],
            RMRKEvent::MintToNft {
                to: PARENT_NFT_CONTRACT.into(),
                token_id: child_token_id.into(),
                destination_id: parent_token_id.into(),
            }
            .encode()
        )));

        // check owner
        let res = owner(&rmrk_child_2, child_token_id.into());
        assert!(res.contains(&(
            10,
            RMRKEvent::Owner {
                token_id: Some(parent_token_id.into()),
                owner_id: PARENT_NFT_CONTRACT.into(),
            }
            .encode()
        )));
        pending_children
            .entry(3.into())
            .and_modify(|c| c.push(child_token_id.into()))
            .or_insert_with(|| vec![child_token_id.into()]);
    }
    // check children
    let res = get_pending_children(&rmrk_parent, parent_token_id.into());
    assert!(res.contains(&(
        10,
        RMRKEvent::PendingChildren {
            children: pending_children
        }
        .encode()
    )));
}

#[test]
fn mint_child_to_child() {
    let sys = System::new();
    before_test(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);
    // grand child contract
    init_rmrk(&sys);
    let rmrk_grand_child = sys.get_program(3);
    let parent_token_id: u64 = 10;
    let child_token_id: u64 = 1;
    let grand_child_id: u64 = 2;
    // mint child_token_id to parent_token_id
    let res = mint_to_nft(
        &rmrk_child,
        USERS[1],
        PARENT_NFT_CONTRACT,
        child_token_id.into(),
        parent_token_id.into(),
    );
    assert!(res.contains(&(
        USERS[1],
        RMRKEvent::MintToNft {
            to: PARENT_NFT_CONTRACT.into(),
            token_id: child_token_id.into(),
            destination_id: parent_token_id.into(),
        }
        .encode()
    )));

    // mint grand_token_id to child_token_id

    let res = mint_to_nft(
        &rmrk_grand_child,
        USERS[1],
        CHILD_NFT_CONTRACT,
        grand_child_id.into(),
        child_token_id.into(),
    );
    assert!(res.contains(&(
        USERS[1],
        RMRKEvent::MintToNft {
            to: CHILD_NFT_CONTRACT.into(),
            token_id: grand_child_id.into(),
            destination_id: child_token_id.into(),
        }
        .encode()
    )));
    // root owner of grand_token_id must be USERS[0]
    let res = get_root_owner(&rmrk_grand_child, grand_child_id.into());
    assert!(res.contains(&(
        10,
        RMRKEvent::RootOwner {
            root_owner: USERS[0].into(),
        }
        .encode()
    )));
}
