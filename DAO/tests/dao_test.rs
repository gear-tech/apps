use gtest::{Program, System, Log, RunResult};
use dao_io::*;
use fungible_token_messages::*;
use codec::Encode;

fn init_fungible_token<'a>(sys: &'a System) -> Program<'a> {
    sys.init_logger();
    let ft = Program::from_file(
        &sys,
        "../../apps/target/wasm32-unknown-unknown/release/fungible_token.wasm",
    );

    let res = ft.send(InitConfig {
        name: String::from("MyToken"),
        symbol: String::from("MTK"),
    });

    assert!(res.log().is_empty());

    let res = ft.send(Action::Mint(MintInput{
        account: 4.into(),
        amount: 10000000,
    }));
    assert!(res.contains(&(gtest::DEFAULT_USER, Event::Transfer(
        TransferReply {
            from: 0.into(),
            to: 4.into(),
            amount: 10000000,
        }).encode())));

    sys.set_user(4);
    let res = ft.send(Action::Approve(ApproveInput{
        spender: 2.into(),
        amount: 10000000,
    }));
    assert!(res.contains(&(4, Event::Approval(
        ApproveReply {
            owner: 4.into(),
            spender: 2.into(),
            amount: 10000000,
        }).encode())));
    ft
    }

fn init_dao<'a>(sys: &'a System) -> Program<'a> {
    sys.init_logger();
    let dao = Program::from_file(
        &sys,
        "../../apps/target/wasm32-unknown-unknown/release/dao.wasm",
    );
    //assert_eq!(sys.user(), gtest::DEFAULT_USER);
    // let log = Log::builder().source(1).dest(3).payload(DaoEvent::MemberAddedToWhitelist(4.into()));
    // Log::error_builder() // not allows payload
    // assert_eq!(res.log()[0], Log::builder().source(0).dest(3).payload(DaoEvent::MemberAddedToWhitelist(4.into())));

    let res = dao.send(InitDao {
        admin: 3.into(),
        approved_token_program_id: 1.into(),
        period_duration: 10000000,
        voting_period_length: 100000000,
        grace_period_length: 10000000,
        dilution_bound: 3,
        abort_window: 10000000,
    });

    assert!(res.log().is_empty());
    sys.set_user(3);

    let res = dao.send(DaoAction::AddToWhiteList(4.into()));
    assert!(res.contains(&(3, DaoEvent::MemberAddedToWhitelist(4.into()).encode())));

    dao
    }

fn create_membership_proposal<'a>(dao: &'a Program, proposal_id: u128) {
    let res = dao.send(DaoAction::SubmitMembershipProposal{
        applicant: 4.into(),
        token_tribute: 1000,
        shares_requested: 1000,
        quorum: 80,
        details: "First membership proposal".to_string(),
    });
    assert!(res.contains(
        &(3,
          DaoEvent::SubmitMembershipProposal{
              proposer: 3.into(),
              applicant: 4.into(),
              proposal_id: proposal_id.clone(),
              token_tribute: 1000
          }.encode())));
}

fn vote<'a>(dao: &'a Program, proposal_id: u128, vote: Vote) {
    let res =dao.send(DaoAction::SubmitVote{
        proposal_id: proposal_id.clone(),
        vote: vote.clone(),
    });
    assert!(res.contains(
         &(3,
           DaoEvent::SubmitVote {
            account: 3.into(),
            proposal_id: proposal_id.clone(),
            vote: vote.clone(),
           }.encode()
        )));
}

