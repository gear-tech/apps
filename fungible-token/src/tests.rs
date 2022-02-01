use codec::Encode;
use fungible_token_messages::*;
use gstd::String;
use gtest::{Program, System};
const USERS: &'static [u64] = &[3, 4, 5];

fn init_with_mint(sys: &System) {
    sys.init_logger();

    let ft = Program::from_file(
        &sys,
        "../../apps/target/wasm32-unknown-unknown/release/fungible_token.wasm",
    );

    let res = ft.send(
        USERS[0],
        InitConfig {
            name: String::from("MyToken"),
            symbol: String::from("MTK"),
        },
    );

    assert!(res.log().is_empty());

    let res = ft.send(
        USERS[0],
        Action::Mint(MintInput {
            account: USERS[1].into(),
            amount: 1000000,
        }),
    );
    assert!(res.contains(&(
        USERS[0],
        Event::Transfer(TransferReply {
            from: 0.into(),
            to: USERS[1].into(),
            amount: 1000000,
        })
        .encode()
    )));
}

#[test]
fn mint() {
    let sys = System::new();
    init_with_mint(&sys);
}

#[test]
fn mint_failures() {
    let sys = System::new();
    init_with_mint(&sys);
    let ft = sys.get_program(1);
    // must fails since the caller is nor the creator nor the admin
    let res = ft.send(
        USERS[1],
        Action::Mint(MintInput {
            account: USERS[1].into(),
            amount: 1000000,
        }),
    );
    assert!(res.main_failed());

    // must fails since mint to zero address
    let res = ft.send(
        USERS[0],
        Action::Mint(MintInput {
            account: 0.into(),
            amount: 1000000,
        }),
    );
    assert!(res.main_failed());
}

#[test]
fn burn() {
    let sys = System::new();
    init_with_mint(&sys);
    let ft = sys.get_program(1);
    let res = ft.send(
        USERS[0],
        Action::Burn(BurnInput {
            account: USERS[1].into(),
            amount: 1000,
        }),
    );
    assert!(res.contains(&(
        USERS[0],
        Event::Transfer(TransferReply {
            from: USERS[1].into(),
            to: 0.into(),
            amount: 1000,
        })
        .encode()
    )));
}

#[test]
fn burn_failures() {
    let sys = System::new();
    sys.init_logger();
    init_with_mint(&sys);
    let ft = sys.get_program(1);
    let res = ft.send(
        USERS[0],
        Action::Mint(MintInput {
            account: USERS[2].into(),
            amount: 2000000,
        }),
    );
    assert!(!res.main_failed());
    // must fail since the amount > the user balance
    let res = ft.send(
        USERS[0],
        Action::Burn(BurnInput {
            account: USERS[1].into(),
            amount: 2000000,
        }),
    );

    assert!(res.main_failed());
}

#[test]
fn add_admin() {
    let sys = System::new();
    init_with_mint(&sys);
    let ft = sys.get_program(1);
    let res = ft.send(USERS[0], Action::AddAdmin(USERS[1].into()));
    assert!(res.contains(&(USERS[0], Event::AdminAdded(USERS[1].into()).encode())));

    let res = ft.send(
        USERS[1],
        Action::Mint(MintInput {
            account: USERS[1].into(),
            amount: 2000000,
        }),
    );
    assert!(!res.main_failed());
}

#[test]
fn balance_of() {
    let sys = System::new();
    init_with_mint(&sys);
    let ft = sys.get_program(1);
    let res = ft.send(USERS[0], Action::BalanceOf(USERS[1].into()));
    assert!(res.contains(&(USERS[0], Event::Balance(1000000).encode())));
}

