#[cfg(test)]
extern crate std;
#[cfg(test)]
use std::println;
use std::time::{SystemTime, UNIX_EPOCH};

use codec::Encode;
use gstd::BTreeMap;
use gtest::{Program, System};
use lt_io::*;
const USERS: &'static [u64] = &[3, 4, 5];

fn init(sys: &System) {
    sys.init_logger();

    let lt = Program::current(&sys);

    let res = lt.send_bytes_with_value(USERS[0], b"Init", 10000);

    assert!(res.log().is_empty());
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
        Action::StartLottery {
            duration: 5000,
            token_address: None,
        },
    );
    assert!(res.log().is_empty());

    println!("time: {}", time);

    let res = lt.send(USERS[0], Action::LotteryState);
    assert!(res.contains(&(USERS[0], Event::LotteryState(state).encode())));
}

#[test]
fn enter() {
    let sys = System::new();
    init(&sys);
    let lt = sys.get_program(1);

    let res = lt.send(
        USERS[0],
        Action::StartLottery {
            duration: 5000,
            token_address: None,
        },
    );
    assert!(res.log().is_empty());

    let res = lt.send_with_value(USERS[0], Action::Enter(1000), 1000);
    assert!(res.contains(&(USERS[0], Event::PlayerAdded(0).encode())));

    let res = lt.send_with_value(USERS[1], Action::Enter(2000), 2000);
    assert!(res.contains(&(USERS[1], Event::PlayerAdded(1).encode())));
}

#[test]
fn pick_winner() {
    let sys = System::new();
    init(&sys);
    let lt = sys.get_program(1);

    let res = lt.send(
        USERS[0],
        Action::StartLottery {
            duration: 5000,
            token_address: None,
        },
    );
    assert!(res.log().is_empty());

    let res = lt.send_with_value(USERS[0], Action::Enter(1000), 1000);
    assert!(res.contains(&(USERS[0], Event::PlayerAdded(0).encode())));

    let res = lt.send_with_value(USERS[1], Action::Enter(2000), 2000);
    assert!(res.contains(&(USERS[1], Event::PlayerAdded(1).encode())));

    sys.spend_blocks(5000);

    let res = lt.send(USERS[0], Action::PickWinner);

    println!("Winner index: {:?}", res.decoded_log::<Event>());
    assert!(
        res.contains(&(USERS[0], Event::Winner(0).encode()))
            || res.contains(&(USERS[0], Event::Winner(1).encode()))
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

    let res = lt.send(
        USERS[0],
        Action::StartLottery {
            duration: 5000,
            token_address: None,
        },
    );
    assert!(res.log().is_empty());

    let res = lt.send_with_value(USERS[0], Action::Enter(1000), 1000);
    assert!(res.contains(&(USERS[0], Event::PlayerAdded(0).encode())));

    let res = lt.send_with_value(USERS[1], Action::Enter(2000), 2000);
    assert!(res.contains(&(USERS[1], Event::PlayerAdded(1).encode())));

    let res = lt.send(USERS[0], Action::GetPlayers);
    assert!(res.contains(&(USERS[0], Event::Players(map.clone()).encode())));
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

    let res = lt.send(
        USERS[0],
        Action::StartLottery {
            duration: 5000,
            token_address: None,
        },
    );
    assert!(res.log().is_empty());

    let res = lt.send_with_value(USERS[0], Action::Enter(1000), 1000);
    assert!(res.contains(&(USERS[0], Event::PlayerAdded(0).encode())));

    let res = lt.send_with_value(USERS[1], Action::Enter(2000), 2000);
    assert!(res.contains(&(USERS[1], Event::PlayerAdded(1).encode())));

    let res = lt.send(USERS[1], Action::LeaveLottery(1));
    assert!(!res.main_failed());

    let res = lt.send(USERS[0], Action::GetPlayers);
    assert!(res.contains(&(USERS[0], Event::Players(map.clone()).encode())));
}

#[test]
fn get_balance() {
    let sys = System::new();
    init(&sys);
    let lt = sys.get_program(1);

    let res = lt.send(
        USERS[0],
        Action::StartLottery {
            duration: 5000,
            token_address: None,
        },
    );

    assert!(res.log().is_empty());

    let res = lt.send_with_value(USERS[0], Action::Enter(1000), 1000);
    assert!(res.contains(&(USERS[0], Event::PlayerAdded(0).encode())));

    let res = lt.send_with_value(USERS[1], Action::Enter(2000), 2000);
    assert!(res.contains(&(USERS[1], Event::PlayerAdded(1).encode())));

    let res = lt.send(USERS[0], Action::BalanceOf(0));
    assert!(res.contains(&(USERS[0], Event::Balance(1000).encode())));

    let res = lt.send(USERS[0], Action::BalanceOf(1));
    assert!(res.contains(&(USERS[0], Event::Balance(2000).encode())));
}
