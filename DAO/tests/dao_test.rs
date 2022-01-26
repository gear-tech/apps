use gtest::{Program, System};
use dao_io::*;
use fungible_token_messages::*;

fn init_fungible_token<'a>(sys: &'a System) -> Program<'a> {
    sys.init_logger();
    let ft = Program::from_file(
        &sys,
        "../../apps/target/wasm32-unknown-unknown/release/fungible_token.wasm",
    );

    ft.send(InitConfig {
        name: String::from("MyToken"),
        symbol: String::from("MTK"),
    });
    sys.assert_log_empty();

    ft.send(Action::Mint(MintInput{
        account: 4.into(),
        amount: 10000000,
    }));
    sys.assert_run_success();
    sys.set_user(4);
    ft.send(Action::Approve(ApproveInput{
        spender: 2.into(),
        amount: 10000000,
    }));
    sys.assert_run_success();
    ft
    }

fn init_dao<'a>(sys: &'a System) -> Program<'a> {
    sys.init_logger();
    let dao = Program::from_file(
        &sys,
        "../../apps/target/wasm32-unknown-unknown/release/dao.wasm",
    );

    dao.send(InitDao {
        admin: 3.into(),
        approved_token_program_id: 1.into(),
        period_duration: 10000000,
        voting_period_length: 100000000,
        grace_period_length: 10000000,
        dilution_bound: 3,
        abort_window: 10000000,
    });
    
    sys.assert_log_empty();
    sys.set_user(3);
    dao.send(DaoAction::AddToWhiteList(4.into()));
    sys.assert_log(
        2,
        DaoEvent::MemberAddedToWhitelist(4.into())
    ); 
    dao
    }

fn create_membership_proposal<'a>(sys: &'a System, dao: &'a Program, proposal_id: u128) {
    sys.init_logger();
    dao.send(DaoAction::SubmitMembershipProposal{
        applicant: 4.into(),
        token_tribute: 1000,
        shares_requested: 1000,
        quorum: 80,
        details: "First membership proposal".to_string(),
    });
    sys.assert_log(
        2,
        DaoEvent::SubmitMembershipProposal {
            proposer: 3.into(),
            applicant: 4.into(),
            proposal_id: proposal_id.clone(),
            token_tribute: 1000,
        }
    );
}

fn vote<'a>(sys: &'a System, dao: &'a Program, proposal_id: u128, vote: Vote) {
    sys.init_logger();
    dao.send(DaoAction::SubmitVote{
        proposal_id: proposal_id.clone(),
        vote: vote.clone(),
    });
    sys.assert_log(
        2,
        DaoEvent::SubmitVote {
            account: 3.into(),
            proposal_id: proposal_id.clone(),
            vote: vote.clone(),
        }
    );
}

#[test]
fn create_proposal() {
     let sys = System::new();
     init_fungible_token(&sys);
     let dao = init_dao(&sys);
     sys.set_user(3);
     create_membership_proposal(&sys, &dao, 0);
}

#[test]
fn vote_on_proposal_failures() {
    let sys = System::new();
    init_fungible_token(&sys);
    let dao = init_dao(&sys);
    sys.set_user(3);
    create_membership_proposal(&sys, &dao, 0);
    // must fail since the proposal does not exist
    dao.send(DaoAction::SubmitVote{
        proposal_id: 1,
        vote: Vote::Yes,
    });
    sys.assert_run_failed();
    
    sys.set_user(4);
    //must fail since the account is not delegate
    dao.send(DaoAction::SubmitVote{
        proposal_id: 0,
        vote: Vote::Yes,
    });
    sys.assert_run_failed();

    sys.spend_blocks(1000000001);
    sys.set_user(3);
    // must fail since the voting period has expired
    dao.send(DaoAction::SubmitVote{
        proposal_id: 0,
        vote: Vote::Yes,
    });
    sys.assert_run_failed();

    create_membership_proposal(&sys, &dao, 1);
    create_membership_proposal(&sys, &dao, 2);
    // must fail since the voting period has not started
    dao.send(DaoAction::SubmitVote{
        proposal_id: 2,
        vote: Vote::Yes,
    });
    sys.assert_run_failed();
}

#[test]
fn vote_on_proposal() {
     let sys = System::new();
     init_fungible_token(&sys);
     let dao = init_dao(&sys);
     sys.set_user(3);
     create_membership_proposal(&sys, &dao, 0);
     vote(&sys, &dao, 0, Vote::Yes);

     // must fail since the account has already voted
     dao.send(DaoAction::SubmitVote{
        proposal_id: 0,
        vote: Vote::Yes,
    });
    sys.assert_run_failed();
}

