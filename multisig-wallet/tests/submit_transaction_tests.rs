use codec::Encode;
use gtest::{Program, RunResult, System};
use multisig_wallet_io::*;
const USERS: &'static [u64] = &[3, 4, 5, 6];

fn common_init<'a>(sys: &'a System, users: &[u64], required: u64) -> Program<'a> {
    sys.init_logger();

    let wallet = Program::from_file(
        &sys,
        "../target/wasm32-unknown-unknown/release/multisig_wallet.wasm",
    );

    wallet.send(
        USERS[0],
        MWInitConfig {
            owners: users.iter().copied().map(|x| x.into()).collect(),
            required,
        },
    );

    wallet
}

#[test]
fn common() {
    let sys = System::new();
    let wallet = common_init(&sys, &USERS[0..3], 3);
    let res = wallet.send_with_value(
        USERS[0],
        MWAction::SubmitTransaction {
            destination: USERS[3].into(),
            data: vec![],
            value: 1000,
        },
        0,
    );

    let expect = MWEvent::Submission {
        transaction_id: 0.into(),
    };

    assert!(res.contains(&(USERS[0], expect.encode())));
}

#[test]
fn submit_several_transactions() {
    let sys = System::new();
    let wallet = common_init(&sys, &USERS[0..3], 3);
    let res = wallet.send_with_value(
        USERS[0],
        MWAction::SubmitTransaction {
            destination: USERS[3].into(),
            data: vec![],
            value: 1000,
        },
        0,
    );

    let expect = MWEvent::Submission {
        transaction_id: 0.into(),
    };

    assert!(res.contains(&(USERS[0], expect.encode())));

    let res = wallet.send_with_value(
        USERS[0],
        MWAction::SubmitTransaction {
            destination: USERS[3].into(),
            data: vec![],
            value: 1000,
        },
        0,
    );

    let expect = MWEvent::Submission {
        transaction_id: 1.into(),
    };

    assert!(res.contains(&(USERS[0], expect.encode())));
}

#[test]
fn submit_and_execute_automatically() {
    let sys = System::new();
    let wallet = common_init(&sys, &USERS[0..3], 1);
    let res = wallet.send_with_value(
        USERS[0],
        MWAction::SubmitTransaction {
            destination: USERS[3].into(),
            data: vec![],
            value: 100,
        },
        1000,
    );

    let expect = MWEvent::Submission {
        transaction_id: 0.into(),
    };
    // contains(&(USERS[0], expect.encode()))
    // assert!(res.main_failed());
    assert!(res.contains(&(USERS[0], expect.encode())));
}
