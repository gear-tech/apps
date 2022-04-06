#[cfg(test)]
extern crate std;
#[cfg(test)]
use std::println;
use std::time::{SystemTime, UNIX_EPOCH};

use codec::Encode;
use gstd::BTreeMap;
use gtest::{Program, System};
use lt_io::*;
const USERS: &'static [u64] = &[3, 4, 5, 0];

fn init(sys: &System) {
    sys.init_logger();

    let lt = Program::current(&sys);

    let res = lt.send_bytes_with_value(USERS[0], b"Init", 10000);

    assert!(res.log().is_empty());
}

#[test]
fn zero_address() {
    let sys = System::new();
    init(&sys);
    let lt = sys.get_program(1);

    let res = lt.send(
        USERS[3],
        LtAction::StartLottery {
            duration: 5000,
            token_address: None,
        },
    );
    assert!(res.main_failed());
}

#[test]
fn start_lottery() {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let state = LotteryState {
        lottery_started: true,
        lottery_start_time: time,
        lottery_duration: 5000,
    };

    let sys = System::new();
    init(&sys);
    let lt = sys.get_program(1);

    let res = lt.send(
        USERS[0],
        LtAction::StartLottery {
            duration: 5000,
            token_address: None,
        },
    );
    assert!(res.log().is_empty());

    let res = lt.send(
        USERS[1],
        LtAction::StartLottery {
            duration: 30000,
            token_address: None,
        },
    );
    assert!(res.main_failed());

    let res = lt.send(
        USERS[0],
        LtAction::StartLottery {
            duration: 40000,
            token_address: None,
        },
    );
    assert!(res.main_failed());

    println!("time: {}", time);

    let res = lt.send(USERS[0], LtAction::LotteryState);
    assert!(res.contains(&(USERS[0], LtEvent::LotteryState(state).encode())));
}

#[test]
fn enter() {
    let sys = System::new();
    init(&sys);
    let lt = sys.get_program(1);

    let res = lt.send_with_value(USERS[1], LtAction::Enter(3000), 3000);
    assert!(res.main_failed());

    let res = lt.send(
        USERS[0],
        LtAction::StartLottery {
            duration: 5000,
            token_address: None,
        },
    );
    assert!(res.log().is_empty());

    let res = lt.send_with_value(USERS[0], LtAction::Enter(1000), 1000);
    assert!(res.contains(&(USERS[0], LtEvent::PlayerAdded(0).encode())));

    let res = lt.send_with_value(USERS[1], LtAction::Enter(2000), 2000);
    assert!(res.contains(&(USERS[1], LtEvent::PlayerAdded(1).encode())));

    let res = lt.send_with_value(USERS[1], LtAction::Enter(3000), 3000);
    assert!(res.main_failed());
}

#[test]
fn pick_winner() {
    let sys = System::new();
    init(&sys);
    let lt = sys.get_program(1);

    let res = lt.send(USERS[0], LtAction::PickWinner);
    assert!(res.main_failed());

    let res = lt.send(
        USERS[0],
        LtAction::StartLottery {
            duration: 5000,
            token_address: None,
        },
    );
    assert!(res.log().is_empty());

    let res = lt.send(USERS[0], LtAction::PickWinner);
    assert!(res.main_failed());

    let res = lt.send_with_value(USERS[0], LtAction::Enter(1000), 1000);
    assert!(res.contains(&(USERS[0], LtEvent::PlayerAdded(0).encode())));

    let res = lt.send_with_value(USERS[1], LtAction::Enter(2000), 2000);
    assert!(res.contains(&(USERS[1], LtEvent::PlayerAdded(1).encode())));

    sys.spend_blocks(5000);

    let res = lt.send(USERS[1], LtAction::PickWinner);
    assert!(res.main_failed());

    let res = lt.send(USERS[0], LtAction::PickWinner);

    println!("Winner index: {:?}", res.decoded_log::<Event>());
    assert!(
        res.contains(&(USERS[0], LtEvent::Winner(0).encode()))
            || res.contains(&(USERS[0], LtEvent::Winner(1).encode()))
    );
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

    let res = lt.send(USERS[0], LtAction::GetPlayers);
    assert!(res.main_failed());

    let res = lt.send(
        USERS[0],
        LtAction::StartLottery {
            duration: 5000,
            token_address: None,
        },
    );
    assert!(res.log().is_empty());

    let res = lt.send_with_value(USERS[0], LtAction::Enter(1000), 1000);
    assert!(res.contains(&(USERS[0], LtEvent::PlayerAdded(0).encode())));

    let res = lt.send_with_value(USERS[1], LtAction::Enter(2000), 2000);
    assert!(res.contains(&(USERS[1], LtEvent::PlayerAdded(1).encode())));

    let res = lt.send(USERS[0], LtAction::GetPlayers);
    assert!(res.contains(&(USERS[0], LtEvent::Players(map.clone()).encode())));
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

    let res = lt.send(USERS[1], LtAction::LeaveLottery(1));
    assert!(res.main_failed());

    let res = lt.send(
        USERS[0],
        LtAction::StartLottery {
            duration: 5000,
            token_address: None,
        },
    );
    assert!(res.log().is_empty());

    let res = lt.send(USERS[1], LtAction::LeaveLottery(1));
    assert!(res.main_failed());

    let res = lt.send_with_value(USERS[0], LtAction::Enter(1000), 1000);
    assert!(res.contains(&(USERS[0], LtEvent::PlayerAdded(0).encode())));

    let res = lt.send_with_value(USERS[1], LtAction::Enter(2000), 2000);
    assert!(res.contains(&(USERS[1], LtEvent::PlayerAdded(1).encode())));

    let res = lt.send(USERS[1], LtAction::LeaveLottery(1));
    assert!(!res.main_failed());

    let res = lt.send(USERS[1], LtAction::LeaveLottery(0));
    assert!(res.main_failed());

    let res = lt.send(USERS[0], LtAction::GetPlayers);
    assert!(res.contains(&(USERS[0], LtEvent::Players(map.clone()).encode())));
}

#[test]
fn get_balance() {
    let sys = System::new();
    init(&sys);
    let lt = sys.get_program(1);

    let res = lt.send(USERS[1], LtAction::BalanceOf(1));
    assert!(res.main_failed());

    let res = lt.send(
        USERS[0],
        LtAction::StartLottery {
            duration: 5000,
            token_address: None,
        },
    );
    assert!(res.log().is_empty());

    let res = lt.send(USERS[1], LtAction::BalanceOf(1));
    assert!(res.main_failed());

    let res = lt.send_with_value(USERS[0], LtAction::Enter(1000), 1000);
    assert!(res.contains(&(USERS[0], LtEvent::PlayerAdded(0).encode())));

    let res = lt.send_with_value(USERS[1], LtAction::Enter(2000), 2000);
    assert!(res.contains(&(USERS[1], LtEvent::PlayerAdded(1).encode())));

    let res = lt.send(USERS[0], LtAction::BalanceOf(0));
    assert!(res.contains(&(USERS[0], LtEvent::Balance(1000).encode())));

    let res = lt.send(USERS[0], LtAction::BalanceOf(1));
    assert!(res.contains(&(USERS[0], LtEvent::Balance(2000).encode())));
}
