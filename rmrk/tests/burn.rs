use codec::Encode;
use gstd::{ActorId, BTreeMap};
use gtest::{Program, System};
use rmrk_io::*;
mod utils;
use utils::*;

#[test]
fn burn_simple() {
    let sys = System::new();
    before_test(&sys);
    let rmrk = sys.get_program(2);
    let res = burn(&rmrk, USERS[0], 5.into());
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::Transfer {
            to: ZERO_ID.into(),
            token_id: 5.into(),
        }
        .encode()
    )));

    // check that token does not exist (must fail)
    assert!(owner(&rmrk, 5.into()).main_failed());
}

#[test]
fn burn_simple_failures() {
    let sys = System::new();
    before_test(&sys);
    let rmrk = sys.get_program(2);
    // must fail since caller is not owner and not approved
    assert!(burn(&rmrk, USERS[3], 5.into()).main_failed());
}

// #[test]
// fn burn_simple_from_approved_account() {
//     let sys = System::new();
//     before_test(&sys);
//     let rmrk = sys.get_program(2);

//      assert!(!approve(&rmrk, USERS[0], USERS[3], 5.into()).main_failed());

//      let res = burn(&rmrk, USERS[3], 5.into());
//      assert!(res.contains(&(
//          USERS[3],
//          RMRKEvent::Transfer {
//              to: ZERO_ID.into(),
//              token_id: 5.into(),
//          }
//          .encode()
//      )));

//      // check that token does not exist (must fail)
//      assert!(owner(&rmrk, 5.into()).main_failed());
// }

#[test]
fn burn_nested_token() {
    let sys = System::new();
    before_test(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);
    let child_accepted_token_id: u64 = 8;
    let child_pending_token_id: u64 = 9;
    let parent_token_id: u64 = 10;

    // mint child_token_id to parent_token_id
    assert!(!mint_to_nft(
        &rmrk_child,
        USERS[1],
        PARENT_NFT_CONTRACT,
        child_pending_token_id.into(),
        parent_token_id.into(),
    )
    .main_failed());

    // mint child_token_id to parent_token_id
    assert!(!mint_to_nft(
        &rmrk_child,
        USERS[1],
        PARENT_NFT_CONTRACT,
        child_accepted_token_id.into(),
        parent_token_id.into(),
    )
    .main_failed());

    // accept one child

    assert!(!accept_child(
        &rmrk_parent,
        USERS[0],
        parent_token_id,
        CHILD_NFT_CONTRACT,
        child_accepted_token_id,
    )
    .main_failed());
    let res = burn(&rmrk_child, USERS[0], child_pending_token_id.into());
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::Transfer {
            to: ZERO_ID.into(),
            token_id: child_pending_token_id.into(),
        }
        .encode()
    )));
    let res = burn(&rmrk_child, USERS[0], child_accepted_token_id.into());
    assert!(res.contains(&(
        USERS[0],
        RMRKEvent::Transfer {
            to: ZERO_ID.into(),
            token_id: child_accepted_token_id.into(),
        }
        .encode()
    )));

    // check that parent contract has no pending children
    let res = get_pending_children(&rmrk_parent, parent_token_id.into());
    assert!(res.contains(&(
        10,
        RMRKEvent::PendingChildren {
            children: BTreeMap::new()
        }
        .encode()
    )));

    // check that parent contract has no accepted children
    let res = get_accepted_children(&rmrk_parent, parent_token_id.into());
    assert!(res.contains(&(
        10,
        RMRKEvent::AcceptedChildren {
            children: BTreeMap::new()
        }
        .encode()
    )));
}

#[test]
fn burn_nested_token_failures() {
    let sys = System::new();
    before_test(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);
    let child_token_id: u64 = 9;
    let parent_token_id: u64 = 10;

    // mint child_token_id to parent_token_id
    assert!(!mint_to_nft(
        &rmrk_child,
        USERS[1],
        PARENT_NFT_CONTRACT,
        child_token_id.into(),
        parent_token_id.into(),
    )
    .main_failed());

    // must fail since caller is not root owner of the nested child token
    assert!(burn(&rmrk_child, USERS[3], child_token_id.into()).main_failed());
}

// ownership chain is now USERS[0] > parent_token_id > child_token_id > grand_token_id
// in that test child_token_id is burning
// rmrk_child contract must also burn grand_token_id and must be removed from parent_token_id
#[test]
fn recursive_burn_nested_token() {
    let sys = System::new();
    before_test(&sys);
    init_rmrk(&sys);
    let rmrk_child = sys.get_program(1);
    let rmrk_parent = sys.get_program(2);
    let rmrk_grand = sys.get_program(3);
    let child_token_id: u64 = 9;
    let parent_token_id: u64 = 10;
    let grand_token_id: u64 = 11;

    // ownership chain is  USERS[0] > parent_token_id > child_token_id > grand_token_id
    rmrk_chain(
        &rmrk_grand,
        &rmrk_child,
        &rmrk_parent,
        grand_token_id,
        child_token_id,
        parent_token_id,
    );

    // check accepted children of parent_token_id
    let res = get_accepted_children(&rmrk_parent, parent_token_id.into());
    let mut accepted_children: BTreeMap<ActorId, Vec<TokenId>> = BTreeMap::new();
    accepted_children.insert(CHILD_NFT_CONTRACT.into(), vec![child_token_id.into()]);
    assert!(res.contains(&(
        10,
        RMRKEvent::AcceptedChildren {
            children: accepted_children
        }
        .encode()
    )));

    // check accepted children of child_token_id
    let res = get_accepted_children(&rmrk_child, child_token_id.into());
    let mut accepted_children: BTreeMap<ActorId, Vec<TokenId>> = BTreeMap::new();
    accepted_children.insert(3.into(), vec![grand_token_id.into()]);
    assert!(res.contains(&(
        10,
        RMRKEvent::AcceptedChildren {
            children: accepted_children
        }
        .encode()
    )));
    assert!(!burn(&rmrk_child, USERS[0], child_token_id.into()).main_failed());
    // check accepted children of parent_token_id
    let res = get_accepted_children(&rmrk_parent, parent_token_id.into());
    assert!(res.contains(&(
        10,
        RMRKEvent::AcceptedChildren {
            children: BTreeMap::new()
        }
        .encode()
    )));

    // check that child_token_id does not exist (must fail)
    assert!(owner(&rmrk_child, child_token_id.into()).main_failed());

    // check that grand_token_id does not exist (must fail)
    assert!(owner(&rmrk_grand, grand_token_id.into()).main_failed());
}

// ownership chain is now USERS[0] > parent_token_id > child_token_id > grand_token_id
// in that test parent_token_id is burning
// rmrk_child contract must also burn child_token_id and grand_token_id
#[test]
fn recursive_burn_parent_token() {
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

    // burn parent_token_id
    assert!(!burn(&rmrk_parent, USERS[0], parent_token_id.into()).main_failed());

    // check that child_token_id does not exist (must fail)
    assert!(owner(&rmrk_child, child_token_id.into()).main_failed());

    // check that grand_token_id does not exist (must fail)
    assert!(owner(&rmrk_grand, grand_token_id.into()).main_failed());
}