//#[test]
fn cancel_proposal_failures() {
    let sys = System::new();
    sys.init_logger();
    init_fungible_token(&sys);
    let dao = init_dao(&sys);
    sys.set_user(3);
    create_membership_proposal(&dao, 0);

    // must fail since the proposal doesnt exist
    let res = dao.send(DaoAction::CancelProposal(1));
    assert!(res.main_failed());

    sys.set_user(5);
    // must fail since the caller isn't the proposer
    let res = dao.send(DaoAction::CancelProposal(0));
    assert!(res.main_failed());

    sys.set_user(3);
    // must fail since the voting period isnt over
    let res = dao.send(DaoAction::CancelProposal(0));
    assert!(res.main_failed());

    sys.set_user(4);
    let res = dao.send(DaoAction::Abort(0));
    assert!(!res.main_failed());
    // assert!(res.contains(
    //     &(4,
    //       DaoEvent::Abort {
    //        member: 4.into(),
    //        proposal_id: 0,
    //        amount: 1000,
    //       }.encode()
    //    )));
 }

// #[test]
// fn create_proposal() {
//      let sys = System::new();
//      init_fungible_token(&sys);
//      let dao = init_dao(&sys);
//      sys.set_user(3);
//      create_membership_proposal(&dao, 0);
// }

// #[test]
// fn vote_on_proposal_failures() {
//     let sys = System::new();
//     init_fungible_token(&sys);
//     let dao = init_dao(&sys);
//     sys.set_user(3);
//     create_membership_proposal(&dao, 0);
//     // must fail since the proposal does not exist
//     let res = dao.send(DaoAction::SubmitVote{
//         proposal_id: 1,
//         vote: Vote::Yes,
//     });
//     assert!(res.main_failed());

//     sys.set_user(4);
//     //must fail since the account is not delegate
//     let res = dao.send(DaoAction::SubmitVote{
//         proposal_id: 0,
//         vote: Vote::Yes,
//     });
//     assert!(res.main_failed());

//     sys.spend_blocks(1000000001);
//     sys.set_user(3);
//     // must fail since the voting period has expired
//     let res = dao.send(DaoAction::SubmitVote{
//         proposal_id: 0,
//         vote: Vote::Yes,
//     });
//     assert!(res.main_failed());

//     create_membership_proposal(&dao, 1);
//     create_membership_proposal(&dao, 2);
//     // must fail since the voting period has not started
//     let res = dao.send(DaoAction::SubmitVote{
//         proposal_id: 2,
//         vote: Vote::Yes,
//     });
//     assert!(res.main_failed());
// }

// #[test]
// fn vote_on_proposal() {
//      let sys = System::new();
//      init_fungible_token(&sys);
//      let dao = init_dao(&sys);
//      sys.set_user(3);
//      create_membership_proposal(&dao, 0);
//      vote(&dao, 0, Vote::Yes);

//      // must fail since the account has already voted
//     let res = dao.send(DaoAction::SubmitVote{
//         proposal_id: 0,
//         vote: Vote::Yes,
//     });
//     assert!(res.main_failed());
// }

// #[test]
// fn process_proposal() {
//     let sys = System::new();
//     init_fungible_token(&sys);
//     let dao = init_dao(&sys);
//     sys.set_user(3);
//     create_membership_proposal(&dao, 0);
//     vote(&dao, 0, Vote::Yes);
//     // must fail since voting period is not over
//     let res = dao.send(DaoAction::ProcessProposal(0));
//     assert!(res.main_failed());

//     sys.spend_blocks(1000000001);
//     let res = dao.send(DaoAction::ProcessProposal(0));
//     assert!(res.contains(
//         &(3,
//           DaoEvent::ProcessProposal {
//            applicant: 4.into(),
//            proposal_id: 0,
//            did_pass: true,
//           }.encode()
//        )));

//     create_membership_proposal(&dao, 1);
//     vote(&dao, 1, Vote::No);
//     sys.spend_blocks(1000000001);
//     let res = dao.send(DaoAction::ProcessProposal(1));
//     assert!(res.contains(
//         &(3,
//           DaoEvent::ProcessProposal {
//            applicant: 4.into(),
//            proposal_id: 1,
//            did_pass: false,
//           }.encode()
//        )));
//  }