#[test]
fn process_proposal() {
    let sys = System::new();
    init_fungible_token(&sys);
    let dao = init_dao(&sys);
    sys.set_user(3);
    create_membership_proposal(&sys, &dao, 0);
    vote(&sys, &dao, 0, Vote::Yes);
    // must fail since voting period is not over
    dao.send(DaoAction::ProcessProposal(0));
    sys.assert_run_failed();

    sys.spend_blocks(1000000001);
    dao.send(DaoAction::ProcessProposal(0));
    sys.assert_log(
        2,
        DaoEvent::ProcessProposal {
            applicant: 4.into(),
            proposal_id: 0,
            did_pass: true,
        }
    );

    create_membership_proposal(&sys, &dao, 1);
    vote(&sys, &dao, 1, Vote::No);
    sys.spend_blocks(1000000001);
    dao.send(DaoAction::ProcessProposal(1));
    sys.assert_log(
        2,
        DaoEvent::ProcessProposal {
            applicant: 4.into(),
            proposal_id: 1,
            did_pass: false,
        }
    );
}

#[test]
fn abort_proposal_failures() {
    let sys = System::new();
    init_fungible_token(&sys);
    let dao = init_dao(&sys);
    sys.set_user(3);
    create_membership_proposal(&sys, &dao, 0);
    vote(&sys, &dao, 0, Vote::Yes);
    // must fail since proposal doesn't exist
    dao.send(DaoAction::Abort(1));
    sys.assert_run_failed();

    // must fail the caller must be the applicant
    dao.send(DaoAction::Abort(0));
    sys.assert_run_failed();

    sys.spend_blocks(100000001);
    // must fail sincle the abort window is over
    dao.send(DaoAction::Abort(0));
    sys.assert_run_failed();
}

#[test]
fn abort_proposal() {
    let sys = System::new();
    let ft = init_fungible_token(&sys);
    let dao = init_dao(&sys);
    sys.set_user(3);
    create_membership_proposal(&sys, &dao, 0);
    vote(&sys, &dao, 0, Vote::Yes);

    sys.set_user(4);
    dao.send(DaoAction::Abort(0));
    sys.assert_log(
        2,
        DaoEvent::Abort {
            member: 4.into(),
            proposal_id: 0,
            amount: 1000,
        }
    );

    // must fail since the proposal has already been aborted
    dao.send(DaoAction::Abort(0));
    sys.assert_run_failed();

    ft.send(Action::BalanceOf(4.into()));
    sys.assert_log(
        1,
        Event::Balance(10000000),
    )
}

#[test]
fn cancel_proposal_failures() {
    let sys = System::new();
    init_fungible_token(&sys);
    let dao = init_dao(&sys);
    sys.set_user(3);
    create_membership_proposal(&sys, &dao, 0);
    
    // must fail since the proposal doesnt exist
    dao.send(DaoAction::CancelProposal(1));
    sys.assert_run_failed();

    sys.set_user(5);
    // must fail since the caller isn't the proposer
    dao.send(DaoAction::CancelProposal(0));
    sys.assert_run_failed();

    // must fail since the voting period isnt over
    dao.send(DaoAction::CancelProposal(0));
    sys.assert_run_failed();
 
    sys.set_user(4);
    dao.send(DaoAction::Abort(0));
    sys.assert_run_success();

    sys.spend_blocks(1000000001);

    sys.set_user(3);
    // must fail since the proposal has been aborted
    dao.send(DaoAction::CancelProposal(0));
    sys.assert_run_failed();

    create_membership_proposal(&sys, &dao, 1);
    vote(&sys, &dao, 1, Vote::Yes);
    sys.spend_blocks(1000000001);
    // must fail since YES votes > NO votes
    dao.send(DaoAction::CancelProposal(1));
    sys.assert_run_failed();
}

#[test]
fn cancel_proposal() {
    let sys = System::new();
    let ft = init_fungible_token(&sys);
    let dao = init_dao(&sys);
    sys.set_user(3);
    create_membership_proposal(&sys, &dao, 0);
    vote(&sys, &dao, 0, Vote::No);
    sys.spend_blocks(1000000001);

    dao.send(DaoAction::CancelProposal(0));
    sys.assert_log(
        2,
        DaoEvent::Cancel{
            member: 3.into(),
            proposal_id: 0,
        }
    );

    // must fail since the proposal has already been cancelled
    dao.send(DaoAction::CancelProposal(0));
    sys.assert_run_failed();

    ft.send(Action::BalanceOf(4.into()));
    sys.assert_log(
        1,
        Event::Balance(10000000),
    )
}