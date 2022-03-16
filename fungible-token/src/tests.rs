use codec::Encode;
use ft_io::*;
use gstd::String;
use gtest::{Program, System};
const USERS: &'static [u64] = &[3, 4, 5];

fn init_with_mint(sys: &System) {
    sys.init_logger();

    let ft = Program::current(&sys);

    let res = ft.send(
        USERS[0],
        InitConfig {
            name: String::from("MyToken"),
            symbol: String::from("MTK"),
        },
    );

    assert!(res.log().is_empty());

    let res = ft.send(USERS[0], Action::Mint(1000000));
    assert!(res.contains(&(
        USERS[0],
        Event::Transfer {
            from: 0.into(),
            to: USERS[0].into(),
            amount: 1000000,
        }
        .encode()
    )));
}

#[test]
fn mint() {
    let sys = System::new();
    init_with_mint(&sys);
    let ft = sys.get_program(1);
    let res = ft.send(USERS[0], Action::BalanceOf(USERS[0].into()));
    assert!(res.contains(&(USERS[0], Event::Balance(1000000).encode())));
}

#[test]
fn burn() {
    let sys = System::new();
    init_with_mint(&sys);
    let ft = sys.get_program(1);
    let res = ft.send(USERS[0], Action::Burn(1000));
    assert!(res.contains(&(
        USERS[0],
        Event::Transfer {
            from: USERS[0].into(),
            to: 0.into(),
            amount: 1000,
        }
        .encode()
    )));
    let res = ft.send(USERS[0], Action::BalanceOf(USERS[0].into()));
    assert!(res.contains(&(USERS[0], Event::Balance(999000).encode())));
}

#[test]
fn burn_failures() {
    let sys = System::new();
    sys.init_logger();
    init_with_mint(&sys);
    let ft = sys.get_program(1);
    // must fail since the amount > the user balance
    let res = ft.send(USERS[0], Action::Burn(1000001));
    assert!(res.main_failed());
}

#[test]
fn transfer() {
    let sys = System::new();
    init_with_mint(&sys);
    let ft = sys.get_program(1);
    let res = ft.send(
        USERS[0],
        Action::Transfer {
            from: USERS[0].into(),
            to: USERS[1].into(),
            amount: 500,
        },
    );

    assert!(res.contains(&(
        USERS[0],
        Event::Transfer {
            from: USERS[0].into(),
            to: USERS[1].into(),
            amount: 500,
        }
        .encode()
    )));

    // check that the balance of `USER[0]` decreased and the balance of `USER[1]` increased
    let res = ft.send(USERS[0], Action::BalanceOf(USERS[0].into()));
    assert!(res.contains(&(USERS[0], Event::Balance(999500).encode())));
    let res = ft.send(USERS[0], Action::BalanceOf(USERS[1].into()));
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
        Action::Transfer {
            from: USERS[0].into(),
            to: USERS[1].into(),
            amount: 2000000,
        },
    );
    assert!(res.main_failed());

    //must fail transfer to zero address
    let res = ft.send(
        USERS[2],
        Action::Transfer {
            from: USERS[0].into(),
            to: 0.into(),
            amount: 100,
        },
    );
    assert!(res.main_failed());
}

#[test]
fn approve_and_transfer() {
    let sys = System::new();
    init_with_mint(&sys);
    let ft = sys.get_program(1);

    let res = ft.send(
        USERS[0],
        Action::Approve {
            to: USERS[1].into(),
            amount: 500,
        },
    );
    assert!(res.contains(&(
        USERS[0],
        Event::Approve {
            from: USERS[0].into(),
            to: USERS[1].into(),
            amount: 500,
        }
        .encode()
    )));

    let res = ft.send(
        USERS[1],
        Action::Transfer {
            from: USERS[0].into(),
            to: USERS[2].into(),
            amount: 200,
        },
    );
    assert!(res.contains(&(
        USERS[1],
        Event::Transfer {
            from: USERS[0].into(),
            to: USERS[2].into(),
            amount: 200,
        }
        .encode()
    )));

    // check that the balance of `USER[0]` decreased and the balance of `USER[1]` increased
    let res = ft.send(USERS[0], Action::BalanceOf(USERS[0].into()));
    assert!(res.contains(&(USERS[0], Event::Balance(999800).encode())));
    let res = ft.send(USERS[0], Action::BalanceOf(USERS[2].into()));
    assert!(res.contains(&(USERS[0], Event::Balance(200).encode())));

    // must fail since not enough allowance
    let res = ft.send(
        USERS[1],
        Action::Transfer {
            from: USERS[0].into(),
            to: USERS[2].into(),
            amount: 800,
        },
    );
    assert!(res.main_failed());
}
