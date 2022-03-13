use codec::Encode;
use lt_io::*;
use gstd::{/*String, ActorId,*/ BTreeMap};
use gtest::{Program, System};
//use std::println;
const USERS: &'static [u64] = &[3, 4, 5];

fn init(sys: &System) {
    sys.init_logger();

    let ft = Program::from_file(&sys,
        "./target/wasm32-unknown-unknown/release/lottery.wasm",);

    let res = ft.send_with_value(USERS[0], InitConfig {owner: USERS[0].into()}, 10000);

    assert!(res.log().is_empty());    
}

#[test]
fn add_player() {
    let sys = System::new();
    init(&sys);
    let lt = sys.get_program(1);
    let res = lt.send_with_value(USERS[0], Action::Enter(USERS[0].into()), 1000);
    assert!(res.contains(&(USERS[0], Event::PlayerAdded(0).encode())));

    let res2 = lt.send_with_value(USERS[0], Action::Enter(USERS[1].into()), 2000);
    assert!(res2.contains(&(USERS[0], Event::PlayerAdded(1).encode())));
}

#[test]
fn pick_winner() {    
    let sys = System::new();
    init(&sys);
    let lt = sys.get_program(1);
    let res = lt.send_with_value(USERS[0], Action::Enter(USERS[0].into()), 1000);
    assert!(res.contains(&(USERS[0], Event::PlayerAdded(0).encode())));

    let res2 = lt.send_with_value(USERS[0], Action::Enter(USERS[1].into()), 2000);
    assert!(res2.contains(&(USERS[0], Event::PlayerAdded(1).encode())));

    let res3 = lt.send(USERS[0], Action::Start);

    //println!("Winner = {}", );
    //println!(res.decoded_log::Winner());

    /*if res3.contains(&(USERS[0], Event::Winner(0).encode())) == false{
        assert!(res3.contains(&(USERS[0], Event::Winner(0).encode())));
    }
    else{*/
        assert!(res3.contains(&(USERS[0], Event::Winner(1).encode())));
    //}    
}

#[test]
fn add_value() {
    let sys = System::new();
    init(&sys);
    let lt = sys.get_program(1);
    let res = lt.send_with_value(USERS[0], Action::Enter(USERS[0].into()), 1000);
    assert!(res.contains(&(USERS[0], Event::PlayerAdded(0).encode())));

    let res2 = lt.send_with_value(USERS[0], Action::Enter(USERS[1].into()), 2000);
    assert!(res2.contains(&(USERS[0], Event::PlayerAdded(1).encode())));

    let res3 = lt.send_with_value(USERS[0], Action::AddValue(1), 500);
    assert!(res3.log().is_empty());

    let res4 = lt.send(USERS[0], Action::BalanceOf(1));
    assert!(res4.contains(&(USERS[0], Event::Balance(2500).encode())));
}

#[test]
fn get_players() {
    let mut map: BTreeMap<u32, Player> = BTreeMap::new();
    map.insert(0, Player{player: USERS[0].into(),
                         balance: 1000,
                        });

    map.insert(1, Player{player: USERS[1].into(),
                         balance: 2000,
                        });                        

    let sys = System::new();
    init(&sys);
    let lt = sys.get_program(1);

    let res = lt.send_with_value(USERS[0], Action::Enter(USERS[0].into()), 1000);
    assert!(res.contains(&(USERS[0], Event::PlayerAdded(0).encode())));

    let res2 = lt.send_with_value(USERS[0], Action::Enter(USERS[1].into()), 2000);
    assert!(res2.contains(&(USERS[0], Event::PlayerAdded(1).encode())));    

    let res3 = lt.send(USERS[0], Action::GetPlayers);
    assert!(res3.contains(&(USERS[0], Event::Players(map.clone()).encode())));
}

#[test]
fn del_player() {
    let mut map: BTreeMap<u32, Player> = BTreeMap::new();
    map.insert(0, Player{player: USERS[0].into(),
                         balance: 1000,
                        });

    let sys = System::new();
    init(&sys);
    let lt = sys.get_program(1);

    let res = lt.send_with_value(USERS[0], Action::Enter(USERS[0].into()), 1000);
    assert!(res.contains(&(USERS[0], Event::PlayerAdded(0).encode())));

    let res2 = lt.send_with_value(USERS[0], Action::Enter(USERS[1].into()), 2000);
    assert!(res2.contains(&(USERS[0], Event::PlayerAdded(1).encode())));

    let res3 = lt.send(USERS[0], Action::DelPlayer(1));
    assert!(res3.log().is_empty());

    let res4 = lt.send(USERS[0], Action::GetPlayers);
    assert!(res4.contains(&(USERS[0], Event::Players(map.clone()).encode())));
}

#[test]
fn get_balance() {
    let sys = System::new();
    init(&sys);
    let lt = sys.get_program(1);
    let res1 = lt.send_with_value(USERS[0], Action::Enter(USERS[0].into()), 1000);
    assert!(res1.contains(&(USERS[0], Event::PlayerAdded(0).encode())));

    let res2 = lt.send_with_value(USERS[0], Action::Enter(USERS[1].into()), 2000);
    assert!(res2.contains(&(USERS[0], Event::PlayerAdded(1).encode())));

    let res3 = lt.send(USERS[0], Action::BalanceOf(0));
    assert!(res3.contains(&(USERS[0], Event::Balance(1000).encode())));

    let res4 = lt.send(USERS[0], Action::BalanceOf(1));
    assert!(res4.contains(&(USERS[0], Event::Balance(2000).encode())));
}