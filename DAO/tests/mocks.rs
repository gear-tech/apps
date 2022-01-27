use codec::{Decode, Encode};
use dao_io::*;
use fungible_token_messages::*;
use gtest::{Log, Program, RunResult, System, WasmProgram};

#[derive(Debug)]
struct FungibleToken;

impl WasmProgram for FungibleToken {
    fn init(&mut self, _: Vec<u8>) -> Result<Option<Vec<u8>>, &'static str> {
        Ok(Some(b"GOT IT".to_vec()))
    }

    fn handle(&mut self, payload: Vec<u8>) -> Result<Option<Vec<u8>>, &'static str> {
        let res = Action::decode(&mut &payload[..]).map_err(|_| "Can't decode")?;

        match res {
            Action::TransferFrom(TransferFromInput {
                owner: _,
                to: _,
                amount: _,
            }) => {
                return Ok(Some(
                    Event::TransferFrom(TransferFromReply {
                        owner: 3.into(),
                        sender: 3.into(),
                        recipient: 3.into(),
                        amount: 10000,
                        new_limit: 1000,
                    })
                    .encode(),
                ));
            }
            _ => return Ok(None),
        }
    }

    fn handle_reply(&mut self, _: Vec<u8>) -> Result<Option<Vec<u8>>, &'static str> {
        Ok(None)
    }
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

    let res = dao.send(
        100001,
        InitDao {
            admin: 3.into(),
            approved_token_program_id: 1.into(),
            period_duration: 10000000,
            voting_period_length: 100000000,
            grace_period_length: 10000000,
            dilution_bound: 3,
            abort_window: 10000000,
        },
    );

    assert!(res.log().is_empty());

    let res = dao.send(3, DaoAction::AddToWhiteList(4.into()));
    assert!(res.contains(&(3, DaoEvent::MemberAddedToWhitelist(4.into()).encode())));

    dao
}

fn create_membership_proposal<'a>(dao: &'a Program, proposal_id: u128) {
    let res = dao.send(
        3,
        DaoAction::SubmitMembershipProposal {
            applicant: 4.into(),
            token_tribute: 1000,
            shares_requested: 1000,
            quorum: 80,
            details: "First membership proposal".to_string(),
        },
    );
    //  assert!(res.main_failed());
    assert!(res.contains(&(
        3,
        DaoEvent::SubmitMembershipProposal {
            proposer: 3.into(),
            applicant: 4.into(),
            proposal_id: proposal_id.clone(),
            token_tribute: 1000
        }
        .encode()
    )));
}
#[test]
fn transfer_from() {
    let sys = System::new();
    sys.init_logger();

    let ft = Program::mock(&sys, FungibleToken);

    let res = ft.send_bytes(100001, "INIT");
    assert!(!res.log().is_empty());

    let action = Action::TransferFrom(TransferFromInput {
        owner: 4.into(),
        to: 4.into(),
        amount: 1000,
    });

    let res = ft.send(100001, action);

    assert!(res.contains(&(
        100001,
        Event::TransferFrom(TransferFromReply {
            owner: 3.into(),
            sender: 3.into(),
            recipient: 3.into(),
            amount: 10000,
            new_limit: 1000,
        })
        .encode()
    )));

    let dao = init_dao(&sys);
    create_membership_proposal(&dao, 0);
}
