#[cfg(test)]
extern crate std;
#[cfg(test)]
use std::println;

use codec::Encode;
use gstd::BTreeMap;
use gtest::{Program, System};
use lt_io::*;
const USERS: &'static [u64] = &[3, 4, 5];

fn init(sys: &System) {
    sys.init_logger();

    let ft = Program::current(&sys);

    let res = ft.send_bytes_with_value(USERS[0], b"Init", 10000);

    assert!(res.log().is_empty());
}

#[test]
fn start_lottery() {
    let sys = System::new();
    init(&sys);
    let lt = sys.get_program(1);

    let res = lt.send(USERS[0], Action::StartLottery(20000));
    assert!(res.log().is_empty());

    lt.send(USERS[0], Action::LotteryState);
}

#[test]
fn enter() {
    let sys = System::new();
    init(&sys);
    let lt = sys.get_program(1);

    let res = lt.send(USERS[0], Action::StartLottery(20000));
    assert!(res.log().is_empty());

    let res2 = lt.send_with_value(USERS[0], Action::Enter, 1000);
    assert!(res2.contains(&(USERS[0], Event::PlayerAdded(0).encode())));

    let res3 = lt.send_with_value(USERS[1], Action::Enter, 2000);
    assert!(res3.contains(&(USERS[1], Event::PlayerAdded(1).encode())));
}

#[test]
fn pick_winner() {
    let sys = System::new();
    init(&sys);
    let lt = sys.get_program(1);

    let res = lt.send(USERS[0], Action::StartLottery(5000));
    assert!(res.log().is_empty());

    let res2 = lt.send_with_value(USERS[0], Action::Enter, 1000);
    assert!(res2.contains(&(USERS[0], Event::PlayerAdded(0).encode())));

    let res3 = lt.send_with_value(USERS[1], Action::Enter, 2000);
    assert!(res3.contains(&(USERS[1], Event::PlayerAdded(1).encode())));

    sys.spend_blocks(5000);

    let res4 = lt.send(USERS[0], Action::PickWinner);

    println!("Winner index: {:?}", res4.decoded_log::<Event>());
    assert!(
        res4.contains(&(USERS[0], Event::Winner(0).encode()))
            || res4.contains(&(USERS[0], Event::Winner(1).encode()))
    );

    lt.send(USERS[0], Action::LotteryState);
}

#[test]
fn get_players() {
    let mut map: BTreeMap<u32, Player> = BTreeMap::new();
    map.insert(
        0,
        Player {
            player_id: USERS[0].into(),
            balance: 1000,
        },
    );

    map.insert(
        1,
        Player {
            player_id: USERS[1].into(),
            balance: 2000,
        },
    );

    let sys = System::new();
    init(&sys);
    let lt = sys.get_program(1);

    let res = lt.send(USERS[0], Action::StartLottery(20000));
    assert!(res.log().is_empty());

    let res2 = lt.send_with_value(USERS[0], Action::Enter, 1000);
    assert!(res2.contains(&(USERS[0], Event::PlayerAdded(0).encode())));

    let res3 = lt.send_with_value(USERS[1], Action::Enter, 2000);
    assert!(res3.contains(&(USERS[1], Event::PlayerAdded(1).encode())));

    let res4 = lt.send(USERS[0], Action::GetPlayers);
    assert!(res4.contains(&(USERS[0], Event::Players(map.clone()).encode())));
}

#[test]
fn leave_lottery() {
    let mut map: BTreeMap<u32, Player> = BTreeMap::new();
    map.insert(
        0,
        Player {
            player_id: USERS[0].into(),
            balance: 1000,
        },
    );

    let sys = System::new();
    init(&sys);
    let lt = sys.get_program(1);

    let res = lt.send(USERS[0], Action::StartLottery(20000));
    assert!(res.log().is_empty());

    let res2 = lt.send_with_value(USERS[0], Action::Enter, 1000);
    assert!(res2.contains(&(USERS[0], Event::PlayerAdded(0).encode())));

    let res3 = lt.send_with_value(USERS[1], Action::Enter, 2000);
    assert!(res3.contains(&(USERS[1], Event::PlayerAdded(1).encode())));

    lt.send(USERS[1], Action::LeaveLottery(1));

    let res4 = lt.send(USERS[0], Action::GetPlayers);
    assert!(res4.contains(&(USERS[0], Event::Players(map.clone()).encode())));
}

#[test]
fn get_balance() {
    let sys = System::new();
    init(&sys);
    let lt = sys.get_program(1);

    let res = lt.send(USERS[0], Action::StartLottery(20000));

    assert!(res.log().is_empty());

    let res2 = lt.send_with_value(USERS[0], Action::Enter, 1000);
    assert!(res2.contains(&(USERS[0], Event::PlayerAdded(0).encode())));

    let res3 = lt.send_with_value(USERS[1], Action::Enter, 2000);
    assert!(res3.contains(&(USERS[1], Event::PlayerAdded(1).encode())));

    let res4 = lt.send(USERS[0], Action::BalanceOf(0));
    assert!(res4.contains(&(USERS[0], Event::Balance(1000).encode())));

    let res5 = lt.send(USERS[1], Action::BalanceOf(1));
    assert!(res5.contains(&(USERS[1], Event::Balance(2000).encode())));
}