// #[test]
// fn abort_proposal_failures() {
//     let sys = System::new();
//     init_fungible_token(&sys);
//     let dao = init_dao(&sys);
//     sys.set_user(3);
//     create_membership_proposal(&dao, 0);
//     vote(&dao, 0, Vote::Yes);
//     // must fail since proposal doesn't exist
//     let res = dao.send(DaoAction::Abort(1));
//     assert!(res.main_failed());

//     // must fail the caller must be the applicant
//     let res = dao.send(DaoAction::Abort(0));
//     assert!(res.main_failed());

//     sys.spend_blocks(100000001);
//     // must fail sincle the abort window is over
//     let res = dao.send(DaoAction::Abort(0));
//     assert!(res.main_failed());
// }

// #[test]
// fn abort_proposal() {
//     let sys = System::new();
//     let ft = init_fungible_token(&sys);
//     let dao = init_dao(&sys);
//     sys.set_user(3);
//     create_membership_proposal(&dao, 0);
//     vote(&dao, 0, Vote::Yes);

//     sys.set_user(4);
//     let res = dao.send(DaoAction::Abort(0));
//     assert!(res.contains(
//         &(4,
//           DaoEvent::Abort {
//            member: 4.into(),
//            proposal_id: 0,
//            amount: 1000,
//           }.encode()
//        )));

//     // must fail since the proposal has already been aborted
//     let res = dao.send(DaoAction::Abort(0));
//     assert!(res.main_failed());

//     let res = ft.send(Action::BalanceOf(4.into()));
//     assert!(res.contains(
//         &(4, Event::Balance(10000000).encode())));
// }

// #[test]
// fn cancel_proposal_failures() {
//     let sys = System::new();
//     sys.init_logger();
//     init_fungible_token(&sys);
//     let dao = init_dao(&sys);
//     sys.set_user(3);
//     create_membership_proposal(&dao, 0);

//     // must fail since the proposal doesnt exist
//     let res = dao.send(DaoAction::CancelProposal(1));
//     assert!(res.main_failed());

//     sys.set_user(5);
//     // must fail since the caller isn't the proposer
//     let res = dao.send(DaoAction::CancelProposal(0));
//     assert!(res.main_failed());

//     sys.set_user(3);
//     // must fail since the voting period isnt over
//     let res = dao.send(DaoAction::CancelProposal(0));
//     assert!(res.main_failed());

//     sys.set_user(4);
//     let res = dao.send(DaoAction::Abort(0));
//     assert!(!res.main_failed());
    // assert!(res.contains(
    //     &(4,
    //       DaoEvent::Abort {
    //        member: 4.into(),
    //        proposal_id: 0,
    //        amount: 1000,
    //       }.encode()
    //    )));



//     sys.spend_blocks(1000000001);

//     sys.set_user(3);
//     // must fail since the proposal has been aborted
//     dao.send(DaoAction::CancelProposal(0));
//     sys.assert_run_failed();

//     create_membership_proposal(&sys, &dao, 1);
//     vote(&sys, &dao, 1, Vote::Yes);
//     sys.spend_blocks(1000000001);
//     // must fail since YES votes > NO votes
//     dao.send(DaoAction::CancelProposal(1));
//     sys.assert_run_failed();
 //}

// #[test]
// fn cancel_proposal() {
//     let sys = System::new();
//     let ft = init_fungible_token(&sys);
//     let dao = init_dao(&sys);
//     sys.set_user(3);
//     create_membership_proposal(&sys, &dao, 0);
//     vote(&sys, &dao, 0, Vote::No);
//     sys.spend_blocks(1000000001);

//     dao.send(DaoAction::CancelProposal(0));
//     sys.assert_log(
//         2,
//         DaoEvent::Cancel{
//             member: 3.into(),
//             proposal_id: 0,
//         }
//     );

//     // must fail since the proposal has already been cancelled
//     dao.send(DaoAction::CancelProposal(0));
//     sys.assert_run_failed();

//     ft.send(Action::BalanceOf(4.into()));
//     sys.assert_log(
//         1,
//         Event::Balance(10000000),
//     )
// }
