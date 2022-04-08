#[cfg(test)]
extern crate std;
#[cfg(test)]
use std::println;

use codec::Encode;
use ft_io::*;
use gstd::String;
use gtest::{Program, System};
use lt_io::*;
const USERS: &'static [u64] = &[1, 2, 3, 4, 5];

fn init_lottery(sys: &System) {
    let lt = Program::current(&sys);

    let res = lt.send_bytes_with_value(USERS[2], b"Init", 10000);

    assert!(res.log().is_empty());
}

fn init_fungible_token(sys: &System) {
    let ft = Program::from_file(
        &sys,
        "../target/wasm32-unknown-unknown/release/fungible_token.wasm",
    );

    let res = ft.send(
        USERS[2],
        InitConfig {
            name: String::from("MyToken"),
            symbol: String::from("MTK"),
        },
    );

    assert!(res.log().is_empty());

    let res = ft.send(USERS[3], Action::Mint(1000));
    assert!(!res.main_failed());

    let res = ft.send(USERS[2], Action::BalanceOf(USERS[3].into()));
    assert!(res.contains(&(USERS[2], Event::Balance(1000).encode())));

    let res = ft.send(USERS[4], Action::Mint(2000));
    assert!(!res.main_failed());

    let res = ft.send(USERS[2], Action::BalanceOf(USERS[4].into()));
    assert!(res.contains(&(USERS[2], Event::Balance(2000).encode())));
}

#[test]
fn enter() {
    let sys = System::new();
    init_fungible_token(&sys);
    init_lottery(&sys);
    sys.init_logger();
    let ft = sys.get_program(1);
    let lt = sys.get_program(2);

    let res = lt.send(
        USERS[2],
        LtAction::StartLottery {
            duration: 5000,
            token_address: Some(USERS[0].into()),
        },
    );
    assert!(res.log().is_empty());

    let res = ft.send(USERS[2], Action::TotalSupply);
    println!("TotalSupply(u128): {:?}", res.decoded_log::<Event>());
    assert!(res.contains(&(USERS[2], Event::TotalSupply(3000).encode())));

    let res = lt.send_with_value(USERS[3], LtAction::Enter(1000), 1000);
    assert!(res.contains(&(USERS[3], LtEvent::PlayerAdded(0).encode())));

    let res = ft.send(USERS[2], Action::BalanceOf(USERS[1].into()));
    println!("Balance(u128): {:?}", res.decoded_log::<Event>());
    assert!(res.contains(&(USERS[2], Event::Balance(1000).encode())));

    let res = lt.send_with_value(USERS[4], LtAction::Enter(2000), 2000);
    assert!(res.contains(&(USERS[4], LtEvent::PlayerAdded(1).encode())));

    let res = ft.send(USERS[2], Action::BalanceOf(USERS[1].into()));
    println!("Balance(u128): {:?}", res.decoded_log::<Event>());
    assert!(res.contains(&(USERS[2], Event::Balance(3000).encode())));
}

#[test]
fn leave_lottery() {
    let sys = System::new();
    init_fungible_token(&sys);
    init_lottery(&sys);
    sys.init_logger();
    let ft = sys.get_program(1);
    let lt = sys.get_program(2);

    let res = lt.send(
        USERS[2],
        LtAction::StartLottery {
            duration: 5000,
            token_address: Some(USERS[0].into()),
        },
    );
    assert!(res.log().is_empty());

    let res = lt.send_with_value(USERS[3], LtAction::Enter(1000), 1000);
    assert!(res.contains(&(USERS[3], LtEvent::PlayerAdded(0).encode())));

    let res = lt.send_with_value(USERS[4], LtAction::Enter(2000), 2000);
    assert!(res.contains(&(USERS[4], LtEvent::PlayerAdded(1).encode())));

    let res = lt.send(USERS[4], LtAction::LeaveLottery(1));
    assert!(!res.main_failed());

    let res = ft.send(USERS[2], Action::BalanceOf(USERS[4].into()));
    assert!(res.contains(&(USERS[2], Event::Balance(2000).encode())));
}

#[test]
fn pick_winner() {
    let sys = System::new();
    init_fungible_token(&sys);
    init_lottery(&sys);
    sys.init_logger();
    let ft = sys.get_program(1);
    let lt = sys.get_program(2);

    let res = lt.send(
        USERS[2],
        LtAction::StartLottery {
            duration: 5000,
            token_address: Some(USERS[0].into()),
        },
    );
    assert!(res.log().is_empty());

    let res = lt.send_with_value(USERS[3], LtAction::Enter(1000), 1000);
    assert!(res.contains(&(USERS[3], LtEvent::PlayerAdded(0).encode())));

    let res = lt.send_with_value(USERS[4], LtAction::Enter(2000), 2000);
    assert!(res.contains(&(USERS[4], LtEvent::PlayerAdded(1).encode())));

    sys.spend_blocks(5000);

    let res = lt.send(USERS[2], LtAction::PickWinner);

    println!("Winner index: {:?}", res.decoded_log::<LtEvent>());
    assert!(
        res.contains(&(USERS[2], LtEvent::Winner(0).encode()))
            || res.contains(&(USERS[2], LtEvent::Winner(1).encode()))
    );

    let res = ft.send(USERS[2], Action::BalanceOf(USERS[1].into()));
    println!("Balance(u128): {:?}", res.decoded_log::<Event>());
    assert!(res.contains(&(USERS[2], Event::Balance(0).encode())));
}