#[test]
fn transfer() {
    let sys = System::new();
    init_with_mint(&sys);
    let ft = sys.get_program(1);
    let res = ft.send(
        USERS[1],
        Action::Transfer(TransferInput {
            to: USERS[2].into(),
            amount: 500,
        }),
    );

    assert!(res.contains(&(
        USERS[1],
        Event::Transfer(TransferReply {
            from: USERS[1].into(),
            to: USERS[2].into(),
            amount: 500,
        })
        .encode()
    )));

    // check that the balance of `USER[0]` decreased and the balance of `USER[1]` increased
    let res = ft.send(USERS[0], Action::BalanceOf(USERS[1].into()));
    assert!(res.contains(&(USERS[0], Event::Balance(999500).encode())));
    let res = ft.send(USERS[0], Action::BalanceOf(USERS[2].into()));
    assert!(res.contains(&(USERS[0], Event::Balance(500).encode())));
}

#[test]
fn transfer_failures() {
    let sys = System::new();
    init_with_mint(&sys);
    let ft = sys.get_program(1);
    //must fail since the amount > balance
    let res = ft.send(
        USERS[0],
        Action::Transfer(TransferInput {
            to: USERS[1].into(),
            amount: 2000000,
        }),
    );
    assert!(res.main_failed());

    //must fail transfer to zero address
    let res = ft.send(
        USERS[2],
        Action::Transfer(TransferInput {
            to: 0.into(),
            amount: 100,
        }),
    );
    assert!(res.main_failed());
}

#[test]
fn approve_and_transfer() {
    let sys = System::new();
    init_with_mint(&sys);
    let ft = sys.get_program(1);

    let res = ft.send(
        USERS[1],
        Action::Approve(ApproveInput {
            spender: USERS[2].into(),
            amount: 500,
        }),
    );
    assert!(res.contains(&(
        USERS[1],
        Event::Approval(ApproveReply {
            owner: USERS[1].into(),
            spender: USERS[2].into(),
            amount: 500,
        })
        .encode()
    )));

    let res = ft.send(
        USERS[2],
        Action::TransferFrom(TransferFromInput {
            owner: USERS[1].into(),
            to: USERS[0].into(),
            amount: 200,
        }),
    );

    assert!(res.contains(&(
        USERS[2],
        Event::TransferFrom(TransferFromReply {
            owner: USERS[1].into(),
            sender: USERS[2].into(),
            recipient: USERS[0].into(),
            amount: 200,
            new_limit: 300,
        })
        .encode()
    )));
}

#[test]
fn allowance() {
    let sys = System::new();
    init_with_mint(&sys);
    let ft = sys.get_program(1);

    let res = ft.send(
        USERS[1],
        Action::IncreaseAllowance(ApproveInput {
            spender: USERS[2].into(),
            amount: 500,
        }),
    );
    assert!(res.contains(&(
        USERS[1],
        Event::Approval(ApproveReply {
            owner: USERS[1].into(),
            spender: USERS[2].into(),
            amount: 500,
        })
        .encode()
    )));

    let res = ft.send(
        USERS[2],
        Action::TransferFrom(TransferFromInput {
            owner: USERS[1].into(),
            to: USERS[0].into(),
            amount: 200,
        }),
    );
    assert!(!res.main_failed());

    // must fail since amount > allowance
    let res = ft.send(
        USERS[1],
        Action::DecreaseAllowance(ApproveInput {
            spender: USERS[2].into(),
            amount: 500,
        }),
    );
    assert!(res.main_failed());

    let res = ft.send(
        USERS[1],
        Action::DecreaseAllowance(ApproveInput {
            spender: USERS[2].into(),
            amount: 100,
        }),
    );
    assert!(res.contains(&(
        USERS[1],
        Event::Approval(ApproveReply {
            owner: USERS[1].into(),
            spender: USERS[2].into(),
            amount: 200,
        })
        .encode()
    )));

    // must fail since amount > allowance
    let res = ft.send(
        USERS[2],
        Action::TransferFrom(TransferFromInput {
            owner: USERS[1].into(),
            to: USERS[0].into(),
            amount: 300,
        }),
    );
    assert!(res.main_failed());
}
